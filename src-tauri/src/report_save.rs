//! Bounded, backend-owned plain-text report saving.
//!
//! The WebView supplies only an opaque generation token. Dialog selection,
//! staging, synchronization, and the no-clobber commit remain native. Tests
//! inject artificial adapters and never open a dialog or touch a filesystem.

use serde::Serialize;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

const MAX_REPORT_BYTES: usize = 1_048_576;
const STAGING_ATTEMPTS: u64 = 64;
const STAGING_PREFIX: &str = ".ai-engine-room-report.tmp-";

pub fn register_dialog<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.plugin(tauri_plugin_dialog::init())
}

fn staging_name(counter: u64, attempt: u64) -> String {
    format!("{STAGING_PREFIX}{counter:016x}-{attempt:02x}")
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReportPreviewResponse {
    pub text: String,
    pub generation: String,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportPreviewError {
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportSaveResult {
    Saved,
    Cancelled,
    Busy,
    PreviewChanged,
    InvalidDestination,
    DestinationExists,
    Unavailable,
    Failed,
    CleanupIncomplete,
    CompletionUncertain,
}

#[derive(Default)]
struct PreviewState {
    next_generation: u64,
    current: Option<(String, Arc<[u8]>)>,
}

struct ReportSaveInner {
    preview: Mutex<PreviewState>,
    active: AtomicBool,
    next_staging: AtomicU64,
}

#[derive(Clone)]
pub struct ReportSaveState {
    inner: Arc<ReportSaveInner>,
}

impl Default for ReportSaveState {
    fn default() -> Self {
        Self {
            inner: Arc::new(ReportSaveInner {
                preview: Mutex::new(PreviewState::default()),
                active: AtomicBool::new(false),
                next_staging: AtomicU64::new(0),
            }),
        }
    }
}

impl ReportSaveState {
    pub fn retain_preview(
        &self,
        text: String,
    ) -> Result<ReportPreviewResponse, ReportPreviewError> {
        let mut state = self
            .inner
            .preview
            .lock()
            .map_err(|_| ReportPreviewError::Unavailable)?;
        let Some(next) = state.next_generation.checked_add(1) else {
            state.current = None;
            return Err(ReportPreviewError::Unavailable);
        };
        state.next_generation = next;
        let generation = format!("{next:016x}");
        state.current = Some((generation.clone(), Arc::from(text.as_bytes())));
        Ok(ReportPreviewResponse { text, generation })
    }

    fn begin(&self, generation: &str) -> Result<(SaveLease, Arc<[u8]>), ReportSaveResult> {
        if !valid_generation(generation) {
            return Err(ReportSaveResult::PreviewChanged);
        }
        let bytes = {
            let state = self
                .inner
                .preview
                .lock()
                .map_err(|_| ReportSaveResult::Failed)?;
            let Some((current, bytes)) = &state.current else {
                return Err(ReportSaveResult::PreviewChanged);
            };
            if current != generation {
                return Err(ReportSaveResult::PreviewChanged);
            }
            if bytes.is_empty() || bytes.len() > MAX_REPORT_BYTES {
                return Err(ReportSaveResult::Unavailable);
            }
            Arc::clone(bytes)
        };
        if self
            .inner
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(ReportSaveResult::Busy);
        }
        Ok((SaveLease(Arc::clone(&self.inner)), bytes))
    }

    fn is_current(&self, generation: &str) -> bool {
        self.inner
            .preview
            .lock()
            .ok()
            .and_then(|state| state.current.as_ref().map(|(value, _)| value == generation))
            .unwrap_or(false)
    }

    fn next_staging_counter(&self) -> Option<u64> {
        self.inner
            .next_staging
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .ok()
            .and_then(|value| value.checked_add(1))
    }
}

struct SaveLease(Arc<ReportSaveInner>);

impl Drop for SaveLease {
    fn drop(&mut self) {
        self.0.active.store(false, Ordering::Release);
    }
}

fn valid_generation(value: &str) -> bool {
    value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

trait DialogAdapter {
    fn destination(&self) -> Result<Option<PathBuf>, ReportSaveResult>;
}

trait StorageAdapter {
    type Staging;

    fn create_staging(
        &self,
        parent: &Path,
        counter: u64,
    ) -> Result<Self::Staging, ReportSaveResult>;
    fn write_and_sync(&self, staging: &mut Self::Staging, bytes: &[u8]) -> io::Result<()>;
    fn commit(&self, staging: &mut Self::Staging, destination: &Path) -> CommitResult;
    fn cleanup(&self, staging: &mut Self::Staging) -> CleanupResult;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitResult {
    Saved,
    DestinationExists,
    Unavailable,
    Failed,
    CompletionUncertain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CleanupResult {
    Removed,
    Absent,
    Failed,
}

fn normalize_destination(mut path: PathBuf) -> Result<PathBuf, ReportSaveResult> {
    if path.file_name().is_none() {
        return Err(ReportSaveResult::InvalidDestination);
    }
    match path.extension().and_then(OsStr::to_str) {
        None => {
            path.set_extension("txt");
        }
        Some(extension) if extension.eq_ignore_ascii_case("txt") => {}
        Some(_) => return Err(ReportSaveResult::InvalidDestination),
    }
    Ok(path)
}

fn run_save<D: DialogAdapter, S: StorageAdapter>(
    state: &ReportSaveState,
    generation: &str,
    dialog: &D,
    storage: &S,
) -> ReportSaveResult {
    let (_lease, bytes) = match state.begin(generation) {
        Ok(value) => value,
        Err(result) => return result,
    };
    let destination = match dialog.destination() {
        Ok(Some(path)) => match normalize_destination(path) {
            Ok(path) => path,
            Err(result) => return result,
        },
        Ok(None) => return ReportSaveResult::Cancelled,
        Err(result) => return result,
    };
    if !state.is_current(generation) {
        return ReportSaveResult::PreviewChanged;
    }
    let Some(parent) = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    else {
        return ReportSaveResult::InvalidDestination;
    };
    let Some(counter) = state.next_staging_counter() else {
        return ReportSaveResult::Unavailable;
    };
    let mut staging = match storage.create_staging(parent, counter) {
        Ok(staging) => staging,
        Err(result) => return result,
    };
    if storage.write_and_sync(&mut staging, &bytes).is_err() {
        return match storage.cleanup(&mut staging) {
            CleanupResult::Removed | CleanupResult::Absent => ReportSaveResult::Failed,
            CleanupResult::Failed => ReportSaveResult::CleanupIncomplete,
        };
    }
    match storage.commit(&mut staging, &destination) {
        CommitResult::Saved => ReportSaveResult::Saved,
        CommitResult::DestinationExists => {
            cleanup_after(storage, &mut staging, ReportSaveResult::DestinationExists)
        }
        CommitResult::Unavailable => {
            cleanup_after(storage, &mut staging, ReportSaveResult::Unavailable)
        }
        CommitResult::Failed => cleanup_after(storage, &mut staging, ReportSaveResult::Failed),
        CommitResult::CompletionUncertain => match storage.cleanup(&mut staging) {
            CleanupResult::Removed => ReportSaveResult::Failed,
            CleanupResult::Absent => ReportSaveResult::CompletionUncertain,
            CleanupResult::Failed => ReportSaveResult::CleanupIncomplete,
        },
    }
}

fn cleanup_after<S: StorageAdapter>(
    storage: &S,
    staging: &mut S::Staging,
    result: ReportSaveResult,
) -> ReportSaveResult {
    match storage.cleanup(staging) {
        CleanupResult::Removed | CleanupResult::Absent => result,
        CleanupResult::Failed => ReportSaveResult::CleanupIncomplete,
    }
}

struct NativeDialog {
    app: tauri::AppHandle,
}

impl DialogAdapter for NativeDialog {
    fn destination(&self) -> Result<Option<PathBuf>, ReportSaveResult> {
        let window = self
            .app
            .get_webview_window("main")
            .ok_or(ReportSaveResult::Unavailable)?;
        let selected = self
            .app
            .dialog()
            .file()
            .set_parent(&window)
            .set_title("Save AI Engine Room report")
            .set_file_name("ai-engine-room-report.txt")
            .add_filter("Plain text", &["txt"])
            .blocking_save_file();
        selected
            .map(|path| match path {
                tauri_plugin_dialog::FilePath::Path(path) => Ok(path),
                tauri_plugin_dialog::FilePath::Url(_) => Err(ReportSaveResult::InvalidDestination),
            })
            .transpose()
    }
}

struct NativeStaging {
    path: PathBuf,
    file: Option<File>,
}

struct NativeStorage;

impl StorageAdapter for NativeStorage {
    type Staging = NativeStaging;

    fn create_staging(
        &self,
        parent: &Path,
        counter: u64,
    ) -> Result<Self::Staging, ReportSaveResult> {
        for attempt in 0..STAGING_ATTEMPTS {
            let path = parent.join(staging_name(counter, attempt));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => {
                    return Ok(NativeStaging {
                        path,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err(ReportSaveResult::Failed),
            }
        }
        Err(ReportSaveResult::Unavailable)
    }

    fn write_and_sync(&self, staging: &mut Self::Staging, bytes: &[u8]) -> io::Result<()> {
        let file = staging
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("closed"))?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        staging.file.take();
        Ok(())
    }

    fn commit(&self, staging: &mut Self::Staging, destination: &Path) -> CommitResult {
        staging.file.take();
        platform_commit(&staging.path, destination)
    }

    fn cleanup(&self, staging: &mut Self::Staging) -> CleanupResult {
        staging.file.take();
        match std::fs::remove_file(&staging.path) {
            Ok(()) => CleanupResult::Removed,
            Err(error) if error.kind() == io::ErrorKind::NotFound => CleanupResult::Absent,
            Err(_) => CleanupResult::Failed,
        }
    }
}

#[cfg(target_os = "linux")]
fn platform_commit(staging: &Path, destination: &Path) -> CommitResult {
    use std::os::unix::ffi::OsStrExt;

    let parent = destination.parent().map(Path::to_path_buf);
    let Ok(staging) = std::ffi::CString::new(staging.as_os_str().as_bytes()) else {
        return CommitResult::Failed;
    };
    let Ok(destination) = std::ffi::CString::new(destination.as_os_str().as_bytes()) else {
        return CommitResult::Failed;
    };
    // Both C strings are NUL-terminated and remain alive for the call; the
    // pointers identify paths owned by this operation. AT_FDCWD is paired with
    // exactly RENAME_NOREPLACE, and errno is retrieved immediately on failure.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            staging.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        let Some(parent) = parent else {
            return CommitResult::CompletionUncertain;
        };
        return match File::open(parent).and_then(|directory| directory.sync_all()) {
            Ok(()) => CommitResult::Saved,
            Err(_) => CommitResult::CompletionUncertain,
        };
    }
    match io::Error::last_os_error().raw_os_error() {
        Some(libc::EEXIST) => CommitResult::DestinationExists,
        Some(code)
            if matches!(
                code,
                libc::ENOSYS | libc::EINVAL | libc::EOPNOTSUPP | libc::EXDEV
            ) =>
        {
            CommitResult::Unavailable
        }
        Some(_) => CommitResult::CompletionUncertain,
        None => CommitResult::Failed,
    }
}

#[cfg(target_os = "windows")]
fn platform_commit(staging: &Path, destination: &Path) -> CommitResult {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS};
    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    let source: Vec<u16> = staging.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    // Both UTF-16 buffers are NUL-terminated and remain alive for the call.
    // Their pointers identify paths owned by this operation. A zero flag value
    // forbids replacement, copy fallback, and delayed execution; the OS error
    // is retrieved immediately if the call fails.
    let result = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) };
    if result != 0 {
        return CommitResult::Saved;
    }
    match io::Error::last_os_error()
        .raw_os_error()
        .map(|value| value as u32)
    {
        Some(code) if matches!(code, ERROR_FILE_EXISTS | ERROR_ALREADY_EXISTS) => {
            CommitResult::DestinationExists
        }
        Some(_) => CommitResult::CompletionUncertain,
        None => CommitResult::Failed,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn platform_commit(_staging: &Path, _destination: &Path) -> CommitResult {
    CommitResult::Unavailable
}

pub fn save_native(
    state: ReportSaveState,
    app: tauri::AppHandle,
    generation: String,
) -> ReportSaveResult {
    run_save(&state, &generation, &NativeDialog { app }, &NativeStorage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    struct FakeDialog {
        result: RefCell<Result<Option<PathBuf>, ReportSaveResult>>,
        calls: Cell<usize>,
    }

    impl DialogAdapter for FakeDialog {
        fn destination(&self) -> Result<Option<PathBuf>, ReportSaveResult> {
            self.calls.set(self.calls.get() + 1);
            self.result.borrow().clone()
        }
    }

    struct FakeStorage {
        staged: RefCell<Vec<u8>>,
        create_calls: Cell<usize>,
        write_error: Cell<bool>,
        commit: Cell<CommitResult>,
        cleanup: Cell<CleanupResult>,
        cleanup_calls: Cell<usize>,
    }

    impl Default for FakeStorage {
        fn default() -> Self {
            Self {
                staged: RefCell::new(Vec::new()),
                create_calls: Cell::new(0),
                write_error: Cell::new(false),
                commit: Cell::new(CommitResult::Saved),
                cleanup: Cell::new(CleanupResult::Removed),
                cleanup_calls: Cell::new(0),
            }
        }
    }

    impl StorageAdapter for FakeStorage {
        type Staging = ();

        fn create_staging(&self, _parent: &Path, _counter: u64) -> Result<(), ReportSaveResult> {
            self.create_calls.set(self.create_calls.get() + 1);
            Ok(())
        }

        fn write_and_sync(&self, _staging: &mut (), bytes: &[u8]) -> io::Result<()> {
            if self.write_error.get() {
                return Err(io::Error::other("artificial failure"));
            }
            self.staged.replace(bytes.to_vec());
            Ok(())
        }

        fn commit(&self, _staging: &mut (), _destination: &Path) -> CommitResult {
            self.commit.get()
        }

        fn cleanup(&self, _staging: &mut ()) -> CleanupResult {
            self.cleanup_calls.set(self.cleanup_calls.get() + 1);
            self.cleanup.get()
        }
    }

    fn setup() -> (ReportSaveState, String, String) {
        let state = ReportSaveState::default();
        let text = "AI Engine Room — Observation Report\n".to_string();
        let generation = state.retain_preview(text.clone()).unwrap().generation;
        (state, generation, text)
    }

    fn dialog(path: Option<&str>) -> FakeDialog {
        FakeDialog {
            result: RefCell::new(Ok(path.map(PathBuf::from))),
            calls: Cell::new(0),
        }
    }

    #[test]
    fn retains_exact_utf8_and_checked_generation() {
        let state = ReportSaveState::default();
        let first = state.retain_preview("line one\n".into()).unwrap();
        let second = state.retain_preview("line two\n\n".into()).unwrap();
        assert_eq!(first.generation, "0000000000000001");
        assert_eq!(second.generation, "0000000000000002");
        assert_eq!(second.text.as_bytes(), b"line two\n\n");
        assert!(!state.is_current(&first.generation));
        assert!(state.is_current(&second.generation));
    }

    #[test]
    fn result_model_and_generation_token_are_closed() {
        assert_eq!(
            serde_json::to_string(&ReportSaveResult::Saved).unwrap(),
            "\"saved\""
        );
        assert_eq!(
            serde_json::to_string(&ReportSaveResult::CompletionUncertain).unwrap(),
            "\"completion_uncertain\""
        );
        assert!(valid_generation("00000000000000af"));
        assert!(!valid_generation("00000000000000AF"));
        assert!(!valid_generation("00000000000000ag"));
        assert!(!valid_generation("short"));
    }

    #[test]
    fn generation_overflow_clears_retained_preview_and_fails_closed() {
        let state = ReportSaveState::default();
        state.retain_preview("first\n".into()).unwrap();
        {
            let mut preview = state.inner.preview.lock().unwrap();
            preview.next_generation = u64::MAX;
        }
        assert_eq!(
            state.retain_preview("replacement\n".into()),
            Err(ReportPreviewError::Unavailable)
        );
        assert!(!state.is_current("0000000000000001"));
    }

    #[test]
    fn staging_names_are_bounded_and_contain_no_report_or_machine_data() {
        let names = (0..STAGING_ATTEMPTS)
            .map(|attempt| staging_name(7, attempt))
            .collect::<Vec<_>>();
        assert_eq!(names.len(), 64);
        assert_eq!(names[0], ".ai-engine-room-report.tmp-0000000000000007-00");
        assert_eq!(names[63], ".ai-engine-room-report.tmp-0000000000000007-3f");
        assert!(names.iter().all(|name| !name.contains("Observation")));
    }

    #[test]
    fn saves_exact_bytes_through_injected_seams() {
        let (state, generation, text) = setup();
        let storage = FakeStorage::default();
        assert_eq!(
            run_save(
                &state,
                &generation,
                &dialog(Some("chosen/report.txt")),
                &storage
            ),
            ReportSaveResult::Saved
        );
        assert_eq!(&*storage.staged.borrow(), text.as_bytes());
        assert!(!storage.staged.borrow().starts_with(&[0xef, 0xbb, 0xbf]));
    }

    #[test]
    fn cancellation_and_invalid_input_do_not_stage() {
        let (state, generation, _) = setup();
        for (selected, expected) in [
            (None, ReportSaveResult::Cancelled),
            (
                Some("chosen/report.json"),
                ReportSaveResult::InvalidDestination,
            ),
        ] {
            let storage = FakeStorage::default();
            assert_eq!(
                run_save(&state, &generation, &dialog(selected), &storage),
                expected
            );
            assert_eq!(storage.create_calls.get(), 0);
        }
        let storage = FakeStorage::default();
        let selected = dialog(Some("chosen/report.txt"));
        assert_eq!(
            run_save(&state, "INVALID", &selected, &storage),
            ReportSaveResult::PreviewChanged
        );
        assert_eq!(selected.calls.get(), 0);
    }

    #[test]
    fn empty_and_oversized_previews_fail_before_the_dialog() {
        for text in [String::new(), "x".repeat(MAX_REPORT_BYTES + 1)] {
            let state = ReportSaveState::default();
            let generation = state.retain_preview(text).unwrap().generation;
            let selected = dialog(Some("chosen/report.txt"));
            let storage = FakeStorage::default();
            assert_eq!(
                run_save(&state, &generation, &selected, &storage),
                ReportSaveResult::Unavailable
            );
            assert_eq!(selected.calls.get(), 0);
            assert_eq!(storage.create_calls.get(), 0);
        }
    }

    #[test]
    fn one_operation_guard_rejects_reentrant_save_and_releases_after_return() {
        struct ReentrantDialog<'a> {
            state: &'a ReportSaveState,
            generation: &'a str,
            storage: &'a FakeStorage,
            nested: Cell<Option<ReportSaveResult>>,
        }
        impl DialogAdapter for ReentrantDialog<'_> {
            fn destination(&self) -> Result<Option<PathBuf>, ReportSaveResult> {
                self.nested.set(Some(run_save(
                    self.state,
                    self.generation,
                    &dialog(None),
                    self.storage,
                )));
                Ok(None)
            }
        }

        let (state, generation, _) = setup();
        let storage = FakeStorage::default();
        let reentrant = ReentrantDialog {
            state: &state,
            generation: &generation,
            storage: &storage,
            nested: Cell::new(None),
        };
        assert_eq!(
            run_save(&state, &generation, &reentrant, &storage),
            ReportSaveResult::Cancelled
        );
        assert_eq!(reentrant.nested.get(), Some(ReportSaveResult::Busy));
        assert_eq!(
            run_save(&state, &generation, &dialog(None), &storage),
            ReportSaveResult::Cancelled
        );
    }

    #[test]
    fn extension_is_bounded_to_plain_text() {
        assert_eq!(
            normalize_destination(PathBuf::from("chosen/report")).unwrap(),
            PathBuf::from("chosen/report.txt")
        );
        assert!(normalize_destination(PathBuf::from("chosen/report.TXT")).is_ok());
        assert_eq!(
            normalize_destination(PathBuf::from("chosen/report.md")),
            Err(ReportSaveResult::InvalidDestination)
        );
    }

    #[test]
    fn stale_preview_after_dialog_stages_nothing() {
        struct RefreshingDialog(ReportSaveState);
        impl DialogAdapter for RefreshingDialog {
            fn destination(&self) -> Result<Option<PathBuf>, ReportSaveResult> {
                self.0.retain_preview("replacement\n".into()).unwrap();
                Ok(Some(PathBuf::from("chosen/report.txt")))
            }
        }
        let (state, generation, _) = setup();
        let storage = FakeStorage::default();
        assert_eq!(
            run_save(
                &state,
                &generation,
                &RefreshingDialog(state.clone()),
                &storage
            ),
            ReportSaveResult::PreviewChanged
        );
        assert_eq!(storage.create_calls.get(), 0);
    }

    #[test]
    fn write_and_commit_failures_have_closed_cleanup_results() {
        let cases = [
            (
                CommitResult::DestinationExists,
                CleanupResult::Removed,
                ReportSaveResult::DestinationExists,
            ),
            (
                CommitResult::Unavailable,
                CleanupResult::Removed,
                ReportSaveResult::Unavailable,
            ),
            (
                CommitResult::Failed,
                CleanupResult::Removed,
                ReportSaveResult::Failed,
            ),
            (
                CommitResult::Failed,
                CleanupResult::Failed,
                ReportSaveResult::CleanupIncomplete,
            ),
            (
                CommitResult::CompletionUncertain,
                CleanupResult::Absent,
                ReportSaveResult::CompletionUncertain,
            ),
            (
                CommitResult::CompletionUncertain,
                CleanupResult::Removed,
                ReportSaveResult::Failed,
            ),
        ];
        for (commit, cleanup, expected) in cases {
            let (state, generation, _) = setup();
            let storage = FakeStorage::default();
            storage.commit.set(commit);
            storage.cleanup.set(cleanup);
            assert_eq!(
                run_save(
                    &state,
                    &generation,
                    &dialog(Some("chosen/report.txt")),
                    &storage
                ),
                expected
            );
        }
    }

    #[test]
    fn write_failure_never_commits_and_reports_cleanup_uncertainty() {
        let (state, generation, _) = setup();
        let storage = FakeStorage::default();
        storage.write_error.set(true);
        storage.cleanup.set(CleanupResult::Failed);
        assert_eq!(
            run_save(
                &state,
                &generation,
                &dialog(Some("chosen/report.txt")),
                &storage
            ),
            ReportSaveResult::CleanupIncomplete
        );
        assert_eq!(storage.cleanup_calls.get(), 1);
    }
}
