//! Bounded live verification for Milestone 1C (Ollama detect).
//!
//! This integration test exercises the real `OllamaAdapter` against the
//! documented local Ollama endpoint `http://127.0.0.1:11434/api/version`. It is
//! NOT gated to an OS (Ollama detection is cross-platform: a local HTTP probe).
//! It is intentionally read-only and narrow: only `GET /api/version` is
//! performed — no model listing, generation, embedding, or model-management
//! endpoints are touched, and no Ollama configuration is changed or service
//! started.
//!
//! The test is ignored by default because it depends on a live local Ollama
//! process and is not deterministic across machines. Run it explicitly with
//! `cargo test --test ollama_detect_live -- --ignored --nocapture`.
//!
//! If no local Ollama is reachable, the test reports that live verification
//! could not run and returns (it does not start Ollama or alter any
//! configuration). The verification assertions below apply only when a local
//! Ollama answers.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aer_core::{AiRuntimeAdapter, SourceAvailability};
use aiengineroom_lib::runtime::ollama::OllamaAdapter;
use aiengineroom_lib::runtime::status::VersionProvenance;

/// The documented local endpoint, as a `(host, port, path)` triple, kept in
/// sync with `OllamaAdapter`'s constant. Used only for the independent
/// `std::net` ground-truth read.
const ENDPOINT_HOST: &str = "127.0.0.1";
const ENDPOINT_PORT: u16 = 11_434;
const ENDPOINT_PATH: &str = "/api/version";

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

/// Live environment verification for Ollama detect.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not depend on a local Ollama process being running.
#[test]
#[ignore = "live environment verification: probes a real local Ollama at 127.0.0.1:11434 and is not deterministic across machines"]
fn live_ollama_detect_matches_independent_read() {
    let independent = match independent_get_version() {
        Ok(r) => r,
        Err(IndependentError::NotReachable) => {
            eprintln!(
                "live verification could not run: no local Ollama reached at \
                 {ENDPOINT_HOST}:{ENDPOINT_PORT} (nothing accepted the connection). \
                 No configuration was changed and no service was started."
            );
            return;
        }
        Err(IndependentError::Other(msg)) => {
            eprintln!(
                "live verification could not run: independent read of \
                 {ENDPOINT_HOST}:{ENDPOINT_PORT} failed ({msg}). \
                 No configuration was changed and no service was started."
            );
            return;
        }
    };

    // The endpoint responded, so the adapter must report Ready.
    let adapter = OllamaAdapter::new();
    let status = tauri::async_runtime::block_on(adapter.runtime_status());
    assert_eq!(
        status.source_availability,
        SourceAvailability::Ready,
        "adapter must report Ready when the local endpoint responds"
    );

    // detect() must agree with runtime_status() on source availability.
    let via_detect = adapter.detect();
    assert_eq!(
        via_detect, status.source_availability,
        "detect() must agree with runtime_status().source_availability"
    );

    // The version the adapter reports must correspond to the independent read.
    let independent_version = if independent.status == 200 {
        extract_version(&independent.body)
    } else {
        None
    };

    match (independent_version.as_deref(), status.version.as_ref()) {
        (Some(expected), Some(actual)) => {
            assert_eq!(
                actual.value, expected,
                "adapter version must match the independent read, verbatim"
            );
            assert_eq!(
                actual.provenance,
                VersionProvenance::RuntimeReported,
                "version must be marked runtime-reported"
            );
            assert!(!actual.value.trim().is_empty(), "version must be non-empty");
        }
        (None, None) => {
            // The endpoint responded but with no usable version (non-200 or
            // malformed/empty body): the controlled "responded unexpectedly"
            // state — Ready with no version. This is a valid live outcome.
            eprintln!(
                "live endpoint responded (status {}) without a usable version; \
                 adapter correctly reports Ready with no version.",
                independent.status
            );
        }
        (expected, actual) => {
            panic!("version presence mismatch: independent={expected:?}, adapter={actual:?}")
        }
    }
}

/// Independent `std::net` GET of `/api/version`. Returns the HTTP status code
/// and body. A connection that nothing accepts is `NotReachable` (the expected
/// "Ollama not running" case); other failures are `Other`.
fn independent_get_version() -> Result<IndependentRead, IndependentError> {
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

/// Extract a non-empty `version` string from an `/api/version` JSON body,
/// mirroring the adapter's pure extraction (verbatim, no semver parsing).
fn extract_version(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let version = value.get("version")?.as_str()?;
    if version.trim().is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}
