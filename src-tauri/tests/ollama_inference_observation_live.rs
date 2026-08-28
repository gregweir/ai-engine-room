//! Bounded live verification for Milestone 1G (observed inference).
//!
//! This integration test exercises the **real** application-layer path a
//! manual "Run observation" uses — `run_inference_observation_with_adapter`
//! (the per-run authorization gate, the one `GET /api/tags` catalogue identity
//! check, the process-global single-run guard, and the single non-streaming
//! `POST /api/generate` carrying the fixed diagnostic prompt) — **once**, then
//! asserts the resulting observation is honest for the live state. It does not
//! construct a second adapter, does not issue a second generate POST, and does
//! not call the private pure mapping in isolation.
//!
//! Read-only on the catalogue side and one-shot on the generate side: one
//! independent `GET /api/tags` (ground truth, `std::net`), the adapter's own
//! `GET /api/tags`, and exactly one `POST /api/generate`. No embedding, pull,
//! push, delete, copy, or model-management endpoint is touched, and no Ollama
//! configuration is changed or service started. No `keep_alive` is sent; the
//! model's current residency is left untouched. The fixed diagnostic prompt
//! contains no user files, repository content, or machine information.
//!
//! The test is Linux-gated to match the 1B–1F live-test convention and ignored
//! by default because it depends on the live machine state (a running local
//! Ollama and a model the operator selects) and is not deterministic across
//! machines. Run it explicitly with
//! `cargo test --test ollama_inference_observation_live -- --ignored --nocapture`.
//!
//! ## Authorization gate (per-run, not locality)
//!
//! The live run is gated on **two** environment variables the operator sets
//! explicitly:
//!
//! - `AER_1G_LIVE_MODEL`: the model identifier to run the fixed diagnostic
//!   prompt against. Must be present and non-empty. Not auto-picked, not
//!   hard-coded, not discovered by inspecting the catalogue from the test.
//! - `AER_1G_ALLOW_UNDETERMINED_EXECUTION`: must be present and equal to `1`.
//!   This is the operator's per-run acknowledgement of the undetermined
//!   execution boundary — the same acknowledgement the UI's disclosure dialog
//!   collects. It is **not locality proof and not cryptographic**; it
//!   authorizes one synthetic observation only.
//!
//! When **explicitly invoked** (`--ignored`), the test **must not report
//! success merely because a prerequisite is missing.** If either variable is
//! absent (or the acknowledgement is not `1`), or the local Ollama cannot be
//! reached for the independent catalogue read, it **fails** with a clear
//! prerequisite message. It does not set the variables itself, start Ollama,
//! change configuration, or silently pass.
//!
//! ## No generated-output logging, no performance assertions
//!
//! The `response` field Ollama returns is discarded by `interpret_inference`
//! and never crosses the observation boundary; this test never reads, prints,
//! or asserts on generated text. It asserts no performance threshold: the rate
//! is only checked for finiteness and consistency with the §8 formula, never
//! against a minimum or maximum. No model name is hard-coded: the model comes
//! from `AER_1G_LIVE_MODEL`, and the catalogue expectation is derived from the
//! independent `GET /api/tags` ground truth, so the test is portable.

#![cfg(target_os = "linux")]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use aiengineroom_lib::runtime::inference::{
    calc_generation_rate, ollama_inference_text, run_inference_observation_with_adapter,
    InferenceObservationState, InferenceObservationText, DIAGNOSTIC_PROFILE,
    EXECUTION_BOUNDARY_UNDETERMINED,
};
use aiengineroom_lib::runtime::ollama::OllamaAdapter;
use aiengineroom_lib::view::inference_observation_view;

/// The documented local `/api/tags` endpoint as a `(host, port, path)` triple,
/// kept in sync with `inventory::OLLAMA_TAGS_ENDPOINT`. Used only for the
/// independent `std::net` ground-truth read of the catalogue identity side.
const ENDPOINT_HOST: &str = "127.0.0.1";
const ENDPOINT_PORT: u16 = 11_434;
const ENDPOINT_PATH: &str = "/api/tags";

/// Environment variable holding the model identifier to run the fixed
/// diagnostic prompt against. Operator-supplied; not auto-picked or
/// hard-coded.
const ENV_MODEL: &str = "AER_1G_LIVE_MODEL";

/// Environment variable holding the per-run execution-boundary acknowledgement.
/// Must equal `1` to authorize one synthetic observation. Not locality proof.
const ENV_ALLOW: &str = "AER_1G_ALLOW_UNDETERMINED_EXECUTION";

/// Live environment verification for the observed-inference path.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not issue a real inference. If the authorization
/// variables are missing or the local Ollama cannot be reached for the
/// independent catalogue read, the test panics with a clear prerequisite
/// message rather than silently passing.
#[test]
#[ignore = "live environment verification: issues one real POST /api/generate to a local Ollama at 127.0.0.1:11434; requires AER_1G_LIVE_MODEL and AER_1G_ALLOW_UNDETERMINED_EXECUTION=1; not deterministic across machines"]
fn live_observed_inference_is_honest_for_observed_state() {
    // --- Per-run authorization gate (no network, no POST) -----------------
    // The operator must supply both the model and the undetermined-execution
    // acknowledgement. This mirrors the UI disclosure: `true` authorizes one
    // synthetic observation and is NOT locality proof and NOT cryptographic.
    // The test never sets these variables itself.
    let model = std::env::var(ENV_MODEL).unwrap_or_default();
    let allow = std::env::var(ENV_ALLOW).unwrap_or_default();
    if model.trim().is_empty() || allow.trim() != "1" {
        panic!(
            "live observed-inference verification requires both \
             {ENV_MODEL}=<model> and {ENV_ALLOW}=1 to be set by the operator. \
             {ENV_MODEL} selects the model to run one observation against and \
             {ENV_ALLOW}=1 is the per-run acknowledgement of the undetermined \
             execution boundary (not locality proof, not cryptographic). No \
             configuration was changed, no service was started, and no \
             inference was issued."
        );
    }

    // --- Independent catalogue ground-truth read (read-only, std::net) ----
    // A separate code path (not the adapter's reqwest transport) confirms the
    // model is genuinely in the live catalogue. Identity/availability ONLY —
    // NOT a locality determination. If Ollama is not reachable at all, the
    // test cannot verify anything and fails with a prerequisite.
    let model_listed = match &independent_get_tags() {
        Ok(read) => catalogue_lists(read, &model),
        Err(IndependentError::NotReachable) => {
            panic!(
                "live observed-inference verification requires a reachable \
                 local Ollama at {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH}; \
                 nothing accepted the connection. No configuration was changed \
                 and no service was started. No inference was issued."
            );
        }
        Err(IndependentError::Other(msg)) => {
            panic!(
                "live observed-inference verification could not complete: \
                 independent read of {ENDPOINT_HOST}:{ENDPOINT_PORT}{ENDPOINT_PATH} \
                 failed ({msg}). No configuration was changed and no service was \
                 started."
            );
        }
    };

    // --- Real application-layer path (the path a manual Run observation uses)
    // One authorized run: the adapter's own `GET /api/tags` identity check, the
    // single-run guard, and exactly one `POST /api/generate` with the fixed
    // diagnostic prompt. No retry, no warm-up, no keep_alive change. The
    // generated `response` is discarded by `interpret_inference`.
    let adapter = OllamaAdapter::new();
    let obs = tauri::async_runtime::block_on(run_inference_observation_with_adapter(
        &adapter, &model, true,
    ));

    // --- Catalogue identity cross-check -----------------------------------
    // The adapter's identity check must agree with the independent read: if the
    // model is listed, the adapter must not refuse it as ModelUnavailable or
    // NotDetected (it proceeds to the POST); if it is not listed, the adapter
    // must report ModelUnavailable and issue NO POST.
    if model_listed {
        assert_ne!(
            obs.state,
            InferenceObservationState::ModelUnavailable,
            "adapter must not report ModelUnavailable when the independent \
             /api/tags read lists the model"
        );
        assert_ne!(
            obs.state,
            InferenceObservationState::NotDetected,
            "adapter must not report NotDetected when the independent /api/tags \
             read reached Ollama"
        );
    } else {
        assert_eq!(
            obs.state,
            InferenceObservationState::ModelUnavailable,
            "adapter must report ModelUnavailable and issue no POST when the \
             model is not in the independent /api/tags read"
        );
        // No run happened; verify the honest non-run shape and stop.
        assert_eq!(obs.execution_boundary, EXECUTION_BOUNDARY_UNDETERMINED);
        assert_eq!(obs.diagnostic_profile, DIAGNOSTIC_PROFILE);
        assert!(obs.eval_count.is_none());
        assert!(obs.generation_tokens_per_second.is_none());
        assert_honest_text(&ollama_inference_text(&obs));
        return;
    }

    // --- The run happened: assert the honest observed shape ---------------
    // A completed run carries the Ollama-reported counts/durations and the
    // ER-calculated rate; an honest POST failure (Unreachable/TimedOut/
    // RespondedUnexpectedly) carries no metrics. Both are valid live outcomes;
    // the test verifies honesty, not that Ollama is healthy.
    assert_eq!(obs.model.as_deref(), Some(model.as_str()));
    assert_eq!(obs.diagnostic_profile, DIAGNOSTIC_PROFILE);
    assert_eq!(obs.execution_boundary, EXECUTION_BOUNDARY_UNDETERMINED);

    match obs.state {
        InferenceObservationState::Completed => {
            // The rate must equal the §8 formula and be finite when both
            // inputs are present; None otherwise. No performance threshold.
            assert_eq!(
                obs.generation_tokens_per_second,
                calc_generation_rate(obs.eval_count, obs.eval_duration_ns),
                "rate must match eval_count * 1e9 / eval_duration_ns"
            );
            if let Some(rate) = obs.generation_tokens_per_second {
                assert!(rate.is_finite(), "rate must be finite");
            }
        }
        InferenceObservationState::Unreachable
        | InferenceObservationState::TimedOut
        | InferenceObservationState::RespondedUnexpectedly => {
            // An honest single-attempt failure: no metrics, no retry.
            assert!(
                obs.eval_count.is_none(),
                "no eval_count on a non-completed run"
            );
            assert!(
                obs.generation_tokens_per_second.is_none(),
                "no rate on a non-completed run"
            );
        }
        other => panic!(
            "unexpected live observation state {other:?} after an authorized \
             run for a catalogue-listed model"
        ),
    }

    // --- Generated-output containment + controlled-text honesty -----------
    // The observation struct has no `response` field (a structural guarantee
    // at compile time: generated text never crosses this boundary). The
    // controlled text must make no locality/verified/benchmark claim and leak
    // no raw transport detail.
    let text = ollama_inference_text(&obs);
    assert_honest_text(&text);

    // The view DTO carries the same controlled text and the undetermined
    // boundary, and has no field for the generated response.
    let view = inference_observation_view(&obs);
    assert_eq!(view.state, obs.state);
    assert_eq!(view.execution_boundary, EXECUTION_BOUNDARY_UNDETERMINED);
    assert_eq!(view.interpretation, text.interpretation);
    assert_eq!(view.why_it_matters, text.why_it_matters);
    assert_eq!(view.limitations, text.limitations);
}

/// Assert the controlled text for an observation makes no forbidden claim and
/// leaks no raw transport or generated-output detail. Mirrors the deterministic
/// `inference.rs` / `view.rs` forbidden-claim checks.
fn assert_honest_text(text: &InferenceObservationText) {
    for field in [
        text.interpretation.as_str(),
        text.why_it_matters.as_str(),
        text.limitations.as_str(),
    ] {
        let lower = field.to_lowercase();
        // No locality, verification, or install claim.
        assert!(!lower.contains("local"), "no 'local' claim: {field:?}");
        assert!(
            !lower.contains("verified"),
            "no 'verified' claim: {field:?}"
        );
        assert!(!lower.contains("install"), "no 'install' claim: {field:?}");
        // No positive benchmark claim; only the "not a general benchmark"
        // disclaimer is allowed.
        if lower.contains("benchmark") {
            assert!(
                lower.contains("not a general benchmark"),
                "only the disclaimer may mention benchmark: {field:?}"
            );
        }
        // No raw transport or endpoint detail.
        assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
        assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
        assert!(!lower.contains("http"), "no scheme: {field:?}");
        assert!(!lower.contains("/api/"), "no path: {field:?}");
        assert!(!lower.contains("refus"), "no transport word: {field:?}");
        // No generated-output phrase.
        assert!(
            !lower.contains("generated output"),
            "no generated-output phrase: {field:?}"
        );
        assert!(
            !lower.contains("generated text"),
            "no generated-text phrase: {field:?}"
        );
    }
}

// --- Independent `/api/tags` ground-truth read (mirrors the 1D/1F pattern) ---

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

/// Independent `std::net` GET of `/api/tags`. Returns the HTTP status and body.
/// A connection that nothing accepts is `NotReachable`; other failures are
/// `Other`. Read-only.
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

    let split = raw
        .find("\r\n\r\n")
        .ok_or_else(|| IndependentError::Other("no header/body separator".to_string()))?;
    let header_block = &raw[..split];
    let body = raw[split + 4..].to_string();

    let status = parse_status_code(header_block)?;
    Ok(IndependentRead { status, body })
}

/// A connection error is `NotReachable` only when nothing accepted the
/// connection. Anything else is `Other`.
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

/// Whether the independent `/api/tags` read lists `model` by identity. Mirrors
/// the adapter's `parse_entry` identity rule: the `name` field if non-empty,
/// else the `model` field. Identity/availability ONLY — not a locality
/// determination. A non-200 or unparseable body is treated as "not listed" so
/// the adapter's own identity check remains the authority for proceeding.
fn catalogue_lists(read: &IndependentRead, model: &str) -> bool {
    if read.status != 200 {
        return false;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&read.body) else {
        return false;
    };
    let Some(models) = value.get("models").and_then(|v| v.as_array()) else {
        return false;
    };
    models
        .iter()
        .any(|entry| entry_identity(entry) == Some(model))
}

/// The identity of a `/api/tags` entry, mirroring `inventory::parse_entry`:
/// `name` if non-empty, else `model`. `None` when neither is a non-empty
/// string.
fn entry_identity(value: &serde_json::Value) -> Option<&str> {
    let object = value.as_object()?;
    non_empty_str(object.get("name")).or_else(|| non_empty_str(object.get("model")))
}

/// A non-empty (non-whitespace) string view of a JSON value, or `None`.
/// Mirrors `inventory::non_empty_str` exactly.
fn non_empty_str(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
}
