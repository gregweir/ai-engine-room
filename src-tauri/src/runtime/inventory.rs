//! Milestone 1D model inventory — application layer, not `aer-core`.
//!
//! A detected runtime's *model catalogue* is structured runtime metadata, not a
//! set of numeric metrics. These types live here in `src-tauri` so `aer-core`,
//! `SampleValue`, `MetricSample`, metric `Provenance`, the report boundary, and
//! the `AiRuntimeAdapter` trait stay unchanged. `SourceAvailability` is reused
//! from `aer-core` unchanged (as in 1C). The inventory flows through the
//! concrete `OllamaAdapter::inventory()` method — exactly as 1C routed the
//! version through `runtime_status()` rather than `runtime_metrics()` — and
//! `list_models()` stays empty for 1D.
//!
//! `ModelsOutcome` is the controlled result of one `GET /api/tags` probe, free
//! of any `reqwest` type. The production transport (`ReqwestModelsProbe`) and
//! any test transport (`FakeModelsProbe`) both reduce to this enum, so the
//! inventory semantics in [`interpret_inventory`] — the deterministic heart of
//! 1D — are testable with fixtures and no socket, no `reqwest`, and no live
//! Ollama.
//!
//! Only the documented local `http://127.0.0.1:11434/api/tags` endpoint is
//! requested. Only the freshly-verified documented `/api/tags` fields are
//! parsed (name, model, size, details.{family, families, parameter_size,
//! quantization_level}); any extra field a live runtime returns beyond that
//! contract is ignored by the parser and never copied into fixtures. No
//! `digest`, `endpoint`, host, port, or `remote_host` crosses this boundary.

use std::future::Future;
use std::time::Duration;

use aer_core::SourceAvailability;

use crate::runtime::status::RuntimeType;

/// Ollama's documented local model-list endpoint, used explicitly as the
/// loopback IP to avoid `localhost` IPv6/IPv4 resolution ambiguity (mirrors
/// 1C's version endpoint). No configurable endpoint in 1D; no hostname, LAN
/// address, or model name leaks through the view boundary.
pub const OLLAMA_TAGS_ENDPOINT: &str = "http://127.0.0.1:11434/api/tags";

/// Connect deadline for the local model-list probe: 500 ms (mirrors 1C).
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Overall request deadline (connect + read): 2 seconds (mirrors 1C). A
/// hanging responder cannot stall the UI beyond this.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Evidence-based locality of a catalogue entry. No inference from name
/// suffixes (e.g. `:cloud`), from `size`, from the model name, or from
/// undocumented `remote_model`/`remote_host` fields.
///
/// In 1D every entry is classified [`ModelLocality::Undetermined`]: the
/// documented `/api/tags` contract lists "available models," which is
/// insufficient evidence that an entry is physically local, and there is no
/// approved evidence rule to claim `Local` or `Remote`. `Local` is kept as a
/// variant so the generic model can support a future locality classification
/// once an approved evidence rule exists; it is **not** assigned by 1D.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelLocality {
    /// Reserved for a future, approved, evidence-based "this entry is local"
    /// rule. Not assigned in 1D.
    Local,
    /// Locality could not be established from the documented contract. The
    /// default and only classification 1D assigns.
    Undetermined,
    // Remote/Cloud deliberately absent: add only with a documented field and
    // an approved developer decision.
}

/// One model in a runtime's inventory. Runtime-neutral; no Ollama JSON shape
/// leaks here. Holds only user-facing fields; `digest`, endpoint, host, and
/// `remote_host` are deliberately excluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeModelSummary {
    /// The model's catalogue name (Ollama `name`, verbatim), falling back to
    /// `model` only during parsing when `name` is absent. How the user refers
    /// to the model.
    pub display_name: String,
    /// Evidence-based locality. Always [`ModelLocality::Undetermined`] in 1D.
    pub locality: ModelLocality,
    /// Ollama `size` in bytes — the reported model artifact size. `None` when
    /// absent, non-integer, or negative (never `0` for "no size").
    pub size_bytes: Option<u64>,
    /// Ollama `details.parameter_size` verbatim (e.g. `"8.0B"`), when present.
    pub parameter_size: Option<String>,
    /// Ollama `details.quantization_level` verbatim (e.g. `"Q4_0"`), when
    /// present.
    pub quantization: Option<String>,
    /// `details.families` preferred; `details.family` as a single-element
    /// fallback. Held internally for forward use; not displayed per model in
    /// 1D's initial UI.
    pub families: Vec<String>,
}

/// The state of an inventory fetch, mirroring the 1C four-state model for
/// `/api/tags`. A valid empty model list is [`InventoryState::Available`] with
/// no models — it is **not** an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryState {
    /// Reached and the top-level JSON was parseable with a `models` array.
    /// The model list may be empty.
    Available,
    /// Reached (the source answered) but the response was non-200 or the
    /// top-level JSON was missing/malformed (no usable `models` array).
    RespondedUnexpectedly,
    /// Nothing answered at the local endpoint (connection refused).
    NotDetected,
    /// Timeout or other transport failure.
    Unreachable,
}

/// The full inventory result for a runtime at one fetch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInventory {
    pub runtime_type: RuntimeType,
    pub state: InventoryState,
    /// Reused from `aer-core` unchanged (as in 1C): reflects whether the
    /// source answered, independent of the catalogue's per-entry validity.
    pub source_availability: SourceAvailability,
    /// Parsed catalogue entries. Empty unless `state == Available`.
    pub models: Vec<RuntimeModelSummary>,
    /// Entries dropped because they had no usable identity (no non-empty
    /// `name`/`model`). A malformed individual entry does not fail the whole
    /// list; its valid siblings are kept.
    pub skipped: usize,
}

/// The controlled result of one local `GET /api/tags` probe, free of
/// transport types. A returned HTTP response reduces to `Responded { status,
/// body }`; transport failures reduce to the honest categories below. No raw
/// error, URL, header, or hostname crosses this boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelsOutcome {
    /// An HTTP response arrived. `status` is the HTTP status code; `body` is
    /// the raw response body text (parsed later, in the pure layer).
    Responded { status: u16, body: String },
    /// Nothing was listening at the local endpoint (connection refused).
    ConnectionRefused,
    /// The connect or read attempt exceeded its deadline.
    Timeout,
    /// Unexpected I/O not expected on localhost plain HTTP.
    Other,
}

/// A transport that performs one local model-list probe and reduces the
/// result to a controlled [`ModelsOutcome`]. `Send + Sync` so an adapter can
/// be shared across the Tauri async boundary. Implementations keep all
/// transport types (e.g. `reqwest`) confined to themselves.
pub trait ModelsProbe: Send + Sync {
    /// Perform one fetch and return the controlled outcome.
    fn fetch(&self) -> impl Future<Output = ModelsOutcome> + Send;
}

/// Map one models-probe outcome to a controlled inventory. Pure: no I/O, no
/// transport types. `runtime_type` is carried through so the pure layer stays
/// runtime-neutral; the caller (the Ollama adapter) supplies it.
///
/// Mapping (design §8):
///
/// | `ModelsOutcome`                                   | `InventoryState`        | `source_availability` | `models`            |
/// |---------------------------------------------------|-------------------------|------------------------|---------------------|
/// | `Responded { 200, valid JSON with models array }` | `Available`              | `Ready`                | parsed (may be empty) |
/// | `Responded { 200, missing/non-array models }`     | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `Responded { 200, malformed JSON }`               | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `Responded { non-200, .. }`                       | `RespondedUnexpectedly` | `Ready`                | empty               |
/// | `ConnectionRefused`                               | `NotDetected`           | `NotDetected`          | empty               |
/// | `Timeout` / `Other`                               | `Unreachable`           | `Unreachable`          | empty               |
///
/// Per-entry parsing: an entry is valid if it has a non-empty `name`
/// (falling back to `model`). `size` is a non-negative integer →
/// `size_bytes`; absent/non-integer/negative → `None` (entry still valid).
/// `details` is optional; `parameter_size`/`quantization_level` are copied
/// verbatim when present and string-typed. `families` is preferred; when
/// absent, `family` is a single-element list. An entry with no usable
/// identity is skipped and counted in `skipped`. Locality is
/// [`ModelLocality::Undetermined`] for every valid entry: no inference from
/// name, size, or any undocumented field. Extra fields beyond the documented
/// contract are ignored.
pub fn interpret_inventory(outcome: ModelsOutcome, runtime_type: RuntimeType) -> ModelInventory {
    match outcome {
        ModelsOutcome::Responded { status, body } => {
            // The endpoint answered, so the source is `Ready` regardless of
            // whether the catalogue body was usable.
            if status != 200 {
                return ModelInventory {
                    runtime_type,
                    state: InventoryState::RespondedUnexpectedly,
                    source_availability: SourceAvailability::Ready,
                    models: Vec::new(),
                    skipped: 0,
                };
            }

            let value: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => {
                    return ModelInventory {
                        runtime_type,
                        state: InventoryState::RespondedUnexpectedly,
                        source_availability: SourceAvailability::Ready,
                        models: Vec::new(),
                        skipped: 0,
                    };
                }
            };

            let models_array = match value.get("models") {
                Some(serde_json::Value::Array(arr)) => arr,
                _ => {
                    return ModelInventory {
                        runtime_type,
                        state: InventoryState::RespondedUnexpectedly,
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

            ModelInventory {
                runtime_type,
                state: InventoryState::Available,
                source_availability: SourceAvailability::Ready,
                models,
                skipped,
            }
        }
        ModelsOutcome::ConnectionRefused => ModelInventory {
            runtime_type,
            state: InventoryState::NotDetected,
            source_availability: SourceAvailability::NotDetected,
            models: Vec::new(),
            skipped: 0,
        },
        // Timeout and other transport failures are grouped: the connect-vs-read
        // distinction is not surfaced in 1D, mirroring 1C.
        ModelsOutcome::Timeout | ModelsOutcome::Other => ModelInventory {
            runtime_type,
            state: InventoryState::Unreachable,
            source_availability: SourceAvailability::Unreachable,
            models: Vec::new(),
            skipped: 0,
        },
    }
}

/// Parse one `/api/tags` entry (`serde_json::Value`) into a
/// [`RuntimeModelSummary`], or `None` if the entry has no usable identity.
///
/// Only documented fields are read; anything else a live runtime returns is
/// ignored. `size` is read as a non-negative integer (i64 or u64); absent,
/// non-integer, or negative → `None`. `details` is optional. Locality is
/// always [`ModelLocality::Undetermined`] (no inference).
fn parse_entry(value: &serde_json::Value) -> Option<RuntimeModelSummary> {
    let object = value.as_object()?;

    // Identity: first non-empty string of `name`, then `model` (parsing
    // fallback only). Whitespace-only counts as empty; the kept value is
    // verbatim (not trimmed).
    let identity =
        non_empty_str(object.get("name")).or_else(|| non_empty_str(object.get("model")))?;
    let display_name = identity.to_string();

    // `size`: documented integer bytes. Accept i64 (to detect negatives) and
    // u64 (to cover the full unsigned range); anything else → None.
    let size_bytes = object.get("size").and_then(size_to_u64);

    let details = object.get("details").and_then(|v| v.as_object());
    let parameter_size = details
        .and_then(|d| d.get("parameter_size"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let quantization = details
        .and_then(|d| d.get("quantization_level"))
        .and_then(|v| v.as_str())
        .map(String::from);

    // `families` preferred; `family` as a single-element fallback.
    let families = families_of(details);

    Some(RuntimeModelSummary {
        display_name,
        locality: ModelLocality::Undetermined,
        size_bytes,
        parameter_size,
        quantization,
        families,
    })
}

/// A non-empty (non-whitespace) string view of a JSON value, or `None`.
fn non_empty_str(value: Option<&serde_json::Value>) -> Option<&str> {
    value
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
}

/// Read a JSON number as a non-negative `u64`. Accepts both integer-encoded
/// and unsigned-overflowing values; rejects negatives, non-numbers, and
/// floats (the documented field is an integer).
fn size_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}

/// Resolve the `families` list: prefer `details.families` (string elements
/// only); if absent/empty/non-array, fall back to a single-element list from
/// `details.family`; otherwise empty.
fn families_of(details: Option<&serde_json::Map<String, serde_json::Value>>) -> Vec<String> {
    if let Some(d) = details {
        if let Some(serde_json::Value::Array(arr)) = d.get("families") {
            let families: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !families.is_empty() {
                return families;
            }
        }
        if let Some(family) = d.get("family").and_then(|v| v.as_str()) {
            return vec![family.to_string()];
        }
    }
    Vec::new()
}

/// Production transport: a `reqwest::Client` with bounded timeouts, used only
/// for the documented local `GET /api/tags`. All `reqwest` types stay in this
/// struct and its `ModelsProbe` impl. Mirrors 1C's `ReqwestProbe` timeouts and
/// honest error classification.
pub struct ReqwestModelsProbe {
    client: reqwest::Client,
}

impl ReqwestModelsProbe {
    /// Build a client with the bounded 1D/1C timeouts. Reused across probes.
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

impl Default for ReqwestModelsProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelsProbe for ReqwestModelsProbe {
    fn fetch(&self) -> impl Future<Output = ModelsOutcome> + Send {
        let client = self.client.clone();
        async move {
            match client.get(OLLAMA_TAGS_ENDPOINT).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    match response.text().await {
                        Ok(body) => ModelsOutcome::Responded { status, body },
                        // Body read failed: classify the transport error
                        // honestly rather than fabricating a response. No raw
                        // error crosses the boundary.
                        Err(error) => classify_reqwest_error(error),
                    }
                }
                Err(error) => classify_reqwest_error(error),
            }
        }
    }
}

/// Reduce a `reqwest` error to a controlled [`ModelsOutcome`]. No raw error
/// text leaves this function. Timeout is checked first, and only an underlying
/// `std::io::ErrorKind::ConnectionRefused` maps to `ConnectionRefused`
/// (NotDetected). Everything else is `Other` (Unreachable) — conservative, no
/// fabricated certainty. Mirrors 1C's `classify_reqwest_error`.
fn classify_reqwest_error(error: reqwest::Error) -> ModelsOutcome {
    if error.is_timeout() {
        return ModelsOutcome::Timeout;
    }

    if let Some(io_error) = find_source_io_error(&error) {
        if io_error.kind() == std::io::ErrorKind::ConnectionRefused {
            return ModelsOutcome::ConnectionRefused;
        }
    }

    ModelsOutcome::Other
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
    //! Deterministic coverage of `interpret_inventory` over every design-§8
    //! row, plus the per-entry parsing rules, honesty invariants, and one
    //! ephemeral-loopback transport check. No live Ollama.

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;
    use crate::runtime::status::RuntimeType;
    use aer_core::SourceAvailability;

    const OLLAMA: RuntimeType = RuntimeType::Ollama;

    fn responded(status: u16, body: &str) -> ModelsOutcome {
        ModelsOutcome::Responded {
            status,
            body: body.to_string(),
        }
    }

    fn interpret(outcome: ModelsOutcome) -> ModelInventory {
        interpret_inventory(outcome, OLLAMA)
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
    fn inventory_client_rejects_redirect_without_retry_or_target_request() {
        let (status, body, request, extra_source_request, target_request) =
            redirected_get(ReqwestModelsProbe::new().client);

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
        assert_eq!(mapped.state, InventoryState::RespondedUnexpectedly);
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
                    "size": 4975683088,
                    "details": {
                        "family": "llama",
                        "families": ["llama"],
                        "parameter_size": "8.0B",
                        "quantization_level": "Q4_0"
                    }
                }
            ]
        }"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.runtime_type, OLLAMA);
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert_eq!(inv.models.len(), 1);
        assert_eq!(inv.skipped, 0);

        let m = &inv.models[0];
        assert_eq!(m.display_name, "llama3.1:8b");
        assert_eq!(m.locality, ModelLocality::Undetermined);
        assert_eq!(m.size_bytes, Some(4_975_683_088));
        assert_eq!(m.parameter_size.as_deref(), Some("8.0B"));
        assert_eq!(m.quantization.as_deref(), Some("Q4_0"));
        assert_eq!(m.families, vec!["llama".to_string()]);
    }

    #[test]
    fn http_200_with_valid_multiple_models_preserves_order() {
        let body = r#"{
            "models": [
                {"name": "alpha:1b", "size": 1},
                {"name": "beta:2b", "size": 2},
                {"name": "gamma:3b", "size": 3}
            ]
        }"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(
            inv.models
                .iter()
                .map(|m| m.display_name.clone())
                .collect::<Vec<_>>(),
            vec!["alpha:1b", "beta:2b", "gamma:3b"],
        );
    }

    #[test]
    fn http_200_with_empty_models_array_is_available_ready_and_empty() {
        let inv = interpret(responded(200, r#"{"models": []}"#));
        // A valid empty list is Available with no models — not an error.
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert!(inv.models.is_empty());
        assert_eq!(inv.skipped, 0);
    }

    #[test]
    fn http_200_with_missing_models_is_responded_unexpectedly() {
        let inv = interpret(responded(200, r#"{"something_else": 1}"#));
        assert_eq!(inv.state, InventoryState::RespondedUnexpectedly);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn http_200_with_non_array_models_is_responded_unexpectedly() {
        let inv = interpret(responded(200, r#"{"models": {}}"#));
        assert_eq!(inv.state, InventoryState::RespondedUnexpectedly);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn http_200_with_malformed_json_is_responded_unexpectedly() {
        let inv = interpret(responded(200, "not-json-at-all"));
        assert_eq!(inv.state, InventoryState::RespondedUnexpectedly);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn non_200_http_status_is_responded_unexpectedly() {
        let inv = interpret(responded(404, r#"{"models": []}"#));
        assert_eq!(inv.state, InventoryState::RespondedUnexpectedly);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn connection_refused_is_not_detected() {
        let inv = interpret(ModelsOutcome::ConnectionRefused);
        assert_eq!(inv.state, InventoryState::NotDetected);
        assert_eq!(inv.source_availability, SourceAvailability::NotDetected);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn timeout_is_unreachable() {
        let inv = interpret(ModelsOutcome::Timeout);
        assert_eq!(inv.state, InventoryState::Unreachable);
        assert_eq!(inv.source_availability, SourceAvailability::Unreachable);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn other_transport_failure_is_unreachable() {
        let inv = interpret(ModelsOutcome::Other);
        assert_eq!(inv.state, InventoryState::Unreachable);
        assert_eq!(inv.source_availability, SourceAvailability::Unreachable);
        assert!(inv.models.is_empty());
    }

    // --- Per-entry optional / invalid fields ----------------------------------

    #[test]
    fn missing_details_and_size_keeps_entry_valid_with_nones() {
        let body = r#"{"models": [{"name": "bare:1b"}]}"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.models.len(), 1);
        let m = &inv.models[0];
        assert_eq!(m.display_name, "bare:1b");
        assert!(m.size_bytes.is_none());
        assert!(m.parameter_size.is_none());
        assert!(m.quantization.is_none());
        assert!(m.families.is_empty());
    }

    #[test]
    fn size_non_integer_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "size": "big"}]}"#;
        let inv = interpret(responded(200, body));
        let m = &inv.models[0];
        assert_eq!(m.display_name, "x:1b");
        assert!(m.size_bytes.is_none());
    }

    #[test]
    fn size_negative_becomes_none_entry_still_valid() {
        let body = r#"{"models": [{"name": "x:1b", "size": -5}]}"#;
        let inv = interpret(responded(200, body));
        assert!(inv.models[0].size_bytes.is_none());
    }

    #[test]
    fn size_large_unsigned_is_preserved() {
        let big: u64 = 9_000_000_000;
        let body = format!(r#"{{"models": [{{"name": "x:1b", "size": {big}}}]}}"#);
        let inv = interpret(responded(200, &body));
        assert_eq!(inv.models[0].size_bytes, Some(big));
    }

    #[test]
    fn size_float_becomes_none() {
        let body = r#"{"models": [{"name": "x:1b", "size": 1.5}]}"#;
        let inv = interpret(responded(200, body));
        assert!(inv.models[0].size_bytes.is_none());
    }

    #[test]
    fn families_preferred_over_family() {
        let body = r#"{"models": [{"name": "x:1b", "details": {"family": "llama", "families": ["llama", "gemma"]}}]}"#;
        let inv = interpret(responded(200, body));
        assert_eq!(
            inv.models[0].families,
            vec!["llama".to_string(), "gemma".to_string()],
        );
    }

    #[test]
    fn family_only_entry_uses_single_element_list() {
        let body = r#"{"models": [{"name": "x:1b", "details": {"family": "llama"}}]}"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.models[0].families, vec!["llama".to_string()]);
    }

    #[test]
    fn name_falls_back_to_model_when_absent() {
        let body = r#"{"models": [{"model": "via-model:1b", "size": 10}]}"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.models.len(), 1);
        assert_eq!(inv.models[0].display_name, "via-model:1b");
        assert_eq!(inv.models[0].size_bytes, Some(10));
    }

    #[test]
    fn empty_name_falls_back_to_model() {
        let body = r#"{"models": [{"name": "   ", "model": "fb:1b"}]}"#;
        let inv = interpret(responded(200, body));
        assert_eq!(inv.models[0].display_name, "fb:1b");
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
        let inv = interpret(responded(200, body));
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(inv.models.len(), 2);
        assert_eq!(inv.skipped, 2);
        assert_eq!(inv.models[0].display_name, "good:1b");
        assert_eq!(inv.models[1].display_name, "also-good:2b");
    }

    // --- Honesty invariants ---------------------------------------------------

    #[test]
    fn every_valid_entry_is_undetermined_no_local_or_remote_claim() {
        // No inference from a :cloud-style name suffix, size, or any field.
        let body = r#"{
            "models": [
                {"name": "thing:cloud", "size": 0},
                {"name": "thing:8b", "size": 4975683088},
                {"name": "zero:1b", "size": 0}
            ]
        }"#;
        let inv = interpret(responded(200, body));
        for m in &inv.models {
            assert_eq!(m.locality, ModelLocality::Undetermined);
        }
        assert_eq!(inv.models.len(), 3);
    }

    #[test]
    fn undocumented_extra_fields_are_ignored() {
        // remote_model/remote_host/capabilities/context length are not part
        // of the documented /api/tags contract (§2.2): the parser ignores
        // them rather than copying or exposing them.
        let body = r#"{
            "models": [{
                "name": "x:1b",
                "size": 1,
                "digest": "deadbeef",
                "remote_model": "x:cloud",
                "remote_host": "example.invalid",
                "capabilities": ["tools"],
                "details": {
                    "parameter_size": "1.0B",
                    "quantization_level": "Q4_0",
                    "context_length": 8192,
                    "embedding_length": 768
                }
            }]
        }"#;
        let inv = interpret(responded(200, body));
        let m = &inv.models[0];
        assert_eq!(m.display_name, "x:1b");
        assert_eq!(m.size_bytes, Some(1));
        assert_eq!(m.parameter_size.as_deref(), Some("1.0B"));
        assert_eq!(m.quantization.as_deref(), Some("Q4_0"));
        // No field on RuntimeModelSummary carries digest/remote/capabilities.
    }

    #[test]
    fn interpret_inventory_is_pure_and_runtime_type_carried() {
        // The same outcome maps consistently; runtime_type is carried through
        // untouched.
        let inv = interpret(responded(200, r#"{"models": []}"#));
        assert_eq!(inv.runtime_type, OLLAMA);
        let inv2 = interpret(ModelsOutcome::ConnectionRefused);
        assert_eq!(inv2.runtime_type, OLLAMA);
    }

    #[test]
    fn tags_endpoint_is_documented_loopback() {
        assert_eq!(OLLAMA_TAGS_ENDPOINT, "http://127.0.0.1:11434/api/tags");
    }
}
