//! Bounded live verification for Milestone 1E (Ollama loaded models / runtime
//! residency).
//!
//! This integration test exercises the real `OllamaAdapter` against the
//! documented local Ollama endpoint `http://127.0.0.1:11434/api/ps`. It is NOT
//! gated to an OS (the loaded-models probe is cross-platform: a local HTTP
//! GET). It is intentionally read-only and narrow: only `GET /api/ps` is
//! performed — no `/api/tags`, generation, embedding, or model-management
//! endpoints (pull/push/delete/copy) are touched, and no Ollama configuration
//! is changed or service started. No model is loaded, unloaded, prompted, or
//! preloaded by this test; it only observes whatever Ollama currently reports
//! as loaded.
//!
//! The test is ignored by default because it depends on a live local Ollama
//! process and is not deterministic across machines. Run it explicitly with
//! `cargo test --test ollama_loaded_models_live -- --ignored --nocapture`.
//!
//! Like the Milestone 1D live test: if no local Ollama is reachable, this test
//! FAILS (panics) with a clear prerequisite message. It does not silently
//! return success, and it does not start Ollama or alter any configuration.
//! Live loaded-model verification is only meaningful when a local Ollama
//! answers, so an unreachable endpoint is a failed precondition, not a quiet
//! pass. The verification assertions below apply only once a local Ollama
//! answers.
//!
//! The adapter's loaded-model result is checked against an independent
//! `std::net` ground-truth read (a separate code path, not the adapter's own
//! `reqwest` transport) that replicates the adapter's per-entry parsing rules
//! (`non_empty_str` identity, `int_to_u64` for the integer fields, verbatim
//! string `expires_at`). No development-machine model names are hard-coded:
//! every expected value is derived from the independent read, so the test is
//! portable across whatever models a given Ollama happens to have loaded.
//! A valid empty loaded list (`200 {"models": []}`) is a normal state and
//! passes — it is not treated as an error.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aiengineroom_lib::runtime::loaded_models::{LoadedModelSet, LoadedModelState};
use aiengineroom_lib::runtime::ollama::OllamaAdapter;

/// The documented local endpoint, as a `(host, port, path)` triple, kept in
/// sync with `loaded_models::OLLAMA_PS_ENDPOINT`. Used only for the independent
/// `std::net` ground-truth read.
const ENDPOINT_HOST: &str = "127.0.0.1";
const ENDPOINT_PORT: u16 = 11_434;
const ENDPOINT_PATH: &str = "/api/ps";

/// Independent ground-truth reading of the local endpoint via plain `std::net`
/// (no `reqwest`), so the adapter's result is checked against a separate code
/// path, not its own transport.
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

/// One entry parsed from the independent `/api/ps` read, mirroring the
/// adapter's `parse_entry` identity, `int_to_u64`, and verbatim-string
/// `expires_at` rules exactly. Only the fields needed for cross-checking the
/// adapter are kept; `digest`, `details.*`, processor/offload, `capabilities`,
/// and remote/cloud fields are not read here (the adapter's exclusion of them
/// is structural and is not asserted against the independent read).
struct IndependentEntry {
    display_name: String,
    reported_size_bytes: Option<u64>,
    reported_vram_bytes: Option<u64>,
    reported_context_length: Option<u64>,
    expires_at: Option<String>,
}

/// The independent classification of the top-level response, mirroring the
/// adapter's `interpret_loaded_models` Responded branch: a 200 with a usable
/// `models` array is `Available` (with the valid entries and the count of
/// malformed entries); anything else the endpoint returned is
/// `RespondedUnexpectedly`.
enum IndependentTopLevel {
    Available {
        entries: Vec<IndependentEntry>,
        malformed: usize,
    },
    RespondedUnexpectedly,
}

/// Live environment verification for the Ollama loaded models.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not depend on a local Ollama process being running.
/// An unreachable endpoint is a FAILED precondition here: the test panics
/// rather than silently passing.
#[test]
#[ignore = "live environment verification: probes a real local Ollama at 127.0.0.1:11434/api/ps and is not deterministic across machines"]
fn live_ollama_loaded_models_match_independent_read() {
    // Precondition: a local Ollama must be reachable. If it is not, FAIL with a
    // clear prerequisite message — do not silently pass, and do not start or
    // reconfigure anything.
    let independent = match independent_get_ps() {
        Ok(r) => r,
        Err(IndependentError::NotReachable) => {
            panic!(
                "live loaded-model verification requires a reachable local \
                 Ollama at {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH}; none \
                 was reachable (nothing accepted the connection). No \
                 configuration was changed and no service was started. No model \
                 was loaded or unloaded."
            );
        }
        Err(IndependentError::Other(msg)) => {
            panic!(
                "live loaded-model verification could not complete: independent \
                 read of {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH} failed \
                 ({msg}). No configuration was changed and no service was \
                 started."
            );
        }
    };

    let top_level = classify_independent(&independent);

    // Only GET /api/ps is issued by this test (the independent read above) and
    // by the adapter's single loaded-models probe below — no /api/tags,
    // generation, or model-management endpoints. This is a structural guarantee
    // of the adapter's `loaded_models_status()` path (ReqwestRunningProbe) and
    // is not observable here without an HTTP recorder; it is recorded as an
    // invariant of this milestone.
    let adapter = OllamaAdapter::new();
    let set: LoadedModelSet = tauri::async_runtime::block_on(adapter.loaded_models_status());

    match top_level {
        IndependentTopLevel::Available { entries, malformed } => {
            // A 200 with a usable models array must map to Available. A valid
            // empty list (entries empty) is Available with no models — normal.
            assert_eq!(
                set.state,
                LoadedModelState::Available,
                "adapter must report Available when /api/ps returns 200 with a \
                 usable models array (a valid empty list is Available, not an \
                 error)"
            );
            // The valid-entry count and the malformed/skipped count must match
            // the independent read, which applies the same per-entry validity
            // rule (non-empty name, else non-empty model).
            assert_eq!(
                set.models.len(),
                entries.len(),
                "adapter loaded-model count must match the independent \
                 valid-entry count"
            );
            assert_eq!(
                set.skipped, malformed,
                "adapter skipped count must match the independent malformed-entry count"
            );

            // Every adapter entry corresponds, in order, to an independent valid
            // entry. Verify identity and the four carried fields without
            // hard-coding any model names.
            for (adapter_entry, independent_entry) in set.models.iter().zip(entries.iter()) {
                assert!(
                    !adapter_entry.display_name.trim().is_empty(),
                    "adapter display name must be non-empty"
                );
                assert_eq!(
                    adapter_entry.display_name, independent_entry.display_name,
                    "adapter display name must match the independent identity \
                     (non-empty name, else non-empty model), verbatim"
                );
                assert_eq!(
                    adapter_entry.reported_size_bytes, independent_entry.reported_size_bytes,
                    "adapter reported size must match the independent size \
                     (non-negative integer bytes, else None)"
                );
                assert_eq!(
                    adapter_entry.reported_vram_bytes, independent_entry.reported_vram_bytes,
                    "adapter reported VRAM size must match the independent \
                     size_vram (non-negative integer bytes, else None)"
                );
                assert_eq!(
                    adapter_entry.reported_context_length,
                    independent_entry.reported_context_length,
                    "adapter reported context length must match the independent \
                     context_length (non-negative integer count, else None)"
                );
                assert_eq!(
                    adapter_entry.expires_at, independent_entry.expires_at,
                    "adapter expires_at must match the independent expires_at \
                     (verbatim string when present and string-typed, else None)"
                );
            }
        }
        IndependentTopLevel::RespondedUnexpectedly => {
            // A non-200 status, or a 200 with a non-JSON / non-array `models`
            // body, must map to RespondedUnexpectedly (source Ready, no models).
            assert_eq!(
                set.state,
                LoadedModelState::RespondedUnexpectedly,
                "adapter must report RespondedUnexpectedly when /api/ps is \
                 reachable but the loaded-models body is not usable"
            );
            assert!(
                set.models.is_empty(),
                "no models are parsed for an unexpected response"
            );
            assert_eq!(
                set.skipped, 0,
                "skipped is zero when the top-level body is not usable"
            );
        }
    }
}

/// Independent `std::net` GET of `/api/ps`. Returns the HTTP status code and
/// body. A connection that nothing accepts is `NotReachable` (the expected
/// "Ollama not running" case); other failures are `Other`.
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

    // Split headers/body at the first blank line. HTTP/1.0 with
    // Connection: close has no chunked encoding, so the remainder is the body.
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

/// Classify the independent `/api/ps` read into the top-level outcome,
/// mirroring the adapter's `interpret_loaded_models` Responded branch: 200 with
/// a JSON body whose `models` is an array → `Available` (entries parsed with the
/// adapter's per-entry rule, malformed counted); otherwise
/// `RespondedUnexpectedly`.
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
///
/// Identity = first non-empty string of `name`, then `model` (whitespace-only
/// counts as empty; the kept value is verbatim, not trimmed). An entry without
/// a usable identity is malformed (None). `size`, `size_vram`, and
/// `context_length` are non-negative integers (i64 to detect negatives, then
/// u64 for the full unsigned range); absent, non-integer, negative, or float →
/// `None`. `expires_at` is a verbatim string when present and string-typed;
/// absent/non-string → `None`.
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

/// Read a JSON number as a non-negative `u64`. Accepts both integer-encoded
/// and unsigned-overflowing values; rejects negatives, non-numbers, and floats.
/// Mirrors `loaded_models::int_to_u64` exactly.
fn int_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}
