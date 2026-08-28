//! Bounded read-only live verification for Milestone 1F (resource context).
//!
//! This integration test exercises the **real** acquisition paths a manual
//! Refresh uses — the `LinuxMetricsProvider` (the single `/proc/meminfo`
//! acquisition behind `current_snapshot`) and the `OllamaAdapter`
//! `loaded_models_status()` (the single `/api/ps` acquisition behind
//! `current_loaded_models`) — **once each**, then calls the **pure**
//! `view::compose_resource_context(&snapshot, &loaded)` (no I/O) and asserts
//! the composed explanation is honest for the live state. It does **not** call
//! the private `build_snapshot()`, does **not** construct a second adapter,
//! and does **not** issue a second `/proc/meminfo` or `/api/ps` read.
//!
//! Read-only and narrow: only `GET /api/ps` and one `/proc/meminfo` read happen.
//! No `/api/tags`, generation, embedding, or model-management endpoints
//! (pull/push/delete/copy) are touched, and no Ollama configuration is changed
//! or service started. No model is loaded, unloaded, prompted, or preloaded by
//! this test; it only observes whatever Ollama currently reports as loaded.
//!
//! The test is Linux-gated because the available-memory side reads
//! `/proc/meminfo` (the 1B `LinuxMetricsProvider` path). It is ignored by
//! default because it depends on the live machine state and is not
//! deterministic across machines. Run it explicitly with
//! `cargo test --test resource_context_live -- --ignored --nocapture`.
//!
//! ## Ignore vs. explicit-fail (owner-confirmed 1D/1E pattern)
//!
//! Ordinary `cargo test` ignores this test (`#[ignore]`), so the deterministic
//! suite never depends on `/proc/meminfo` holding any value or on a local
//! Ollama running. When **explicitly invoked** (`--ignored`), the test **must
//! not report success merely because a source is unavailable.** If it can
//! neither read `/proc/meminfo` **nor** reach the local `/api/ps` endpoint, it
//! **fails** with a clear prerequisite message. It does not start Ollama,
//! change configuration, or silently pass. If **one** source is available, the
//! test still verifies that side and treats the other via its controlled
//! variant.
//!
//! ## Empty-result acceptance
//!
//! A valid empty `/api/ps` result (`200 {"models": []}`) is a **passing**
//! outcome ("no models are currently running" is normal). The test fails only
//! if both required sources are unreachable when explicitly invoked, or if the
//! composed variant materially disagrees with the observed state. No
//! development-machine available-memory value or loaded-model name is
//! hard-coded: every expected value is derived from independent ground-truth
//! reads, so the test is portable.

#![cfg(target_os = "linux")]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aer_core::{explain, OsMetricsProvider, Snapshot, SnapshotEntry, Timestamp};
use aiengineroom_lib::platform::linux::LinuxMetricsProvider;
use aiengineroom_lib::runtime::ollama::{resource_context_text, OllamaAdapter};
use aiengineroom_lib::view::{
    compose_resource_context, loaded_models_view, snapshot_view, LoadedModelStateView, OutcomeView,
    ResourceContextView,
};

/// Controlled display name of the available-memory metric, mirroring
/// `view::AVAILABLE_MEMORY_DISPLAY_NAME` (which is `pub(crate)` and so not
/// reachable from this integration test). A local mirror is safe because the
/// deterministic `view.rs` test pins the constant to the 1B `LinuxMetricsProvider`
/// descriptor's actual `display_name`; this test additionally re-pins the mirror
/// to the live descriptor below.
const AVAILABLE_MEMORY_DISPLAY_NAME: &str = "Available memory";

/// The documented local `/api/ps` endpoint as a `(host, port, path)` triple, kept
/// in sync with `loaded_models::OLLAMA_PS_ENDPOINT`. Used only for the
/// independent `std::net` ground-truth read of the loaded-model side.
const ENDPOINT_HOST: &str = "127.0.0.1";
const ENDPOINT_PORT: u16 = 11_434;
const ENDPOINT_PATH: &str = "/api/ps";

/// Live environment verification for the resource-context composition.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not depend on the live machine state. If **both**
/// `/proc/meminfo` and the local `/api/ps` are unavailable when explicitly run,
/// the test panics with a clear prerequisite message rather than silently
/// passing.
#[test]
#[ignore = "live environment verification: reads real /proc/meminfo and probes a real local Ollama at 127.0.0.1:11434/api/ps; not deterministic across machines"]
fn live_resource_context_matches_observed_state() {
    // --- Independent ground-truth reads (separate code paths, not the real
    //     provider/adapter transports) ------------------------------------
    let ram_independent_kib = independent_read_memavailable_kib();
    let loaded_independent = independent_get_ps();

    // Precondition: if BOTH sources are unavailable, FAIL with a clear
    // prerequisite message. Do not silently pass, and do not start or
    // reconfigure anything. If one source is available, proceed and verify
    // that side (the other is treated via its controlled variant).
    let ram_available = ram_independent_kib.is_some();
    let loaded_reachable = loaded_independent.is_ok();
    if !ram_available && !loaded_reachable {
        panic!(
            "live resource-context verification requires a readable \
             `/proc/meminfo` and a reachable local Ollama at \
             {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH}; both were \
             unavailable (could not read `/proc/meminfo` MemAvailable and \
             nothing accepted the connection at the local endpoint). No \
             configuration was changed and no service was started. No model \
             was loaded or unloaded."
        );
    }
    // An ambiguous independent-read failure (not a clean NotReachable) still
    // has `loaded_reachable == false` above; surface it distinctly so the
    // failure is legible rather than a silent variant-2 pass.
    if let Err(IndependentError::Other(msg)) = &loaded_independent {
        panic!(
            "live resource-context verification could not complete: \
             independent read of {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH} \
             failed ({msg}). No configuration was changed and no service was \
             started."
        );
    }

    // --- Real acquisition paths (the paths a manual Refresh uses) ---------
    // RAM side: the single `/proc/meminfo` acquisition via the 1B provider,
    // exactly as `current_snapshot()` does on Linux (but without calling the
    // private `build_snapshot()`). One read.
    let provider = LinuxMetricsProvider::new();
    let snapshot = build_live_snapshot(&provider);
    let snapshot_view = snapshot_view(&snapshot);

    // Loaded side: the single `/api/ps` acquisition via the 1E adapter. One
    // read. Read-only — no load/unload/generate/pull/delete/keep-alive change.
    let adapter = OllamaAdapter::new();
    let set = tauri::async_runtime::block_on(adapter.loaded_models_status());
    let loaded_view = loaded_models_view(&set);

    // Re-pin the local display-name mirror to the live 1B descriptor so the
    // stringly match used to locate the available-memory entry stays audited
    // against the real provider (belt-and-suspenders alongside the
    // deterministic `view.rs` pin).
    let descriptors = provider.list_metrics();
    assert!(
        !descriptors.is_empty(),
        "Linux provider lists os.ram.available"
    );
    assert_eq!(
        descriptors[0].display_name, AVAILABLE_MEMORY_DISPLAY_NAME,
        "local mirror matches the live 1B descriptor display name"
    );

    // --- Derive the two booleans from the views (what compose uses) -------
    let ram_has_value = snapshot_view.entries.iter().any(|e| {
        e.display_name == AVAILABLE_MEMORY_DISPLAY_NAME
            && matches!(e.outcome, OutcomeView::Ok { .. })
    });
    let loaded_is_available = matches!(loaded_view.state, LoadedModelStateView::Available);

    // --- Cross-check the view-derived booleans against independent truth --
    // RAM: the view says available iff independent `/proc/meminfo` MemAvailable
    // parses to a positive value. This proves the boolean compose consumes is
    // honest, not hard-coded.
    assert_eq!(
        ram_has_value, ram_available,
        "view-derived ram_has_value must match independent `/proc/meminfo` \
         readability"
    );
    // If RAM is available, the projected value must equal the independent
    // MemAvailable (KiB → bytes), proving no hard-coded machine value.
    if ram_has_value {
        let entry = snapshot_view
            .entries
            .iter()
            .find(|e| e.display_name == AVAILABLE_MEMORY_DISPLAY_NAME)
            .expect("available-memory entry present when ram_has_value");
        let expected_bytes = ram_independent_kib.unwrap() as f64 * 1024.0;
        match &entry.outcome {
            OutcomeView::Ok { value, .. } => assert!(
                (value - expected_bytes).abs() < f64::EPSILON,
                "available-memory value ({value} B) must match independent \
                 `/proc/meminfo` MemAvailable ({expected_bytes} B)"
            ),
            other => panic!("expected Ok available-memory outcome, got {other:?}"),
        }
    }

    // Loaded: the view state must match the independent `/api/ps`
    // classification. A valid empty list is Available (normal), not an error.
    match &loaded_independent {
        Ok(read) => match classify_independent(read) {
            IndependentTopLevel::Available { entries, malformed } => {
                assert_eq!(
                    loaded_view.state,
                    LoadedModelStateView::Available,
                    "adapter must report Available when /api/ps returns 200 with \
                     a usable models array (a valid empty list is Available)"
                );
                assert_eq!(
                    loaded_view.models.len(),
                    entries.len(),
                    "adapter loaded-model count must match the independent \
                     valid-entry count"
                );
                assert_eq!(
                    loaded_view.skipped, malformed,
                    "adapter skipped count must match the independent \
                     malformed-entry count"
                );
                // Cross-check each entry's carried fields, in order, without
                // hard-coding any model name.
                for (v, ind) in loaded_view.models.iter().zip(entries.iter()) {
                    assert!(
                        !v.display_name.trim().is_empty(),
                        "adapter display name must be non-empty"
                    );
                    assert_eq!(v.display_name, ind.display_name);
                    assert_eq!(v.reported_size_bytes, ind.reported_size_bytes);
                    assert_eq!(v.reported_vram_bytes, ind.reported_vram_bytes);
                    assert_eq!(v.reported_context_length, ind.reported_context_length);
                    assert_eq!(v.expires_at, ind.expires_at);
                }
            }
            IndependentTopLevel::RespondedUnexpectedly => {
                assert_eq!(
                    loaded_view.state,
                    LoadedModelStateView::RespondedUnexpectedly,
                    "adapter must report RespondedUnexpectedly when /api/ps is \
                     reachable but the loaded-models body is not usable"
                );
                assert!(
                    loaded_view.models.is_empty(),
                    "no models are parsed for an unexpected response"
                );
                assert_eq!(
                    loaded_view.skipped, 0,
                    "skipped is zero when the top-level body is not usable"
                );
            }
        },
        Err(IndependentError::NotReachable) => {
            // Nothing answered: the adapter must not claim Available. The
            // precise NotDetected-vs-Unreachable split depends on the transport
            // kind (refused → NotDetected, timeout → Unreachable); both are
            // valid here. The variant is driven by `loaded_is_available`,
            // which is false for both, so the compose is honest either way.
            assert_ne!(
                loaded_view.state,
                LoadedModelStateView::Available,
                "adapter must not report Available when /api/ps is unreachable"
            );
            assert!(
                loaded_view.models.is_empty(),
                "no models when /api/ps is unreachable"
            );
        }
        // Other-variant unreachable is already panicked out above as a
        // legibility guard; this arm is unreachable but kept for exhaustiveness.
        Err(IndependentError::Other(_)) => {
            unreachable!("ambiguous independent read failure is panicked out above")
        }
    }

    // --- Pure compose + variant correctness -------------------------------
    // The compose step takes only the two already-built views and performs no
    // I/O. Clone the inputs first so we can prove afterwards they were not
    // mutated (no value altered, summed, or subtracted by compose).
    let snap_before = snapshot_view.clone();
    let loaded_before = loaded_view.clone();
    let rc: ResourceContextView = compose_resource_context(&snapshot_view, &loaded_view);

    // The composed explanation must be exactly the variant matching the
    // observed (ram_has_value, loaded_is_available).
    let expected = resource_context_text(ram_has_value, loaded_is_available);
    assert_eq!(
        rc.interpretation, expected.interpretation,
        "composed interpretation must match the controlled variant for \
         (ram_has_value={ram_has_value}, loaded_is_available={loaded_is_available})"
    );
    assert_eq!(
        rc.why_it_matters, expected.why_it_matters,
        "composed why_it_matters must match the controlled variant for \
         (ram_has_value={ram_has_value}, loaded_is_available={loaded_is_available})"
    );

    // --- Honesty ----------------------------------------------------------
    let combined = format!("{} {}", rc.interpretation, rc.why_it_matters);
    assert_no_forbidden_claim(&combined);
    assert_no_raw_leakage(&combined);

    // Where both sources are available (variant 1), the explanation must name
    // both sources, state the non-attribution, and carry the approved
    // qualitative context-length line.
    if ram_has_value && loaded_is_available {
        let lower = combined.to_lowercase();
        assert!(lower.contains("linux"), "variant 1 names the OS source");
        assert!(
            lower.contains("ollama"),
            "variant 1 names the runtime source"
        );
        assert!(
            lower.contains("not be added to or subtracted from"),
            "variant 1 states the non-attribution"
        );
        assert!(
            rc.why_it_matters
                .contains("A larger configured context can require more memory."),
            "variant 1 carries the approved qualitative context-length line"
        );
    } else {
        // The context-length line appears in variant 1 only.
        assert!(
            !rc.why_it_matters
                .contains("A larger configured context can require more memory."),
            "context-length line appears only when both sources are available"
        );
    }

    // --- Composition does not change its inputs ----------------------------
    assert_eq!(
        snapshot_view, snap_before,
        "snapshot view unchanged after the pure compose"
    );
    assert_eq!(
        loaded_view, loaded_before,
        "loaded view unchanged after the pure compose"
    );

    // --- No second acquisition / no model action (structural invariant) ---
    // `compose_resource_context` takes only `&SnapshotView` and
    // `&LoadedModelSetView` and returns `ResourceContextView`: it has no
    // provider/adapter/filesystem/network argument, so it physically cannot
    // read `/proc/meminfo` or call `/api/ps`. Purity is also demonstrated by
    // calling it a second time and observing the same output with no side
    // effect (the inputs are unchanged above).
    let rc_again = compose_resource_context(&snapshot_view, &loaded_view);
    assert_eq!(
        rc, rc_again,
        "compose is pure: same inputs yield the same output (no side-effecting \
         I/O that could change the result)"
    );
    // Only `current_snapshot` (the provider, once) + `current_loaded_models`
    // (the adapter, once) + the pure compose were invoked. No load, unload,
    // generate, prompt, keep-alive change, pull, delete, `/api/tags`, or
    // benchmark was issued; no second `/proc/meminfo` or `/api/ps` read was
    // issued by the compose step. This is a structural guarantee of the
    // compose signature (no I/O argument) and is recorded as an invariant of
    // this milestone, mirroring the 1E live test's structural guarantee note.
}

/// Build a real domain snapshot from the live `LinuxMetricsProvider`, exactly
/// mirroring the Linux branch of `commands::build_snapshot()` — the single
/// `/proc/meminfo` acquisition path `current_snapshot()` uses — without calling
/// the private `build_snapshot()`. One `/proc/meminfo` read per descriptor.
fn build_live_snapshot(provider: &LinuxMetricsProvider) -> Snapshot {
    let descriptors = provider.list_metrics();
    let entries: Vec<SnapshotEntry> = descriptors
        .into_iter()
        .map(|descriptor| {
            let result = provider.acquire(&descriptor.id);
            let explanation = explain(&descriptor, &result);
            SnapshotEntry {
                descriptor,
                result,
                explanation,
            }
        })
        .collect();
    Snapshot {
        at: Timestamp::now(),
        entries,
    }
}

/// Independent ground-truth read of `/proc/meminfo` `MemAvailable` via plain
/// `std::fs` (not the provider), returning the value in KiB. `None` when the
/// file is unreadable or the field is absent/unparseable/non-positive — i.e.
/// the OS side is not available. Mirrors `linux_ram_live`'s
/// `extract_meminfo_field_kib` but never panics (returns `None` on any failure).
fn independent_read_memavailable_kib() -> Option<u64> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let line = content
        .lines()
        .find(|line| line.starts_with("MemAvailable:"))?;
    let mut parts = line["MemAvailable:".len()..].split_whitespace();
    let value = parts.next()?;
    let unit = parts.next()?;
    if unit != "kB" {
        return None;
    }
    let kib: u64 = value.parse().ok()?;
    if kib == 0 {
        return None;
    }
    Some(kib)
}

// --- Independent `/api/ps` ground-truth read (mirrors the 1E live test) ------
//
// A separate `std::net` code path (not the adapter's `reqwest` transport) so
// the adapter's loaded-model result is checked against independent truth. The
// per-entry parsing mirrors the adapter's `non_empty_str` identity, `int_to_u64`
// integer, and verbatim-string `expires_at` rules exactly.

struct IndependentRead {
    status: u16,
    body: String,
}

#[derive(Debug)]
enum IndependentError {
    /// Nothing accepted the connection at the local endpoint.
    NotReachable,
    /// Some other I/O or parsing failure.
    Other(String),
}

struct IndependentEntry {
    display_name: String,
    reported_size_bytes: Option<u64>,
    reported_vram_bytes: Option<u64>,
    reported_context_length: Option<u64>,
    expires_at: Option<String>,
}

/// The independent classification of the top-level `/api/ps` response, mirroring
/// the adapter's `interpret_loaded_models` Responded branch: a 200 with a
/// usable `models` array is `Available` (valid entries + malformed count);
/// anything else the endpoint returned is `RespondedUnexpectedly`.
enum IndependentTopLevel {
    Available {
        entries: Vec<IndependentEntry>,
        malformed: usize,
    },
    RespondedUnexpectedly,
}

/// Independent `std::net` GET of `/api/ps`. Returns the HTTP status and body. A
/// connection that nothing accepts is `NotReachable`; other failures are
/// `Other`.
fn independent_get_ps() -> Result<IndependentRead, IndependentError> {
    let addr = format!("{ENDPOINT_HOST}:{ENDPOINT_PORT}")
        .parse()
        .map_err(|e| IndependentError::Other(format!("address parse: {e}")))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .map_err(|e| classify_connect_error(&e))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| IndependentError::Other(format!("set_read_timeout: {e}")))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| IndependentError::Other(format!("set_write_timeout: {e}")))?;

    let request = format!(
        "GET {ENDPOINT_PATH} HTTP/1.0\r\nHost: {ENDPOINT_HOST}:{ENDPOINT_PORT}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| IndependentError::Other(format!("write: {e}")))?;

    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|e| IndependentError::Other(format!("read: {e}")))?;
    let raw = String::from_utf8_lossy(&bytes);

    let split = raw
        .find("\r\n\r\n")
        .ok_or_else(|| IndependentError::Other("no header/body separator".to_string()))?;
    let header_block = &raw[..split];
    let body = raw[split + 4..].to_string();

    let status = parse_status_code(header_block)?;
    Ok(IndependentRead { status, body })
}

/// A connection error is `NotReachable` only when nothing accepted the
/// connection (refused / timed out / unreachable). Anything else is `Other`.
fn classify_connect_error(e: &std::io::Error) -> IndependentError {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::ConnectionRefused
        | ErrorKind::TimedOut
        | ErrorKind::AddrNotAvailable
        | ErrorKind::NetworkDown
        | ErrorKind::NotConnected => IndependentError::NotReachable,
        _ => IndependentError::Other(format!("connect: {e}")),
    }
}

/// Parse the HTTP status code from the first response line (`HTTP/1.0 200 OK`).
fn parse_status_code(headers: &str) -> Result<u16, IndependentError> {
    let first_line = headers
        .lines()
        .next()
        .ok_or_else(|| IndependentError::Other("empty header block".to_string()))?;
    let mut parts = first_line.split_whitespace();
    parts.next(); // "HTTP/1.0"
    let code = parts
        .next()
        .ok_or_else(|| IndependentError::Other("missing status code".to_string()))?;
    code.parse::<u16>()
        .map_err(|e| IndependentError::Other(format!("status code parse: {e}")))
}

/// Classify the independent `/api/ps` read into the top-level outcome.
fn classify_independent(read: &IndependentRead) -> IndependentTopLevel {
    if read.status != 200 {
        return IndependentTopLevel::RespondedUnexpectedly;
    }

    let value: serde_json::Value = match serde_json::from_str(&read.body) {
        Ok(v) => v,
        Err(_) => return IndependentTopLevel::RespondedUnexpectedly,
    };

    let models_array = match value.get("models") {
        Some(serde_json::Value::Array(arr)) => arr,
        _ => return IndependentTopLevel::RespondedUnexpectedly,
    };

    let mut entries = Vec::new();
    let mut malformed = 0;
    for entry in models_array {
        match independent_parse_entry(entry) {
            Some(parsed) => entries.push(parsed),
            None => malformed += 1,
        }
    }

    IndependentTopLevel::Available { entries, malformed }
}

/// Parse one `/api/ps` entry the way the adapter does, keeping only the fields
/// cross-checked against the adapter: identity, size, size_vram,
/// context_length, and expires_at.
fn independent_parse_entry(value: &serde_json::Value) -> Option<IndependentEntry> {
    let object = value.as_object()?;
    let identity =
        non_empty_str(object.get("name")).or_else(|| non_empty_str(object.get("model")))?;
    let display_name = identity.to_string();
    let reported_size_bytes = object.get("size").and_then(int_to_u64);
    let reported_vram_bytes = object.get("size_vram").and_then(int_to_u64);
    let reported_context_length = object.get("context_length").and_then(int_to_u64);
    let expires_at = object
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(String::from);
    Some(IndependentEntry {
        display_name,
        reported_size_bytes,
        reported_vram_bytes,
        reported_context_length,
        expires_at,
    })
}

/// A non-empty (non-whitespace) string view of a JSON value, or `None`.
/// Mirrors `loaded_models::non_empty_str` exactly.
fn non_empty_str(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
}

/// Read a JSON number as a non-negative `u64`. Mirrors
/// `loaded_models::int_to_u64` exactly.
fn int_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}

// --- Honesty helpers (mirror the deterministic `view.rs` / `ollama.rs`
//     forbidden-claim checks) ----------------------------------------------

/// Tokenise text into lowercase alphanumeric words so forbidden claims are
/// checked as whole words, not accidental substrings (e.g. "Loaded" must not
/// be flagged as the verb "load").
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Assert the composed explanation makes no forbidden arithmetic/health/fit
/// claim and contains no percentage (plan §9 forbidden list).
fn assert_no_forbidden_claim(text: &str) {
    let lower = text.to_lowercase();
    let tokens = words(text);
    for forbidden in [
        "fit",
        "headroom",
        "health",
        "healthy",
        "low",
        "dangerous",
        "sufficient",
        "insufficient",
        "good",
        "bad",
        "safe",
        "performance",
        "fast",
        "slow",
        "total",
        "recommend",
        "unload",
        "load",
    ] {
        assert!(
            !tokens.iter().any(|t| t.as_str() == forbidden),
            "forbidden word claim {forbidden:?} in composed explanation: {text:?}"
        );
    }
    for forbidden in ["%", "used by ollama", "percent of ram"] {
        assert!(
            !lower.contains(forbidden),
            "forbidden phrase {forbidden:?} in composed explanation: {text:?}"
        );
    }
}

/// Assert the composed explanation leaks no raw transport or endpoint detail.
fn assert_no_raw_leakage(text: &str) {
    let lower = text.to_lowercase();
    for leak in ["digest", "127.0.0.1", "11434", "http", "endpoint", "host"] {
        assert!(
            !lower.contains(leak),
            "raw leakage {leak:?} in composed explanation: {text:?}"
        );
    }
}
