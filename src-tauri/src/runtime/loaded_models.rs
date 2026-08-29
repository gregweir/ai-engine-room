//! Milestone 1E loaded models / runtime residency — application layer, not
//! `aer-core`.
//!
//! A detected runtime's *currently loaded/running* models are structured runtime
//! metadata, not a set of numeric metrics. These types live here in `src-tauri`
//! so `aer-core`, `SampleValue`, `MetricSample`, metric `Provenance`, the report
//! boundary, and the `AiRuntimeAdapter` trait stay unchanged. `SourceAvailability`
//! is reused from `aer-core` unchanged (as in 1C/1D). The loaded-model set flows
//! through the concrete `OllamaAdapter::loaded_models_status()` method — exactly
//! as 1C routed the version through `runtime_status()` and 1D routed the
//! catalogue through `inventory()` rather than `list_models()`/
//! `runtime_metrics()` — and `loaded_models()` stays empty for 1E.
//!
//! `RunningOutcome` is the controlled result of one `GET /api/ps` probe, free of
//! any `reqwest` type. The production transport (`ReqwestRunningProbe`) and any
//! test transport (`FakeRunningProbe`) both reduce to this enum, so the
//! loaded-model semantics in [`interpret_loaded_models`] — the deterministic
//! heart of 1E — are testable with fixtures and no socket, no `reqwest`, and no
//! live Ollama.
//!
//! Only the documented local `http://127.0.0.1:11434/api/ps` endpoint is
//! requested. Only the developer-verified documented `/api/ps` fields are parsed
//! (`name`, `model` as a parsing fallback, `size`, `size_vram`,
//! `context_length`, `expires_at`); any extra field a live runtime returns
//! beyond that contract is ignored by the parser and never copied into fixtures.
//! No `digest`, `details.*`, endpoint, host, port, processor/offload, or
//! remote/cloud field crosses this boundary. `expires_at` is carried verbatim
//! for forward use and is **not** displayed in 1E's initial UI.
//!
//! 1D and 1E answer different questions and are kept distinct: 1D (`/api/tags`)
//! is "what appears in the catalogue"; 1E (`/api/ps`) is "what is currently
//! running/loaded". Presence in the catalogue does not imply loaded, and 1E
//! makes no locality claim for any loaded entry (no identity join to the 1D
//! catalogue, no inference from name suffixes such as `:cloud`).

use std::future::Future;
use std::time::Duration;

use aer_core::SourceAvailability;

use crate::runtime::status::RuntimeType;

/// Ollama's documented local loaded-models endpoint ("List running models"),
/// used explicitly as the loopback IP to avoid `localhost` IPv6/IPv4 resolution
/// ambiguity (mirrors 1C's version endpoint and 1D's tags endpoint). No
/// configurable endpoint in 1E; no hostname, LAN address, or model name leaks
/// through the view boundary.
pub const OLLAMA_PS_ENDPOINT: &str = "http://127.0.0.1:11434/api/ps";

/// Connect deadline for the local loaded-models probe: 500 ms (mirrors 1C/1D).
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Overall request deadline (connect + read): 2 seconds (mirrors 1C/1D). A
/// hanging responder cannot stall the UI beyond this.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// One model Ollama reports as currently loaded/running. Runtime-neutral; no
/// Ollama JSON shape leaks here. Holds only the user-facing/forward-use fields;
/// `digest`, `details.*`, endpoint, host, and transport details are deliberately
/// excluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedModelSummary {
    /// Ollama `name`, verbatim (falling back to `model` only during parsing when
    /// `name` is absent). How the user refers to the loaded model.
    pub display_name: String,
    /// Ollama `size` — the loaded size Ollama reports for this running instance,
    /// in bytes. Labelled conservatively as "Ollama-reported loaded size"; never
    /// claimed to be exact RAM consumption, exact total memory footprint, or
    /// disk usage. `None` when absent/invalid (never `0` for "no size").
    pub reported_size_bytes: Option<u64>,
    /// Ollama `size_vram` — the VRAM size Ollama reports for this running
    /// instance, in bytes. Labelled conservatively as "Ollama-reported VRAM
    /// size"; never claimed to be independently-measured physical VRAM
    /// consumption, exclusive GPU residency, or "GPU memory used." `None` when
    /// absent/invalid.
    pub reported_vram_bytes: Option<u64>,
    /// Ollama `context_length` — the context length Ollama reports for this
    /// running instance, as a count. Labelled conservatively as
    /// "Ollama-reported context length"; never claimed to be a theoretical
    /// maximum context window or a performance figure. `None` when
    /// absent/invalid.
    pub reported_context_length: Option<u64>,
    /// Ollama `expires_at` — the verbatim ISO-8601 string Ollama reports. Carried
    /// for forward use; **not displayed** in 1E's initial UI. Residency can be
    /// extended by runtime activity, so this is a soft signal, not a guarantee;
    /// no countdown timer is implied. `None` when absent/non-string.
    pub expires_at: Option<String>,
}

/// The state of a loaded-model fetch, mirroring the 1C/1D four-state model for
/// `/api/ps`. A valid empty loaded-model list is [`LoadedModelState::Available`]
/// with `models` empty — it is **not** an error ("No models are currently
/// running in Ollama" is normal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadedModelState {
    /// Reached and the top-level JSON was parseable with a `models` array. The
    /// list may be empty (nothing is loaded right now).
    Available,
    /// Reached (the source answered) but the response was non-200 or the
    /// top-level JSON was missing/malformed (no usable `models` array).
    RespondedUnexpectedly,
    /// Nothing answered at the local endpoint (connection refused).
    NotDetected,
    /// Timeout or other transport failure.
    Unreachable,
}

/// The full loaded-model result for a runtime at one fetch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedModelSet {
    pub runtime_type: RuntimeType,
    pub state: LoadedModelState,
    /// Reused from `aer-core` unchanged (as in 1C/1D): reflects whether the
    /// source answered, independent of the per-entry validity.
    pub source_availability: SourceAvailability,
    /// Parsed loaded-model entries. Empty unless `state == Available`.
    pub models: Vec<LoadedModelSummary>,
    /// Entries dropped because they had no usable identity (no non-empty
    /// `name`/`model`). A malformed individual entry does not fail the whole
    /// list; its valid siblings are kept.
    pub skipped: usize,
}

/// The controlled result of one local `GET /api/ps` probe, free of transport
/// types. A returned HTTP response reduces to `Responded { status, body }`;
/// transport failures reduce to the honest categories below. No raw error, URL,
/// header, or hostname crosses this boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunningOutcome {
    /// An HTTP response arrived. `status` is the HTTP status code; `body` is the
    /// raw response body text (parsed later, in the pure layer).
    Responded { status: u16, body: String },
    /// Nothing was listening at the local endpoint (connection refused).
    ConnectionRefused,
    /// The connect or read attempt exceeded its deadline.
    Timeout,
    /// Unexpected I/O not expected on localhost plain HTTP.
    Other,
}

/// A transport that performs one local loaded-models probe and reduces the
/// result to a controlled [`RunningOutcome`]. `Send + Sync` so an adapter can be
/// shared across the Tauri async boundary. Implementations keep all transport
/// types (e.g. `reqwest`) confined to themselves.
pub trait RunningProbe: Send + Sync {
    /// Perform one fetch and return the controlled outcome.
    fn fetch(&self) -> impl Future<Output = RunningOutcome> + Send;
}

/// Map one running-probe outcome to a controlled loaded-model set. Pure: no I/O,
/// no transport types. `runtime_type` is carried through so the pure layer stays
/// runtime-neutral; the caller (the Ollama adapter) supplies it.
///
/// Mapping (design §8):
///
/// | `RunningOutcome`                                     | `LoadedModelState`      | `source_availability` | `models`            |
/// |------------------------------------------------------|-------------------------|------------------------|---------------------|
/// | `Responded { 200, valid JSON with models array }`    | `Available`             | `Ready`                | parsed (may be empty) |
/// | `Responded { 200, missing/non-array models }`        | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `Responded { 200, malformed JSON }`                  | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `Responded { non-200, .. }`                          | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `ConnectionRefused`                                  | `NotDetected`           | `NotDetected`          | empty               |
/// | `Timeout` / `Other`                                  | `Unreachable`           | `Unreachable`          | empty               |
///
/// Per-entry parsing: an entry is valid if it has a non-empty `name` (falling
/// back to `model`). `size`, `size_vram`, and `context_length` are read as
/// non-negative integers; absent, non-numeric, negative, or float → `None`
/// (entry still valid). `expires_at` is read as a verbatim string when present
/// and string-typed; absent/non-string → `None`. An entry with no usable
/// identity is skipped and counted in `skipped`. Locality is never claimed for
/// any loaded entry; no inference from name suffixes (e.g. `:cloud`). Extra
/// fields beyond the documented contract are ignored.
pub fn interpret_loaded_models(
    outcome: RunningOutcome,
    runtime_type: RuntimeType,
) -> LoadedModelSet {
    match outcome {
        RunningOutcome::Responded { status, body } => {
            // The endpoint answered, so the source is `Ready` regardless of
            // whether the loaded-models body was usable.
            if status != 200 {
                return LoadedModelSet {
                    runtime_type,
                    state: LoadedModelState::RespondedUnexpectedly,
                    source_availability: SourceAvailability::Ready,
                    models: Vec::new(),
                    skipped: 0,
                };
            }

            let value: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => {
                    return LoadedModelSet {
                        runtime_type,
                        state: LoadedModelState::RespondedUnexpectedly,
                        source_availability: SourceAvailability::Ready,
                        models: Vec::new(),
                        skipped: 0,
                    };
                }
            };

            let models_array = match value.get("models") {
                Some(serde_json::Value::Array(arr)) => arr,
                _ => {
                    return LoadedModelSet {
                        runtime_type,
                        state: LoadedModelState::RespondedUnexpectedly,
                        source_availability: SourceAvailability::Ready,
                        models: Vec::new(),
                        skipped: 0,
                    };
                }
            };

            let mut models = Vec::new();
            let mut skipped = 0;
            for entry in models_array {
                match parse_entry(entry) {
                    Some(summary) => models.push(summary),
                    None => skipped += 1,
                }
            }

            LoadedModelSet {
                runtime_type,
                state: LoadedModelState::Available,
                source_availability: SourceAvailability::Ready,
                models,
                skipped,
            }
        }
        RunningOutcome::ConnectionRefused => LoadedModelSet {
            runtime_type,
            state: LoadedModelState::NotDetected,
            source_availability: SourceAvailability::NotDetected,
            models: Vec::new(),
            skipped: 0,
        },
        // Timeout and other transport failures are grouped: the connect-vs-read
        // distinction is not surfaced in 1E, mirroring 1C/1D.
        RunningOutcome::Timeout | RunningOutcome::Other => LoadedModelSet {
            runtime_type,
            state: LoadedModelState::Unreachable,
            source_availability: SourceAvailability::Unreachable,
            models: Vec::new(),
            skipped: 0,
        },
    }
}

/// Parse one `/api/ps` entry (`serde_json::Value`) into a
/// [`LoadedModelSummary`], or `None` if the entry has no usable identity.
///
/// Only the developer-verified documented fields are read; anything else a live
/// runtime returns is ignored (including `digest`, `details.*`, processor/
/// offload, `capabilities`, and remote/cloud fields). `size`, `size_vram`, and
/// `context_length` are read as non-negative integers (i64 to detect negatives,
/// u64 for the full unsigned range); absent, non-numeric, negative, or float →
/// `None`. `expires_at` is read as a verbatim string when present and
/// string-typed; absent/non-string → `None`. No locality claim is made; no
/// inference from name suffixes.
fn parse_entry(value: &serde_json::Value) -> Option<LoadedModelSummary> {
    let object = value.as_object()?;

    // Identity: first non-empty string of `name`, then `model` (parsing
    // fallback only). Whitespace-only counts as empty; the kept value is
    // verbatim (not trimmed).
    let identity =
        non_empty_str(object.get("name")).or_else(|| non_empty_str(object.get("model")))?;
    let display_name = identity.to_string();

    // `size` / `size_vram` / `context_length`: documented integer fields. Accept
    // i64 (to detect negatives) and u64 (to cover the full unsigned range);
    // anything else (absent, non-numeric, negative, float) → None.
    let reported_size_bytes = object.get("size").and_then(int_to_u64);
    let reported_vram_bytes = object.get("size_vram").and_then(int_to_u64);
    let reported_context_length = object.get("context_length").and_then(int_to_u64);

    // `expires_at`: verbatim string when present and string-typed; else None.
    let expires_at = object
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(LoadedModelSummary {
        display_name,
        reported_size_bytes,
        reported_vram_bytes,
        reported_context_length,
        expires_at,
    })
}

/// A non-empty (non-whitespace) string view of a JSON value, or `None`.
fn non_empty_str(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
}

/// Read a JSON number as a non-negative `u64`. Accepts both integer-encoded and
/// unsigned-overflowing values; rejects negatives, non-numbers, and floats (the
/// documented fields are integers).
fn int_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}

/// Production transport: a `reqwest::Client` with bounded timeouts, used only
/// for the documented local `GET /api/ps`. All `reqwest` types stay in this
/// struct and its `RunningProbe` impl. Mirrors 1C's `ReqwestProbe` and 1D's
/// `ReqwestModelsProbe` timeouts and honest error classification.
pub struct ReqwestRunningProbe {
    client: reqwest::Client,
}

impl ReqwestRunningProbe {
    /// Build a client with the bounded 1E/1C/1D timeouts. Reused across probes.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest client with bounded timeouts builds with default features");
        Self { client }
    }
}

impl Default for ReqwestRunningProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl RunningProbe for ReqwestRunningProbe {
    fn fetch(&self) -> impl Future<Output = RunningOutcome> + Send {
        let client = self.client.clone();
        async move {
            match client.get(OLLAMA_PS_ENDPOINT).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    match response.text().await {
                        Ok(body) => RunningOutcome::Responded { status, body },
                        // Body read failed: classify the transport error honestly
                        // rather than fabricating a response. No raw error
                        // crosses the boundary.
                        Err(error) => classify_reqwest_error(error),
                    }
                }
                Err(error) => classify_reqwest_error(error),
            }
        }
    }
}

/// Reduce a `reqwest` error to a controlled [`RunningOutcome`]. No raw error text
/// leaves this function. Timeout is checked first, and only an underlying
/// `std::io::ErrorKind::ConnectionRefused` maps to `ConnectionRefused`
/// (NotDetected). Everything else is `Other` (Unreachable) — conservative, no
/// fabricated certainty. Mirrors 1C/1D's `classify_reqwest_error`.
fn classify_reqwest_error(error: reqwest::Error) -> RunningOutcome {
    if error.is_timeout() {
        return RunningOutcome::Timeout;
    }

    if let Some(io_error) = find_source_io_error(&error) {
        if io_error.kind() == std::io::ErrorKind::ConnectionRefused {
            return RunningOutcome::ConnectionRefused;
        }
    }

    RunningOutcome::Other
}

fn find_source_io_error<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a std::io::Error> {
    let mut current = Some(error);

    while let Some(source) = current {
        if let Some(io_error) = source.downcast_ref::<std::io::Error>() {
            return Some(io_error);
        }
        current = source.source();
    }

    None
}

#[cfg(test)]
mod tests {
    //! Deterministic coverage of `interpret_loaded_models` over every design-§8
    //! row, plus the per-entry parsing rules, honesty invariants, and one
    //! ephemeral-loopback transport check. No live Ollama.

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;
    use crate::runtime::status::RuntimeType;
    use aer_core::SourceAvailability;

    const OLLAMA: RuntimeType = RuntimeType::Ollama;

    fn responded(status: u16, body: &str) -> RunningOutcome {
        RunningOutcome::Responded {
            status,
            body: body.to_string(),
        }
    }

    fn interpret(outcome: RunningOutcome) -> LoadedModelSet {
        interpret_loaded_models(outcome, OLLAMA)
    }

    fn redirected_get(client: reqwest::Client) -> (u16, String, String, bool, bool) {
        let target = TcpListener::bind(("127.0.0.1", 0)).expect("bind redirect target");
        target
            .set_nonblocking(true)
            .expect("make redirect target nonblocking");
        let location = format!("http://{}/target", target.local_addr().unwrap());

        let source = TcpListener::bind(("127.0.0.1", 0)).expect("bind redirect source");
        let source_url = format!("http://{}/source", source.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = source.accept().expect("accept source request");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("bound source read");
            let mut bytes = [0_u8; 2048];
            let count = stream.read(&mut bytes).expect("read source request");
            let request = String::from_utf8_lossy(&bytes[..count]).into_owned();
            let response = format!(
                "HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 8\r\nConnection: close\r\n\r\nredirect"
            );
            stream
                .write_all(response.as_bytes())
                .expect("write redirect response");
            drop(stream);
            source
                .set_nonblocking(true)
                .expect("make redirect source nonblocking");
            thread::sleep(Duration::from_millis(25));
            let extra_source_request = source.accept().is_ok();
            (request, extra_source_request)
        });

        let response =
            tauri::async_runtime::block_on(async { client.get(source_url).send().await })
                .expect("receive original redirect response");
        let status = response.status().as_u16();
        let body = tauri::async_runtime::block_on(response.text()).expect("read redirect body");
        let (request, extra_source_request) = server.join().expect("join redirect source");
        let target_request = target.accept().is_ok();
        (status, body, request, extra_source_request, target_request)
    }

    #[test]
    fn loaded_state_client_rejects_redirect_without_retry_or_target_request() {
        let (status, body, request, extra_source_request, target_request) =
            redirected_get(ReqwestRunningProbe::new().client);

        assert_eq!(status, 307);
        assert_eq!(body, "redirect");
        assert!(request.starts_with("GET /source HTTP/1.1\r\n"));
        assert!(
            !extra_source_request,
            "source must receive exactly one request"
        );
        assert!(
            !target_request,
            "redirect target must not receive a request"
        );

        let mapped = interpret(responded(status, &body));
        assert_eq!(mapped.state, LoadedModelState::RespondedUnexpectedly);
        assert_eq!(mapped.source_availability, SourceAvailability::Ready);
    }

    // --- §8 rows: state + source availability + models ------------------------

    #[test]
    fn http_200_with_valid_one_model_is_available_ready_with_entry() {
        let body = r#"{
            "models": [
                {
                    "name": "llama3.1:8b",
                    "model": "llama3.1:8b",
                    "size": 1234567890,
                    "digest": "365c0bf3",
                    "details": {
                        "parent_model": "",
                        "format": "gguf",
                        "family": "llama",
                        "families": ["llama"],
                        "parameter_size": "8.0B",
                        "quantization_level": "Q4_0"
                    },
                    "expires_at": "2024-01-01T00:00:00.000Z",
                    "size_vram": 1234567890,
                    "context_length": 8192
                }
            ]
        }"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.runtime_type, OLLAMA);
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert_eq!(set.models.len(), 1);
        assert_eq!(set.skipped, 0);

        let m = &set.models[0];
        assert_eq!(m.display_name, "llama3.1:8b");
        assert_eq!(m.reported_size_bytes, Some(1_234_567_890));
        assert_eq!(m.reported_vram_bytes, Some(1_234_567_890));
        assert_eq!(m.reported_context_length, Some(8192));
        assert_eq!(m.expires_at.as_deref(), Some("2024-01-01T00:00:00.000Z"));
    }

    #[test]
    fn http_200_with_valid_multiple_models_preserves_order() {
        let body = r#"{
            "models": [
                {"name": "alpha:1b", "size": 1, "size_vram": 0, "context_length": 2048},
                {"name": "beta:2b", "size": 2, "size_vram": 2, "context_length": 4096},
                {"name": "gamma:3b", "size": 3, "size_vram": 3, "context_length": 8192}
            ]
        }"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(
            set.models
                .iter()
                .map(|m| m.display_name.clone())
                .collect::<Vec<_>>(),
            vec!["alpha:1b", "beta:2b", "gamma:3b"],
        );
        assert_eq!(set.models[2].reported_context_length, Some(8192));
    }

    #[test]
    fn http_200_with_empty_models_array_is_available_ready_and_empty() {
        let set = interpret(responded(200, r#"{"models": []}"#));
        // A valid empty list is Available with no models — not an error.
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert!(set.models.is_empty());
        assert_eq!(set.skipped, 0);
    }

    #[test]
    fn http_200_with_missing_models_is_responded_unexpectedly() {
        let set = interpret(responded(200, r#"{"something_else": 1}"#));
        assert_eq!(set.state, LoadedModelState::RespondedUnexpectedly);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert!(set.models.is_empty());
    }

    #[test]
    fn http_200_with_non_array_models_is_responded_unexpectedly() {
        let set = interpret(responded(200, r#"{"models": {}}"#));
        assert_eq!(set.state, LoadedModelState::RespondedUnexpectedly);
        assert!(set.models.is_empty());
    }

    #[test]
    fn http_200_with_malformed_json_is_responded_unexpectedly() {
        let set = interpret(responded(200, "not-json-at-all"));
        assert_eq!(set.state, LoadedModelState::RespondedUnexpectedly);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert!(set.models.is_empty());
    }

    #[test]
    fn non_200_http_status_is_responded_unexpectedly() {
        let set = interpret(responded(404, r#"{"models": []}"#));
        assert_eq!(set.state, LoadedModelState::RespondedUnexpectedly);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert!(set.models.is_empty());
    }

    #[test]
    fn connection_refused_is_not_detected() {
        let set = interpret(RunningOutcome::ConnectionRefused);
        assert_eq!(set.state, LoadedModelState::NotDetected);
        assert_eq!(set.source_availability, SourceAvailability::NotDetected);
        assert!(set.models.is_empty());
    }

    #[test]
    fn timeout_is_unreachable() {
        let set = interpret(RunningOutcome::Timeout);
        assert_eq!(set.state, LoadedModelState::Unreachable);
        assert_eq!(set.source_availability, SourceAvailability::Unreachable);
        assert!(set.models.is_empty());
    }

    #[test]
    fn other_transport_failure_is_unreachable() {
        let set = interpret(RunningOutcome::Other);
        assert_eq!(set.state, LoadedModelState::Unreachable);
        assert_eq!(set.source_availability, SourceAvailability::Unreachable);
        assert!(set.models.is_empty());
    }

    // --- Per-entry optional / invalid fields ----------------------------------

    #[test]
    fn missing_optional_fields_keeps_entry_valid_with_nones() {
        let body = r#"{"models": [{"name": "bare:1b"}]}"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.models.len(), 1);
        let m = &set.models[0];
        assert_eq!(m.display_name, "bare:1b");
        assert!(m.reported_size_bytes.is_none());
        assert!(m.reported_vram_bytes.is_none());
        assert!(m.reported_context_length.is_none());
        assert!(m.expires_at.is_none());
    }

    #[test]
    fn size_non_integer_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "size": "big"}]}"#;
        let set = interpret(responded(200, body));
        let m = &set.models[0];
        assert_eq!(m.display_name, "x:1b");
        assert!(m.reported_size_bytes.is_none());
    }

    #[test]
    fn size_negative_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "size": -5}]}"#;
        let set = interpret(responded(200, body));
        assert!(set.models[0].reported_size_bytes.is_none());
    }

    #[test]
    fn size_float_becomes_none() {
        let body = r#"{"models": [{"name": "x:1b", "size": 1.5}]}"#;
        let set = interpret(responded(200, body));
        assert!(set.models[0].reported_size_bytes.is_none());
    }

    #[test]
    fn size_large_unsigned_is_preserved() {
        let big: u64 = 9_000_000_000;
        let body = format!(r#"{{"models": [{{"name": "x:1b", "size": {big}}}]}}"#);
        let set = interpret(responded(200, &body));
        assert_eq!(set.models[0].reported_size_bytes, Some(big));
    }

    #[test]
    fn size_vram_invalid_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "size_vram": -1, "size": 10}]}"#;
        let set = interpret(responded(200, body));
        let m = &set.models[0];
        assert_eq!(m.reported_size_bytes, Some(10));
        assert!(m.reported_vram_bytes.is_none());
    }

    #[test]
    fn size_vram_large_unsigned_is_preserved() {
        let big: u64 = 8_000_000_000;
        let body = format!(r#"{{"models": [{{"name": "x:1b", "size_vram": {big}}}]}}"#);
        let set = interpret(responded(200, &body));
        assert_eq!(set.models[0].reported_vram_bytes, Some(big));
    }

    #[test]
    fn context_length_invalid_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "context_length": "wide", "size": 10}]}"#;
        let set = interpret(responded(200, body));
        let m = &set.models[0];
        assert_eq!(m.reported_size_bytes, Some(10));
        assert!(m.reported_context_length.is_none());
    }

    #[test]
    fn context_length_negative_becomes_none() {
        let body = r#"{"models": [{"name": "x:1b", "context_length": -8192}]}"#;
        let set = interpret(responded(200, body));
        assert!(set.models[0].reported_context_length.is_none());
    }

    #[test]
    fn expires_at_non_string_becomes_none() {
        let body = r#"{"models": [{"name": "x:1b", "expires_at": 12345}]}"#;
        let set = interpret(responded(200, body));
        assert!(set.models[0].expires_at.is_none());
    }

    #[test]
    fn expires_at_is_preserved_verbatim() {
        let body = r#"{"models": [{"name": "x:1b", "expires_at": "2024-01-01T00:00:00.000Z"}]}"#;
        let set = interpret(responded(200, body));
        assert_eq!(
            set.models[0].expires_at.as_deref(),
            Some("2024-01-01T00:00:00.000Z")
        );
    }

    #[test]
    fn name_falls_back_to_model_when_absent() {
        let body = r#"{"models": [{"model": "via-model:1b", "size": 10}]}"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.models.len(), 1);
        assert_eq!(set.models[0].display_name, "via-model:1b");
        assert_eq!(set.models[0].reported_size_bytes, Some(10));
    }

    #[test]
    fn empty_name_falls_back_to_model() {
        let body = r#"{"models": [{"name": "   ", "model": "fb:1b"}]}"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.models[0].display_name, "fb:1b");
    }

    #[test]
    fn entry_with_no_usable_identity_is_skipped_siblings_kept() {
        let body = r#"{
            "models": [
                {"name": "good:1b", "size": 1},
                {"size": 2},
                {"model": "   "},
                {"name": "also-good:2b", "size": 3}
            ]
        }"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(set.models.len(), 2);
        assert_eq!(set.skipped, 2);
        assert_eq!(set.models[0].display_name, "good:1b");
        assert_eq!(set.models[1].display_name, "also-good:2b");
    }

    // --- Honesty invariants ---------------------------------------------------

    #[test]
    fn no_locality_inference_from_cloud_name_suffix() {
        // No inference from a :cloud-style name suffix or any field: the parser
        // carries only the documented fields and never classifies locality.
        let body = r#"{
            "models": [
                {"name": "thing:cloud", "size": 0, "size_vram": 0, "context_length": 0}
            ]
        }"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.models.len(), 1);
        assert_eq!(set.models[0].display_name, "thing:cloud");
    }

    #[test]
    fn undocumented_extra_fields_are_ignored() {
        // digest, details.*, processor, capabilities, and remote/cloud fields
        // are not part of the documented /api/ps contract the parser reads: it
        // ignores them rather than copying or exposing them. Only the
        // user-facing/forward-use fields are carried.
        let body = r#"{
            "models": [{
                "name": "x:1b",
                "size": 1,
                "size_vram": 0,
                "context_length": 2048,
                "expires_at": "2024-01-01T00:00:00.000Z",
                "digest": "deadbeef",
                "details": {
                    "parent_model": "",
                    "format": "gguf",
                    "family": "llama",
                    "families": ["llama"],
                    "parameter_size": "1.0B",
                    "quantization_level": "Q4_0"
                },
                "processor": "100% GPU",
                "capabilities": ["tools"],
                "remote_host": "example.invalid"
            }]
        }"#;
        let set = interpret(responded(200, body));
        let m = &set.models[0];
        assert_eq!(m.display_name, "x:1b");
        assert_eq!(m.reported_size_bytes, Some(1));
        assert_eq!(m.reported_vram_bytes, Some(0));
        assert_eq!(m.reported_context_length, Some(2048));
        assert_eq!(m.expires_at.as_deref(), Some("2024-01-01T00:00:00.000Z"));
        // No field on LoadedModelSummary carries digest/details/processor/
        // capabilities/remote_host — they are structurally excluded.
    }

    #[test]
    fn loaded_model_summary_carries_only_controlled_fields() {
        // The summary type exposes no digest/details/endpoint/host field; a
        // digest present in the body must not surface anywhere.
        let body = r#"{"models": [{"name": "x:1b", "size": 1, "digest": "deadbeef", "remote_host": "example.invalid"}]}"#;
        let set = interpret(responded(200, body));
        assert_eq!(set.models.len(), 1);
        let m = &set.models[0];
        let _: &LoadedModelSummary = m;
        assert_eq!(m.display_name, "x:1b");
        assert_eq!(m.reported_size_bytes, Some(1));
    }

    #[test]
    fn interpret_loaded_models_is_pure_and_runtime_type_carried() {
        // The same outcome maps consistently; runtime_type is carried through
        // untouched.
        let set = interpret(responded(200, r#"{"models": []}"#));
        assert_eq!(set.runtime_type, OLLAMA);
        let set2 = interpret(RunningOutcome::ConnectionRefused);
        assert_eq!(set2.runtime_type, OLLAMA);
    }

    #[test]
    fn ps_endpoint_is_documented_loopback() {
        assert_eq!(OLLAMA_PS_ENDPOINT, "http://127.0.0.1:11434/api/ps");
    }
}
