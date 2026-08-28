//! Bounded live verification for Milestone 1D (Ollama model inventory).
//!
//! This integration test exercises the real `OllamaAdapter` against the
//! documented local Ollama endpoint `http://127.0.0.1:11434/api/tags`. It is
//! NOT gated to an OS (the inventory probe is cross-platform: a local HTTP
//! GET). It is intentionally read-only and narrow: only `GET /api/tags` is
//! performed — no `/api/ps`, generation, embedding, or model-management
//! endpoints (pull/push/delete/copy) are touched, and no Ollama configuration
//! is changed or service started.
//!
//! The test is ignored by default because it depends on a live local Ollama
//! process and is not deterministic across machines. Run it explicitly with
//! `cargo test --test ollama_inventory_live -- --ignored --nocapture`.
//!
//! Departure from the Milestone 1C live test: if no local Ollama is reachable,
//! this test FAILS (panics) with a clear prerequisite message. It does not
//! silently return success, and it does not start Ollama or alter any
//! configuration. Live inventory verification is only meaningful when a local
//! Ollama answers, so an unreachable endpoint is a failed precondition, not a
//! quiet pass. The verification assertions below apply only once a local
//! Ollama answers.
//!
//! The adapter's inventory result is checked against an independent
//! `std::net` ground-truth read (a separate code path, not the adapter's own
//! `reqwest` transport) that replicates the adapter's per-entry parsing rules
//! (`non_empty_str` identity, `size_to_u64`). No development-machine model
//! names are hard-coded: every expected value is derived from the independent
//! read, so the test is portable across whatever models a given Ollama has.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aiengineroom_lib::runtime::inventory::{InventoryState, ModelInventory, ModelLocality};
use aiengineroom_lib::runtime::ollama::OllamaAdapter;

/// The documented local endpoint, as a `(host, port, path)` triple, kept in
/// sync with `inventory::OLLAMA_TAGS_ENDPOINT`. Used only for the independent
/// `std::net` ground-truth read.
const ENDPOINT_HOST: &str = "127.0.0.1";
const ENDPOINT_PORT: u16 = 11_434;
const ENDPOINT_PATH: &str = "/api/tags";

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

/// One entry parsed from the independent `/api/tags` read, mirroring the
/// adapter's `parse_entry` identity and `size_to_u64` rules exactly. Only the
/// fields needed for cross-checking the adapter are kept; `details`-derived
/// metadata is not re-derived here (the adapter's extraction of it is
/// structural and not asserted against the independent read).
struct IndependentEntry {
    display_name: String,
    size_bytes: Option<u64>,
}

/// The independent classification of the top-level response, mirroring the
/// adapter's `interpret_inventory` Responded branch: a 200 with a usable
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

/// Live environment verification for the Ollama model inventory.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not depend on a local Ollama process being running.
/// Unlike the 1C detect test, an unreachable endpoint is a FAILED precondition
/// here: the test panics rather than silently returning.
#[test]
#[ignore = "live environment verification: probes a real local Ollama at 127.0.0.1:11434/api/tags and is not deterministic across machines"]
fn live_ollama_inventory_matches_independent_read() {
    // Precondition: a local Ollama must be reachable. If it is not, FAIL with a
    // clear prerequisite message — do not silently pass, and do not start or
    // reconfigure anything.
    let independent = match independent_get_tags() {
        Ok(r) => r,
        Err(IndependentError::NotReachable) => {
            panic!(
                "live inventory verification requires a reachable local Ollama \
                 at {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH}; none was \
                 reachable (nothing accepted the connection). No configuration \
                 was changed and no service was started."
            );
        }
        Err(IndependentError::Other(msg)) => {
            panic!(
                "live inventory verification could not complete: independent \
                 read of {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH} failed \
                 ({msg}). No configuration was changed and no service was \
                 started."
            );
        }
    };

    let top_level = classify_independent(&independent);

    // Only GET /api/tags is issued by this test (the independent read above) and
    // by the adapter's single inventory probe below — no /api/ps, generation,
    // or model-management endpoints. This is a structural guarantee of the
    // adapter's `inventory()` path (ReqwestModelsProbe) and is not observable
    // here without an HTTP recorder; it is recorded as an invariant of this
    // milestone.
    let adapter = OllamaAdapter::new();
    let inventory: ModelInventory = tauri::async_runtime::block_on(adapter.inventory());

    match top_level {
        IndependentTopLevel::Available { entries, malformed } => {
            // A 200 with a usable models array must map to Available.
            assert_eq!(
                inventory.state,
                InventoryState::Available,
                "adapter must report Available when /api/tags returns 200 with a \
                 usable models array"
            );
            // The valid-entry count and the malformed/skipped count must match
            // the independent read, which applies the same per-entry validity
            // rule (non-empty name, else non-empty model).
            assert_eq!(
                inventory.models.len(),
                entries.len(),
                "adapter model count must match the independent valid-entry count"
            );
            assert_eq!(
                inventory.skipped, malformed,
                "adapter skipped count must match the independent malformed-entry count"
            );

            // Every adapter entry corresponds, in order, to an independent valid
            // entry. Verify identity, size, and locality without hard-coding
            // any model names.
            for (adapter_entry, independent_entry) in inventory.models.iter().zip(entries.iter()) {
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
                    adapter_entry.size_bytes, independent_entry.size_bytes,
                    "adapter size must match the independent size (non-negative \
                     integer bytes, else None)"
                );
                assert_eq!(
                    adapter_entry.locality,
                    ModelLocality::Undetermined,
                    "every adapter entry must be locality Undetermined (no \
                     inference from /api/tags)"
                );
            }
        }
        IndependentTopLevel::RespondedUnexpectedly => {
            // A non-200 status, or a 200 with a non-JSON / non-array `models`
            // body, must map to RespondedUnexpectedly (source Ready, no models).
            assert_eq!(
                inventory.state,
                InventoryState::RespondedUnexpectedly,
                "adapter must report RespondedUnexpectedly when /api/tags is \
                 reachable but the catalogue body is not usable"
            );
            assert!(
                inventory.models.is_empty(),
                "no models are parsed for an unexpected response"
            );
            assert_eq!(
                inventory.skipped, 0,
                "skipped is zero when the top-level body is not usable"
            );
        }
    }
}

/// Independent `std::net` GET of `/api/tags`. Returns the HTTP status code and
/// body. A connection that nothing accepts is `NotReachable` (the expected
/// "Ollama not running" case); other failures are `Other`.
fn independent_get_tags() -> Result<IndependentRead, IndependentError> {
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

/// Classify the independent `/api/tags` read into the top-level outcome,
/// mirroring the adapter's `interpret_inventory` Responded branch: 200 with a
/// JSON body whose `models` is an array → `Available` (entries parsed with the
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

/// Parse one `/api/tags` entry the way the adapter does, keeping only the
/// fields cross-checked against the adapter: identity and size.
///
/// Identity = first non-empty string of `name`, then `model` (whitespace-only
/// counts as empty; the kept value is verbatim, not trimmed). An entry without
/// a usable identity is malformed (None). `size` is a non-negative integer
/// (i64 to detect negatives, then u64 for the full unsigned range); absent,
/// non-integer, negative, or float → `None`.
fn independent_parse_entry(value: &serde_json::Value) -> Option<IndependentEntry> {
    let object = value.as_object()?;
    let identity =
        non_empty_str(object.get("name")).or_else(|| non_empty_str(object.get("model")))?;
    let display_name = identity.to_string();
    let size_bytes = object.get("size").and_then(size_to_u64);
    Some(IndependentEntry {
        display_name,
        size_bytes,
    })
}

/// A non-empty (non-whitespace) string view of a JSON value, or `None`.
/// Mirrors `inventory::non_empty_str` exactly.
fn non_empty_str(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
}

/// Read a JSON number as a non-negative `u64`. Accepts both integer-encoded
/// and unsigned-overflowing values; rejects negatives, non-numbers, and floats.
/// Mirrors `inventory::size_to_u64` exactly.
fn size_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}
