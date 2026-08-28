//! Linux OS metric provider for AI Engine Room.
//!
//! Reads `/proc/meminfo` and exposes the kernel-supplied `MemAvailable` value
//! as `os.ram.available`. The kernel reports the value in kibibytes (labelled
//! "kB" in the file); this provider normalises it to bytes.
//!
//! The parser is intentionally small and standard-library only. It is tested
//! with controlled fixture strings so deterministic tests do not depend on the
//! live development machine.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use aer_core::{
    AcquisitionError, LimitationCode, MetricAvailability, MetricDescriptor, MetricId, MetricResult,
    MetricSample, OsMetricsProvider, Provenance, SampleValue, SourceAvailability, SourceClass,
    Unit,
};

// Ensure tests compile only on the same Linux target; the parser is still
// testable with fixture files, but the provider is gated to the Linux build.

const MEMINFO_DEFAULT_PATH: &str = "/proc/meminfo";
const RAM_AVAILABLE_ID: &str = "os.ram.available";
const RAM_TOTAL_ID: &str = "os.ram.total";

/// Linux-specific provider for a narrow set of OS metrics.
pub struct LinuxMetricsProvider {
    meminfo_path: PathBuf,
    cached_meminfo: Mutex<Option<Result<String, ReadMeminfoError>>>,
}

#[derive(Clone, Copy)]
enum ReadMeminfoError {
    Permission,
    NotFound,
    Other,
}

impl LinuxMetricsProvider {
    /// Provider that reads from the live `/proc/meminfo`.
    pub fn new() -> Self {
        Self {
            meminfo_path: PathBuf::from(MEMINFO_DEFAULT_PATH),
            cached_meminfo: Mutex::new(None),
        }
    }

    /// Provider that reads from an arbitrary path, used by tests with fixture
    /// files so the parser and provider logic can be exercised deterministically.
    #[cfg(test)]
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            meminfo_path: path.into(),
            cached_meminfo: Mutex::new(None),
        }
    }
}

impl Default for LinuxMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OsMetricsProvider for LinuxMetricsProvider {
    fn source_availability(&self) -> SourceAvailability {
        if self.meminfo_path.exists() {
            SourceAvailability::Ready
        } else {
            SourceAvailability::NotDetected
        }
    }

    fn list_metrics(&self) -> Vec<MetricDescriptor> {
        vec![
            MetricDescriptor {
                id: MetricId::new(RAM_AVAILABLE_ID),
                display_name: String::from("Available memory"),
                unit: Unit::Bytes,
                source_class: SourceClass::OperatingSystem,
            },
            MetricDescriptor {
                id: MetricId::new(RAM_TOTAL_ID),
                display_name: String::from("Total memory"),
                unit: Unit::Bytes,
                source_class: SourceClass::OperatingSystem,
            },
        ]
    }

    fn acquire(&self, id: &MetricId) -> MetricResult {
        if id.as_str() != RAM_AVAILABLE_ID && id.as_str() != RAM_TOTAL_ID {
            return MetricResult::unsupported(LimitationCode::NotImplementedOnThisPlatform);
        }

        let content = match self.cached_meminfo.lock() {
            Ok(mut cache) => cache
                .get_or_insert_with(|| {
                    fs::read_to_string(&self.meminfo_path).map_err(|err| match err.kind() {
                        std::io::ErrorKind::PermissionDenied => ReadMeminfoError::Permission,
                        std::io::ErrorKind::NotFound => ReadMeminfoError::NotFound,
                        _ => ReadMeminfoError::Other,
                    })
                })
                .clone(),
            Err(_) => Err(ReadMeminfoError::Other),
        };
        match content {
            Ok(content) => match parse_memory_kib(
                &content,
                if id.as_str() == RAM_AVAILABLE_ID {
                    "MemAvailable:"
                } else {
                    "MemTotal:"
                },
            ) {
                Ok(kib) => {
                    let bytes = match kib.checked_mul(1024) {
                        Some(bytes) => bytes,
                        None => return MetricResult::failed(AcquisitionError::Other),
                    };
                    MetricResult::ok(MetricSample {
                        value: SampleValue::Count(bytes),
                        unit: Unit::Bytes,
                        provenance: Provenance::OperatingSystemReported,
                        timestamp: aer_core::Timestamp::now(),
                        limitations: if id.as_str() == RAM_AVAILABLE_ID {
                            vec![aer_core::LimitationCode::KernelEstimate]
                        } else {
                            Vec::new()
                        },
                    })
                }
                Err(ParseError::NotFound) => {
                    MetricResult::metric_not_available(MetricAvailability::NotExposed)
                }
                Err(ParseError::Malformed) => MetricResult::failed(AcquisitionError::Parsing),
            },
            Err(ReadMeminfoError::Permission) => MetricResult::failed(AcquisitionError::Permission),
            Err(ReadMeminfoError::NotFound) => {
                MetricResult::source_not_ready(SourceAvailability::NotDetected)
            }
            Err(_) => MetricResult::failed(AcquisitionError::Other),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseError {
    /// The `MemAvailable` field was not present in `/proc/meminfo`.
    NotFound,
    /// The `MemAvailable` field was present but could not be parsed as a
    /// positive integer followed by "kB".
    Malformed,
}

/// Parse the `MemAvailable` value from `/proc/meminfo` content.
///
/// The file format is stable: `Key:   value kB`. Returns the value in kibibytes
/// (KiB) as a `u64` so the caller can normalise to bytes.
#[cfg(test)]
fn parse_memavailable_kib(content: &str) -> Result<u64, ParseError> {
    parse_memory_kib(content, "MemAvailable:")
}

fn parse_memory_kib(content: &str, key: &str) -> Result<u64, ParseError> {
    let line = content
        .lines()
        .find(|line| line.starts_with(key))
        .ok_or(ParseError::NotFound)?;

    let prefix_len = key.len();
    let mut parts = line[prefix_len..].split_whitespace();
    let value_part = parts.next().ok_or(ParseError::Malformed)?;
    let unit_part = parts.next().ok_or(ParseError::Malformed)?;

    if unit_part != "kB" || parts.next().is_some() {
        return Err(ParseError::Malformed);
    }

    value_part.parse::<u64>().map_err(|_| ParseError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A fixture file that deletes itself when dropped so tests do not leave
    /// files behind in `.test-fixtures/`.
    struct FixtureFile {
        path: std::path::PathBuf,
    }

    impl FixtureFile {
        fn new(name: &str, content: &str) -> Self {
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("src-tauri crate is inside the project")
                .join(".test-fixtures");
            let _ = fs::create_dir_all(&dir);

            // Clean up stale fixture files left behind by prior processes or
            // crashed runs. Files from the current process are left untouched so
            // parallel tests in this process remain safe.
            let pid = format!("{}", std::process::id());
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("meminfo-") && !name.contains(&pid) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }

            let path = dir.join(format!(
                "meminfo-{}-{}-{}.txt",
                name,
                pid,
                aer_core::Timestamp::now().millis_since_epoch()
            ));
            let mut file = fs::File::create(&path).expect("create fixture file");
            file.write_all(content.as_bytes())
                .expect("write fixture content");
            Self { path }
        }

        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for FixtureFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            // If the fixture directory is now empty, remove it so tests do
            // not leave an unused `.test-fixtures/` directory behind.
            if let Some(parent) = self.path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
    }

    #[test]
    fn parse_extracts_memavailable_in_kib() {
        let content = "MemAvailable:   12345678 kB\n";
        assert_eq!(parse_memavailable_kib(content).unwrap(), 12_345_678);
    }

    #[test]
    fn parse_finds_memavailable_when_not_first_line() {
        let content = "MemTotal:       16000000 kB\nMemAvailable:   8000000 kB\n";
        assert_eq!(parse_memavailable_kib(content).unwrap(), 8_000_000);
    }

    #[test]
    fn parse_returns_not_found_when_field_missing() {
        let content = "MemTotal:       16000000 kB\n";
        assert_eq!(parse_memavailable_kib(content), Err(ParseError::NotFound));
    }

    #[test]
    fn parse_returns_malformed_for_non_numeric_value() {
        let content = "MemAvailable:   broken kB\n";
        assert_eq!(parse_memavailable_kib(content), Err(ParseError::Malformed));
    }

    #[test]
    fn parse_returns_malformed_for_missing_unit() {
        let content = "MemAvailable:   12345678\n";
        assert_eq!(parse_memavailable_kib(content), Err(ParseError::Malformed));
    }

    #[test]
    fn parse_returns_malformed_for_wrong_unit() {
        let content = "MemAvailable:   12345678 MB\n";
        assert_eq!(parse_memavailable_kib(content), Err(ParseError::Malformed));
    }

    #[test]
    fn provider_lists_available_and_total_memory() {
        let provider = LinuxMetricsProvider::new();
        let metrics = provider.list_metrics();
        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].id.as_str(), RAM_AVAILABLE_ID);
        assert_eq!(metrics[0].unit, Unit::Bytes);
        assert_eq!(metrics[1].id.as_str(), RAM_TOTAL_ID);
        assert_eq!(metrics[1].display_name, "Total memory");
    }

    #[test]
    fn provider_with_fixture_path_returns_ready_and_ok_sample() {
        let fixture = FixtureFile::new(
            "ok",
            "MemTotal:       8000000 kB\nMemAvailable:   4000000 kB\n",
        );
        let provider = LinuxMetricsProvider::with_path(fixture.path());

        assert_eq!(provider.source_availability(), SourceAvailability::Ready);

        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));
        assert!(result.is_legal());
        assert_eq!(result.source_availability, Some(SourceAvailability::Ready));
        assert_eq!(
            result.metric_availability,
            Some(MetricAvailability::Available)
        );
        match result.outcome {
            aer_core::Outcome::Ok(sample) => {
                assert_eq!(sample.value, SampleValue::Count(4_096_000_000));
                assert_eq!(sample.unit, Unit::Bytes);
                assert_eq!(sample.provenance, Provenance::OperatingSystemReported);
                assert!(sample
                    .limitations
                    .contains(&aer_core::LimitationCode::KernelEstimate));
            }
            _ => panic!("expected Ok sample"),
        }

        let total = provider.acquire(&MetricId::new(RAM_TOTAL_ID));
        match total.outcome {
            aer_core::Outcome::Ok(sample) => {
                assert_eq!(sample.value, SampleValue::Count(8_192_000_000));
                assert!(sample.limitations.is_empty());
            }
            _ => panic!("expected total-memory sample"),
        }
    }

    #[test]
    fn provider_with_missing_file_returns_not_detected() {
        let provider = LinuxMetricsProvider::with_path("/nonexistent/meminfo");
        assert_eq!(
            provider.source_availability(),
            SourceAvailability::NotDetected
        );
        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));
        assert!(result.is_legal());
        assert_eq!(result.outcome, aer_core::Outcome::NoValue);
        assert_eq!(
            result.source_availability,
            Some(SourceAvailability::NotDetected)
        );
        assert_eq!(result.metric_availability, None);
        let total = provider.acquire(&MetricId::new(RAM_TOTAL_ID));
        assert_eq!(total.outcome, aer_core::Outcome::NoValue);
        assert_eq!(
            total.source_availability,
            Some(SourceAvailability::NotDetected)
        );
    }

    #[test]
    fn provider_with_missing_field_returns_not_exposed() {
        let fixture = FixtureFile::new("missing", "MemTotal:       16000000 kB\n");
        let provider = LinuxMetricsProvider::with_path(fixture.path());
        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));
        assert!(result.is_legal());
        assert_eq!(result.outcome, aer_core::Outcome::NoValue);
        assert_eq!(
            result.metric_availability,
            Some(MetricAvailability::NotExposed)
        );
    }

    #[test]
    fn provider_with_malformed_value_returns_parsing_failure() {
        let fixture = FixtureFile::new("malformed", "MemAvailable:   broken kB\n");
        let provider = LinuxMetricsProvider::with_path(fixture.path());
        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));
        assert!(result.is_legal());
        assert_eq!(
            result.outcome,
            aer_core::Outcome::Failed(AcquisitionError::Parsing)
        );
    }

    #[test]
    fn total_memory_missing_and_malformed_states_are_controlled() {
        let missing = FixtureFile::new("total-missing", "MemAvailable: 100 kB\n");
        let provider = LinuxMetricsProvider::with_path(missing.path());
        assert_eq!(
            provider
                .acquire(&MetricId::new(RAM_TOTAL_ID))
                .metric_availability,
            Some(MetricAvailability::NotExposed)
        );

        let malformed = FixtureFile::new(
            "total-malformed",
            "MemAvailable: 100 kB\nMemTotal: broken kB\n",
        );
        let provider = LinuxMetricsProvider::with_path(malformed.path());
        assert_eq!(
            provider.acquire(&MetricId::new(RAM_TOTAL_ID)).outcome,
            aer_core::Outcome::Failed(AcquisitionError::Parsing)
        );

        for (name, line) in [
            ("total-unit-missing", "MemTotal: 100\n"),
            ("total-unit-wrong", "MemTotal: 100 MB\n"),
            ("total-trailing", "MemTotal: 100 kB unexpected\n"),
        ] {
            let fixture = FixtureFile::new(name, line);
            let provider = LinuxMetricsProvider::with_path(fixture.path());
            assert_eq!(
                provider.acquire(&MetricId::new(RAM_TOTAL_ID)).outcome,
                aer_core::Outcome::Failed(AcquisitionError::Parsing),
                "{name} must fail closed"
            );
        }
    }

    #[test]
    fn total_memory_overflow_is_a_controlled_failure() {
        let fixture = FixtureFile::new("total-overflow", &format!("MemTotal: {} kB\n", u64::MAX));
        let provider = LinuxMetricsProvider::with_path(fixture.path());
        assert_eq!(
            provider.acquire(&MetricId::new(RAM_TOTAL_ID)).outcome,
            aer_core::Outcome::Failed(AcquisitionError::Other)
        );
    }

    #[test]
    fn provider_returns_unsupported_for_unknown_metric() {
        let provider = LinuxMetricsProvider::with_path("/nonexistent/meminfo");
        let result = provider.acquire(&MetricId::new("os.ram.used"));
        assert!(result.is_legal());
        assert_eq!(result.outcome, aer_core::Outcome::NoValue);
        assert!(matches!(
            result.support,
            aer_core::Support::Unsupported { .. }
        ));
    }
}
