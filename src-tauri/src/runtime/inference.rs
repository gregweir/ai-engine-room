//! Milestone 1G observed inference — application layer, not `aer-core`.
//!
//! An "observed inference" is one real, non-streaming `POST /api/generate`
//! request carrying a **fixed synthetic diagnostic prompt** to a model the user
//! explicitly selected from the 1D catalogue. The result is the counts and
//! durations Ollama reports for that single run, plus an Engine-Room-calculated
//! generation rate. It is **one observed run**, not a general benchmark or
//! model-quality score, and Engine Room never claims the model ran locally.
//!
//! These types live here in `src-tauri` so `aer-core`, `SampleValue`,
//! `MetricSample`, metric `Provenance`, the report boundary, and the
//! `AiRuntimeAdapter` trait stay unchanged. The observation flows through a
//! concrete `OllamaAdapter::inference_observation(model)` method — parallel to
//! 1C's `runtime_status()`, 1D's `inventory()`, and 1E's
//! `loaded_models_status()` — NOT through `runtime_metrics()`, which stays
//! unsupported. No `AiRuntimeAdapter` trait method is added.
//!
//! `GenerateOutcome` is the controlled result of one `POST /api/generate`
//! probe, free of any `reqwest` type. The production transport
//! (`ReqwestGenerateProbe`) and any test transport (`FakeGenerateProbe`) both
//! reduce to this enum, so the inference semantics in [`interpret_inference`] —
//! the deterministic heart of 1G — are testable with fixtures and no socket, no
//! `reqwest`, and no live Ollama.
//!
//! Only the documented local `http://127.0.0.1:11434/api/generate` endpoint is
//! requested. Only the fixed diagnostic prompt/profile is ever sent; `num_predict`
//! is bounded to 64, `temperature` is 0, `stream` is false. **No `seed` is sent.
//! No `keep_alive` is sent** (Ollama's current residency configuration is left
//! untouched). No system prompt, no tools, no images. The request body is built
//! with `serde_json` and POSTed as a string body with a `content-type` header —
//! `reqwest` is configured with `default-features = false`, so the `json` feature
//! is unavailable and `.json()` is not used (no Cargo change).
//!
//! **Generated-output containment:** the `response` field Ollama returns is
//! discarded by [`interpret_inference`]. It is never stored, logged, reported,
//! or surfaced in any view DTO. The fixed prompt is never echoed. No raw error,
//! endpoint, host, port, status code, or transport detail crosses the view
//! boundary. The controlled human text travels through [`ollama_inference_text`].
//!
//! **Execution boundary:** Engine Room cannot determine from Ollama's current
//! API whether a model will run on this computer or through Ollama Cloud. The
//! observation therefore carries `execution_boundary: "undetermined"` — the only
//! value in 1G — and the UI states "Execution location not determined." This is
//! not locality verification. Authorization for one run is per-run and
//! request-scoped (handled in the command layer); it is not cryptographic and
//! not a locality signal.

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use crate::runtime::inventory::{InventoryState, ModelsProbe};
use crate::runtime::loaded_models::RunningProbe;
use crate::runtime::ollama::OllamaAdapter;
use crate::runtime::probe::VersionProbe;

/// Ollama's documented local generation endpoint ("Generate a completion"),
/// used explicitly as the loopback IP to avoid `localhost` IPv6/IPv4 resolution
/// ambiguity (mirrors 1C/1D/1E). No configurable endpoint in 1G; no hostname,
/// LAN address, or generated text leaks through the view boundary.
pub const OLLAMA_GENERATE_ENDPOINT: &str = "http://127.0.0.1:11434/api/generate";

/// The controlled diagnostic profile identifier for 1G's fixed observation.
/// ER-controlled metadata; carried to the view so the UI can name what ran.
pub const DIAGNOSTIC_PROFILE: &str = "diagnostic-basic-v1";

/// The fixed synthetic diagnostic prompt. Contains no user files, repository
/// content, machine information, or other user-provided data. Never echoed back
/// to the UI; never varied.
pub const DIAGNOSTIC_PROMPT: &str = "In one short sentence, explain why leaves appear green.";

/// Maximum number of tokens to generate for the diagnostic observation.
/// Bounded so a single run cannot produce unbounded output or run indefinitely.
pub const NUM_PREDICT: u64 = 64;

/// Sampling temperature for the diagnostic observation. 0 minimises randomness
/// so the observation is as reproducible as one run can be (not a guarantee).
pub const TEMPERATURE: f64 = 0.0;

/// Non-streaming completion: the whole response is returned in one JSON object
/// so the counts/durations are present in a single body. 1G does not stream.
pub const STREAM: bool = false;

// Compile-time pin that streaming is disabled (clippy-blessed form for a
// constant bool assertion; a runtime `assert!`/`assert_eq!` trips
// `assertions_on_constants`/`bool_assert_comparison`).
const _: () = {
    assert!(!STREAM);
};

/// The only execution-boundary value 1G produces. The view DTO carries this
/// verbatim as a controlled string field (no enum, no Local/Cloud variants).
pub const EXECUTION_BOUNDARY_UNDETERMINED: &str = "undetermined";

/// Connect deadline for the local generate probe: 2 seconds (per the 1G plan;
/// longer than 1C/1D/1E's 500 ms because a generate request may need to load a
/// model before accepting the connection).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Overall request deadline (connect + read): 120 seconds. A single diagnostic
/// generation of at most 64 tokens is bounded by this; no retry is performed.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

// --- Controlled disclosure + rate-label text (single source of truth) ----------
//
// The §4 disclosure copy and the §8 rate label/explanation are Rust-authored
// controlled strings. The deterministic Rust tests assert against these
// constants directly; the frontend mirrors them verbatim (pinned by its own
// tests). Keeping them here means the wording has one auditable source on the
// backend.

/// Disclosure dialog title.
pub const DISCLOSURE_TITLE: &str = "Run an observed inference?";

/// Disclosure dialog body paragraphs, in display order. Verbatim from the 1G
/// plan §4.
pub const DISCLOSURE_BODY: &[&str] = &[
    "Engine Room will send a fixed synthetic diagnostic prompt to the selected Ollama model.",
    "Engine Room cannot determine from Ollama's current API whether this model will run on this computer or through Ollama Cloud.",
    "The diagnostic prompt contains no user files, repository content, machine information, or other user-provided data.",
    "This will perform one real inference and may load the model and use CPU, GPU, and memory resources. Ollama may keep the model loaded afterward according to its current configuration.",
    "The result is one observed run, not a general benchmark or model-quality score. This authorization applies to one run only; another run requires another authorization.",
];

/// Primary disclosure action label. Terminology: "Run observation".
pub const DISCLOSURE_PRIMARY_ACTION: &str = "Run observation";

/// Secondary disclosure action label. There is intentionally no
/// "Don't ask again" action.
pub const DISCLOSURE_SECONDARY_ACTION: &str = "Cancel";

/// Controlled label for the Engine-Room-calculated generation rate. Not an
/// Ollama-reported figure.
pub const GENERATION_RATE_LABEL: &str = "Observed generation rate";

/// Controlled explanation of how the generation rate was calculated.
pub const GENERATION_RATE_EXPLANATION: &str =
    "Calculated from the output-token count and generation duration Ollama reported for this run.";

/// The controlled "execution location not determined" limitation sentence,
/// carried in the `limitations` field of the observation text.
pub const EXECUTION_LOCATION_NOT_DETERMINED: &str = "Execution location not determined.";

/// The state of an observed-inference run, serialised as a controlled
/// snake_case string. The nine values are the complete 1G result model.
///
/// `NoModelSelected`, `BlockedByExecutionBoundary`, `Running`, `NotDetected`,
/// and `ModelUnavailable` are produced by the command-layer orchestration (no
/// model / no authorization / already running / Ollama not detected / model off
/// the current catalogue). `Completed`, `Unreachable`, `TimedOut`, and
/// `RespondedUnexpectedly` are produced by [`interpret_inference`] from a
/// `GenerateOutcome`. The enum is shared by the intermediate observation and
/// the view DTO so the view layer depends on the runtime layer (never the
/// reverse), matching 1D/1E's dependency direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceObservationState {
    /// Run activated with no usable model selected (defensive; no POST).
    NoModelSelected,
    /// The user did not acknowledge the undetermined execution boundary for this
    /// run. No POST is issued.
    BlockedByExecutionBoundary,
    /// One observation is already in progress; a second concurrent run is not
    /// issued. No second POST.
    Running,
    /// One observation completed; the counts/durations Ollama reported and the
    /// ER-calculated rate are present.
    Completed,
    /// Ollama was not detected during the catalogue identity check. No POST.
    NotDetected,
    /// Ollama could not be reached, or a transport failure occurred on the
    /// generate request.
    Unreachable,
    /// The 120 s request deadline elapsed on the generate request. No retry.
    TimedOut,
    /// Ollama answered but the response was not a usable completed generation
    /// (non-200, malformed, missing `done`/`model`).
    RespondedUnexpectedly,
    /// The selected model was not found in the current Ollama catalogue. No
    /// POST. Identity/availability only — not a locality determination.
    ModelUnavailable,
}

/// The intermediate observed-inference result produced by [`interpret_inference`]
/// (and by the command layer for the non-POST states). Runtime-neutral; no
/// `reqwest` type, no generated `response` text, no raw error crosses here.
/// Provenance is field-level: the counts/durations are Ollama-reported (read
/// verbatim from the generate response), and `generation_tokens_per_second` is
/// ER-calculated. Durations are carried in nanoseconds (Ollama's unit).
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceObservation {
    pub state: InferenceObservationState,
    /// The model the run targeted, when one was selected. `None` only for
    /// `NoModelSelected`.
    pub model: Option<String>,
    /// The fixed diagnostic profile identifier (`diagnostic-basic-v1`).
    pub diagnostic_profile: &'static str,
    /// The execution-boundary string. Always `undetermined` in 1G.
    pub execution_boundary: &'static str,
    /// Ollama-reported prompt token count for the run. `None` when absent.
    pub prompt_eval_count: Option<u64>,
    /// Ollama-reported generated (output) token count for the run. `None` when
    /// absent.
    pub eval_count: Option<u64>,
    /// Ollama-reported model load duration, in nanoseconds. `None` when absent.
    pub load_duration_ns: Option<u64>,
    /// Ollama-reported prompt evaluation duration, in nanoseconds. `None` when
    /// absent.
    pub prompt_eval_duration_ns: Option<u64>,
    /// Ollama-reported generation (eval) duration, in nanoseconds. `None` when
    /// absent.
    pub eval_duration_ns: Option<u64>,
    /// Ollama-reported total duration, in nanoseconds. `None` when absent.
    pub total_duration_ns: Option<u64>,
    /// Engine-Room-calculated generation rate in tokens per second, from
    /// `eval_count` and `eval_duration_ns`. `None` unless the §8 guards pass
    /// (count present, duration present and > 0, finite result). Not
    /// Ollama-reported.
    pub generation_tokens_per_second: Option<f64>,
}

/// The controlled result of one local `POST /api/generate` probe, free of
/// transport types. A returned HTTP response reduces to `Responded { status,
/// body }`; transport failures reduce to the honest categories below. No raw
/// error, URL, header, hostname, or generated text crosses this boundary (the
/// `body` is the raw response body, parsed and reduced later in the pure layer,
/// and `response` is discarded there).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerateOutcome {
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

/// A transport that performs one local generate probe for a given model and
/// reduces the result to a controlled [`GenerateOutcome`]. `Send + Sync` so an
/// adapter can be shared across the Tauri async boundary. Implementations keep
/// all transport types (e.g. `reqwest`) confined to themselves.
pub trait GenerateProbe: Send + Sync {
    /// Perform one non-streaming generate request for `model` using the fixed
    /// diagnostic prompt/profile and return the controlled outcome.
    fn probe(&self, model: &str) -> impl Future<Output = GenerateOutcome> + Send;
}

/// Build the fixed diagnostic `/api/generate` request body for `model` as a JSON
/// string. Pure: no I/O. The body contains exactly `model`, `prompt`, `stream`,
/// and `options { num_predict, temperature }` — **no `seed`, no `keep_alive`,
/// no `system`, no `tools`, no `images`**. Exposed so the profile is testable
/// without a socket and so [`ReqwestGenerateProbe`] and tests share one builder.
pub fn diagnostic_request_body(model: &str) -> String {
    serde_json::json!({
        "model": model,
        "prompt": DIAGNOSTIC_PROMPT,
        "stream": STREAM,
        "options": {
            "num_predict": NUM_PREDICT,
            "temperature": TEMPERATURE,
        },
    })
    .to_string()
}

/// Calculate the Engine-Room generation rate in tokens per second from the
/// Ollama-reported output-token count and generation duration. Pure.
///
/// Formula (plan §8): `eval_count * 1_000_000_000 / eval_duration_ns`. Returns
/// `Some(rate)` only when `eval_count` is present, `eval_duration_ns` is present
/// and greater than zero, and the result is finite. Returns `None` otherwise
/// (absent count, absent/zero duration, non-finite result). This is
/// **ER-calculated**, not Ollama-reported.
pub fn calc_generation_rate(eval_count: Option<u64>, eval_duration_ns: Option<u64>) -> Option<f64> {
    let count = eval_count?;
    let duration = eval_duration_ns?;
    if duration == 0 {
        return None;
    }
    let rate = count as f64 * 1_000_000_000.0 / duration as f64;
    if rate.is_finite() {
        Some(rate)
    } else {
        None
    }
}

/// Map one generate-probe outcome to a controlled observed-inference result.
/// Pure: no I/O, no transport types. `requested_model` is carried through as the
/// targeted model (the response's own `model` field is only checked for
/// presence, never stored).
///
/// Mapping (plan §10/§14):
///
/// | `GenerateOutcome`                                              | `InferenceObservationState` |
/// |----------------------------------------------------------------|-----------------------------|
/// | `Responded { 200, done==true, model present, parseable }`      | `Completed` (with fields)   |
/// | `Responded { 200, missing/invalid done/model or malformed }`   | `RespondedUnexpectedly`     |
/// | `Responded { non-200, .. }`                                    | `RespondedUnexpectedly`     |
/// | `ConnectionRefused`                                            | `Unreachable`               |
/// | `Timeout`                                                      | `TimedOut`                  |
/// | `Other`                                                        | `Unreachable`               |
///
/// The `response` field is never read. Counts/durations are read as non-negative
/// integers (i64 to detect negatives, u64 for the full unsigned range);
/// absent/non-numeric/negative/float → `None`. Extra fields beyond the
/// documented contract are ignored. No locality is claimed; the execution
/// boundary is always `undetermined`.
pub fn interpret_inference(
    outcome: GenerateOutcome,
    requested_model: &str,
) -> InferenceObservation {
    match outcome {
        GenerateOutcome::Responded { status, body } => {
            // A non-200 is an unusable response, not a completed run.
            if status != 200 {
                return observation_for(
                    InferenceObservationState::RespondedUnexpectedly,
                    requested_model,
                );
            }

            let value: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => {
                    return observation_for(
                        InferenceObservationState::RespondedUnexpectedly,
                        requested_model,
                    );
                }
            };

            // A completed generation must report done == true and a non-empty
            // model. Missing/invalid done/model → responded unexpectedly.
            let done = value.get("done").and_then(|v| v.as_bool());
            if done != Some(true) {
                return observation_for(
                    InferenceObservationState::RespondedUnexpectedly,
                    requested_model,
                );
            }
            let resp_model = value.get("model").and_then(|v| v.as_str());
            if resp_model
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_none()
            {
                return observation_for(
                    InferenceObservationState::RespondedUnexpectedly,
                    requested_model,
                );
            }

            // The `response` field is deliberately never read — generated text
            // is discarded at this boundary. Only the documented counts/
            // durations (nanoseconds) are read.
            let prompt_eval_count = value.get("prompt_eval_count").and_then(int_to_u64);
            let eval_count = value.get("eval_count").and_then(int_to_u64);
            let load_duration_ns = value.get("load_duration").and_then(int_to_u64);
            let prompt_eval_duration_ns = value.get("prompt_eval_duration").and_then(int_to_u64);
            let eval_duration_ns = value.get("eval_duration").and_then(int_to_u64);
            let total_duration_ns = value.get("total_duration").and_then(int_to_u64);
            let generation_tokens_per_second = calc_generation_rate(eval_count, eval_duration_ns);

            InferenceObservation {
                state: InferenceObservationState::Completed,
                model: Some(requested_model.to_string()),
                diagnostic_profile: DIAGNOSTIC_PROFILE,
                execution_boundary: EXECUTION_BOUNDARY_UNDETERMINED,
                prompt_eval_count,
                eval_count,
                load_duration_ns,
                prompt_eval_duration_ns,
                eval_duration_ns,
                total_duration_ns,
                generation_tokens_per_second,
            }
        }
        // Per plan §14: connection refused → unreachable; timeout → timed_out;
        // other transport failure → unreachable. (NotDetected is produced by the
        // command-layer catalogue identity check, not here.)
        GenerateOutcome::ConnectionRefused => {
            observation_for(InferenceObservationState::Unreachable, requested_model)
        }
        GenerateOutcome::Timeout => {
            observation_for(InferenceObservationState::TimedOut, requested_model)
        }
        GenerateOutcome::Other => {
            observation_for(InferenceObservationState::Unreachable, requested_model)
        }
    }
}

/// Build a metric-free observation for a state produced without a completed
/// generate response (transport failures, unexpected responses, and the
/// command-layer non-POST states). The targeted model is carried when known.
fn observation_for(state: InferenceObservationState, model: &str) -> InferenceObservation {
    InferenceObservation {
        state,
        model: Some(model.to_string()),
        diagnostic_profile: DIAGNOSTIC_PROFILE,
        execution_boundary: EXECUTION_BOUNDARY_UNDETERMINED,
        prompt_eval_count: None,
        eval_count: None,
        load_duration_ns: None,
        prompt_eval_duration_ns: None,
        eval_duration_ns: None,
        total_duration_ns: None,
        generation_tokens_per_second: None,
    }
}

/// Read a JSON number as a non-negative `u64`. Accepts both integer-encoded and
/// unsigned-overflowing values; rejects negatives, non-numbers, and floats (the
/// documented count/duration fields are integers). Mirrors 1D/1E's helper.
fn int_to_u64(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_i64() {
        return if n >= 0 { Some(n as u64) } else { None };
    }
    value.as_u64()
}

/// Production transport: a `reqwest::Client` with bounded timeouts, used only
/// for the documented local non-streaming `POST /api/generate`. All `reqwest`
/// types stay in this struct and its `GenerateProbe` impl. The request body is
/// built with `serde_json` and POSTed as a string body with a `content-type`
/// header because `reqwest` is configured with `default-features = false` (the
/// `json` feature is unavailable; no Cargo change is made).
pub struct ReqwestGenerateProbe {
    client: reqwest::Client,
}

impl ReqwestGenerateProbe {
    /// Build a client with the bounded 1G timeouts (2 s connect, 120 s overall).
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

impl Default for ReqwestGenerateProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerateProbe for ReqwestGenerateProbe {
    fn probe(&self, model: &str) -> impl Future<Output = GenerateOutcome> + Send {
        let client = self.client.clone();
        let body = diagnostic_request_body(model);
        async move {
            match client
                .post(OLLAMA_GENERATE_ENDPOINT)
                .header(CONTENT_TYPE, "application/json")
                .body(body)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status().as_u16();
                    match response.text().await {
                        Ok(body) => GenerateOutcome::Responded { status, body },
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

/// Reduce a `reqwest` error to a controlled [`GenerateOutcome`]. No raw error
/// text leaves this function. Timeout is checked first; only an underlying
/// `std::io::ErrorKind::ConnectionRefused` maps to `ConnectionRefused`.
/// Everything else is `Other` — conservative, no fabricated certainty. Mirrors
/// 1C/1D/1E's `classify_reqwest_error`.
fn classify_reqwest_error(error: reqwest::Error) -> GenerateOutcome {
    if error.is_timeout() {
        return GenerateOutcome::Timeout;
    }

    if let Some(io_error) = find_source_io_error(&error) {
        if io_error.kind() == std::io::ErrorKind::ConnectionRefused {
            return GenerateOutcome::ConnectionRefused;
        }
    }

    GenerateOutcome::Other
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

// --- Controlled text (`ollama_inference_text`) --------------------------------

/// Controlled, Ollama-named display text for one observed inference. Deterministic
/// and application-layer (not `aer-core`, not Svelte): the frontend renders these
/// pre-computed strings and does not infer inference state from raw errors,
/// status codes, or transport kinds. The `limitations` field carries the
/// execution-boundary caveat ("Execution location not determined.") for every
/// state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InferenceObservationText {
    pub interpretation: String,
    pub why_it_matters: String,
    pub limitations: String,
}

/// Map an observed inference to controlled display text. No locality claim
/// (never "local"/"verified local"); no positive benchmark claim (the Completed
/// state explicitly says it is *not* a general benchmark); no raw errors, URLs,
/// hostnames, ports, status codes, generated text, or transport kinds. The
/// `limitations` field is the execution-boundary caveat for every state.
pub fn ollama_inference_text(observation: &InferenceObservation) -> InferenceObservationText {
    let limitations = EXECUTION_LOCATION_NOT_DETERMINED.to_string();
    match observation.state {
        InferenceObservationState::NoModelSelected => InferenceObservationText {
            interpretation: "No model is selected for an observed inference.".to_string(),
            why_it_matters:
                "Select a model from the catalogue to run an observation.".to_string(),
            limitations,
        },
        InferenceObservationState::BlockedByExecutionBoundary => InferenceObservationText {
            interpretation:
                "Engine Room will not run an observation without explicit authorization."
                    .to_string(),
            why_it_matters: "Engine Room cannot determine whether the model will run on this computer or through Ollama Cloud, so each observation requires you to authorize that one run.".to_string(),
            limitations,
        },
        InferenceObservationState::Running => InferenceObservationText {
            interpretation: "One observed inference is in progress.".to_string(),
            why_it_matters:
                "Engine Room issues at most one observation request. Controls are disabled until it finishes.".to_string(),
            limitations,
        },
        InferenceObservationState::Completed => InferenceObservationText {
            interpretation:
                "One observed inference completed. The counts and durations are values Ollama reported for this run.".to_string(),
            why_it_matters:
                "This is a single observed run, not a general benchmark or model-quality score.".to_string(),
            limitations,
        },
        InferenceObservationState::NotDetected => InferenceObservationText {
            interpretation:
                "Ollama was not detected, so the observation could not run.".to_string(),
            why_it_matters:
                "Nothing answered the Ollama endpoint. This only shows that nothing answered right now.".to_string(),
            limitations,
        },
        InferenceObservationState::Unreachable => InferenceObservationText {
            interpretation:
                "Ollama could not be reached, so the observation could not run.".to_string(),
            why_it_matters:
                "The Ollama endpoint did not respond in time. It may be unreachable or too slow to answer right now.".to_string(),
            limitations,
        },
        InferenceObservationState::TimedOut => InferenceObservationText {
            interpretation: "The observation did not finish within the time limit.".to_string(),
            why_it_matters:
                "The inference request took longer than the allowed time. No retry is attempted automatically.".to_string(),
            limitations,
        },
        InferenceObservationState::RespondedUnexpectedly => InferenceObservationText {
            interpretation:
                "Ollama responded in a way Engine Room could not use for an observation.".to_string(),
            why_it_matters:
                "The endpoint answered, but the response did not include the expected completion information.".to_string(),
            limitations,
        },
        InferenceObservationState::ModelUnavailable => InferenceObservationText {
            interpretation:
                "The selected model was not found in the Ollama catalogue.".to_string(),
            why_it_matters:
                "Engine Room only runs an observation for a model Ollama currently lists. Select a listed model.".to_string(),
            limitations,
        },
    }
}

/// Whether an inventory state is one the command layer treats as "catalogue not
/// usable, do not POST". Returns the [`InferenceObservationState`] to report in
/// that case, or `None` when the catalogue is `Available` (the caller then
/// checks whether the selected model is present). Used by the command layer;
/// kept here so the state mapping lives with the inference model.
pub fn catalogue_unavailable_state(state: InventoryState) -> Option<InferenceObservationState> {
    match state {
        InventoryState::NotDetected => Some(InferenceObservationState::NotDetected),
        InventoryState::Unreachable => Some(InferenceObservationState::Unreachable),
        InventoryState::RespondedUnexpectedly => {
            Some(InferenceObservationState::RespondedUnexpectedly)
        }
        InventoryState::Available => None,
    }
}

// --- Command-layer orchestration: authorization, identity, concurrency -------
//
// `run_inference_observation_with_adapter` is the application-layer policy that
// sits between the IPC command and the single generate POST. It enforces, in
// order: a defensive model-identifier check, the per-run execution-boundary
// authorization gate, a one-GET catalogue identity/availability check, and a
// process-global single-run guard. Only after all four pass does it issue one
// generate POST (no retry). Every failure path returns a controlled
// [`InferenceObservation`] and issues NO POST. The generated `response` text is
// discarded by [`interpret_inference`]; nothing here reads it.

/// Process-global single-run guard for observed inference. `false` when no
/// observation is in progress. Held for the duration of one generate POST only;
/// released on completion, error, or panic via the [`InferenceGuard`] drop impl.
/// No persisted lock state — the flag is the whole of the lock.
static INFERENCING: AtomicBool = AtomicBool::new(false);

/// RAII guard that releases the [`INFERENCING`] flag on drop (including unwind
/// from an error or panic mid-run). Acquired through [`InferenceGuard::try_acquire`].
pub(crate) struct InferenceGuard;

impl InferenceGuard {
    /// Acquire the single-run guard. Returns `Some` if it was free, `None` if an
    /// observation is already in progress (the caller reports `Running` and
    /// issues no second POST).
    pub(crate) fn try_acquire() -> Option<Self> {
        match INFERENCING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => Some(Self),
            Err(_) => None,
        }
    }
}

impl Drop for InferenceGuard {
    fn drop(&mut self) {
        INFERENCING.store(false, Ordering::SeqCst);
    }
}

/// A usable model identifier: non-empty after trimming and free of control
/// characters. Defensive validation of the request's `model` field — the UI
/// selects from the 1D catalogue, but the backend rejects unacknowledged direct
/// IPC that carries an empty or control-laden identifier. Returns `false` for a
/// blank or whitespace-only string; `true` for a normal Ollama tag like
/// `"example:1b"`. Not a locality, format, or existence check.
fn is_valid_model_identifier(model: &str) -> bool {
    !model.trim().is_empty() && !model.chars().any(char::is_control)
}

/// The observation for the no-model-selected state: no targeted model, no
/// metrics. Distinct from [`observation_for`] (which always carries a model).
fn no_model_selected_observation() -> InferenceObservation {
    InferenceObservation {
        state: InferenceObservationState::NoModelSelected,
        model: None,
        diagnostic_profile: DIAGNOSTIC_PROFILE,
        execution_boundary: EXECUTION_BOUNDARY_UNDETERMINED,
        prompt_eval_count: None,
        eval_count: None,
        load_duration_ns: None,
        prompt_eval_duration_ns: None,
        eval_duration_ns: None,
        total_duration_ns: None,
        generation_tokens_per_second: None,
    }
}

/// Orchestrate one observed-inference run against an adapter. Application-layer
/// policy only — no `aer-core`, no `AiRuntimeAdapter` trait change. The steps run
/// in a fixed order, and every failure returns a controlled observation with NO
/// generate POST:
///
/// 1. **Model-identifier validation.** An empty/control-laden `model` yields
///    `NoModelSelected` (no model carried, no POST).
/// 2. **Per-run authorization gate.** `execution_boundary_acknowledged == false`
///    yields `BlockedByExecutionBoundary` (no POST). `true` is **not locality
///    proof and not cryptographic** — it means the user acknowledged the
///    undetermined execution boundary and authorized one synthetic observation.
///    Request-scoped: no global/persistent auth state, no allow-list, no session
///    token, no "Don't ask again".
/// 3. **Catalogue identity/availability validation** — one `GET /api/tags` via
///    `adapter.inventory()`. A catalogue that is not `Available` yields the
///    matching controlled state (`NotDetected`/`Unreachable`/
///    `RespondedUnexpectedly`) with no POST. A catalogue that is `Available` but
///    does not list the requested model yields `ModelUnavailable` with no POST.
///    This is identity/availability ONLY — NOT a locality determination.
/// 4. **Single-run concurrency guard.** If [`INFERENCING`] is already held,
///    yields `Running` with no second POST. Otherwise the guard is acquired and
///    held for the duration of the POST, releasing on drop (completion, error,
///    or panic).
/// 5. **One generate POST** via `adapter.inference_observation(model)` — no
///    retry, no warm-up, no `keep_alive` change. The generated `response` is
///    discarded by [`interpret_inference`].
pub async fn run_inference_observation_with_adapter<P, M, R, G>(
    adapter: &OllamaAdapter<P, M, R, G>,
    model: &str,
    execution_boundary_acknowledged: bool,
) -> InferenceObservation
where
    P: VersionProbe,
    M: ModelsProbe,
    R: RunningProbe,
    G: GenerateProbe,
{
    // 1. Defensive model-identifier validation (no POST).
    if !is_valid_model_identifier(model) {
        return no_model_selected_observation();
    }

    // 2. Per-run authorization gate. `false` → NO POST. `true` authorizes one
    //    run only and is not a locality signal.
    if !execution_boundary_acknowledged {
        return observation_for(InferenceObservationState::BlockedByExecutionBoundary, model);
    }

    // 3. Catalogue identity/availability validation: one GET /api/tags. NOT a
    //    locality determination. A non-Available catalogue yields its controlled
    //    state with no POST.
    let inventory = adapter.inventory().await;
    if let Some(unavailable_state) = catalogue_unavailable_state(inventory.state) {
        return observation_for(unavailable_state, model);
    }
    // inventory.state == Available here: confirm the model is currently listed.
    let present = inventory.models.iter().any(|m| m.display_name == model);
    if !present {
        return observation_for(InferenceObservationState::ModelUnavailable, model);
    }

    // 4. Single-run concurrency guard. If held, report Running with no second
    //    POST. The guard releases when `_guard` drops at the end of this
    //    function (and on any error/panic before then).
    let _guard = match InferenceGuard::try_acquire() {
        Some(guard) => guard,
        None => return observation_for(InferenceObservationState::Running, model),
    };

    // 5. One generate POST (no retry, no warm-up, no keep_alive change). The
    //    generated `response` text is discarded by `interpret_inference`.
    adapter.inference_observation(model).await
}

#[cfg(test)]
mod tests {
    //! Deterministic coverage of `interpret_inference`, the rate calculation,
    //! the fixed diagnostic profile, the controlled disclosure/rate-label
    //! constants, controlled per-state text, and one ephemeral-loopback
    //! transport check. No live Ollama. Orchestration coverage uses a
    //! `FakeGenerateProbe`.

    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    use super::*;

    fn responded(status: u16, body: &str) -> GenerateOutcome {
        GenerateOutcome::Responded {
            status,
            body: body.to_string(),
        }
    }

    fn interpret(outcome: GenerateOutcome) -> InferenceObservation {
        interpret_inference(outcome, "example:1b")
    }

    fn request_is_complete(bytes: &[u8]) -> bool {
        let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
            return false;
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).expect("request headers are utf-8");
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length").then(|| {
                    value
                        .trim()
                        .parse::<usize>()
                        .expect("numeric content length")
                })
            })
            .unwrap_or(0);
        bytes.len() >= header_end + 4 + content_length
    }

    fn read_request(stream: &mut TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("bound source read");
        let mut bytes = Vec::new();
        while !request_is_complete(&bytes) {
            let mut chunk = [0_u8; 1024];
            let count = stream.read(&mut chunk).expect("read source request");
            assert_ne!(count, 0, "request ended before its body was complete");
            bytes.extend_from_slice(&chunk[..count]);
        }
        String::from_utf8(bytes).expect("request is utf-8")
    }

    fn redirected_post(
        client: reqwest::Client,
        request_body: String,
    ) -> (u16, String, String, bool, bool) {
        let target = TcpListener::bind(("127.0.0.1", 0)).expect("bind redirect target");
        target
            .set_nonblocking(true)
            .expect("make redirect target nonblocking");
        let location = format!("http://{}/target", target.local_addr().unwrap());

        let source = TcpListener::bind(("127.0.0.1", 0)).expect("bind redirect source");
        let source_url = format!("http://{}/source", source.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = source.accept().expect("accept source request");
            let request = read_request(&mut stream);
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

        let response = tauri::async_runtime::block_on(async {
            client
                .post(source_url)
                .header(CONTENT_TYPE, "application/json")
                .body(request_body)
                .send()
                .await
        })
        .expect("receive original redirect response");
        let status = response.status().as_u16();
        let body = tauri::async_runtime::block_on(response.text()).expect("read redirect body");
        let (request, extra_source_request) = server.join().expect("join redirect source");
        let target_request = target.accept().is_ok();
        (status, body, request, extra_source_request, target_request)
    }

    // --- Transport / probe (`interpret_inference`) ----------------------------

    #[test]
    fn inference_client_rejects_redirect_without_retry_or_target_request() {
        let request_body = diagnostic_request_body("example:1b");
        let (status, body, request, extra_source_request, target_request) =
            redirected_post(ReqwestGenerateProbe::new().client, request_body.clone());

        assert_eq!(status, 307);
        assert_eq!(body, "redirect");
        assert!(request.starts_with("POST /source HTTP/1.1\r\n"));
        assert!(request.ends_with(&request_body));
        assert!(
            !extra_source_request,
            "source must receive exactly one request"
        );
        assert!(
            !target_request,
            "redirect target must not receive a request"
        );
        assert_eq!(
            interpret(responded(status, &body)).state,
            InferenceObservationState::RespondedUnexpectedly
        );
    }

    #[test]
    fn success_complete_maps_to_completed_with_all_fields() {
        let body = r#"{
            "model": "example:1b",
            "response": "Leaves look green because they reflect green light.",
            "done": true,
            "done_reason": "stop",
            "total_duration": 3000000000,
            "load_duration": 1000000000,
            "prompt_eval_count": 12,
            "prompt_eval_duration": 500000000,
            "eval_count": 20,
            "eval_duration": 1500000000
        }"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert_eq!(obs.model.as_deref(), Some("example:1b"));
        assert_eq!(obs.diagnostic_profile, DIAGNOSTIC_PROFILE);
        assert_eq!(obs.execution_boundary, EXECUTION_BOUNDARY_UNDETERMINED);
        assert_eq!(obs.prompt_eval_count, Some(12));
        assert_eq!(obs.eval_count, Some(20));
        assert_eq!(obs.load_duration_ns, Some(1_000_000_000));
        assert_eq!(obs.prompt_eval_duration_ns, Some(500_000_000));
        assert_eq!(obs.eval_duration_ns, Some(1_500_000_000));
        assert_eq!(obs.total_duration_ns, Some(3_000_000_000));
        // Rate = 20 * 1e9 / 1.5e9 = 13.333... tokens/s.
        let rate = obs.generation_tokens_per_second.expect("rate present");
        assert!((rate - (20.0 * 1_000_000_000.0 / 1_500_000_000.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn success_with_missing_optional_fields_is_completed_with_nones() {
        let body = r#"{"model": "example:1b", "done": true}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert!(obs.prompt_eval_count.is_none());
        assert!(obs.eval_count.is_none());
        assert!(obs.load_duration_ns.is_none());
        assert!(obs.prompt_eval_duration_ns.is_none());
        assert!(obs.eval_duration_ns.is_none());
        assert!(obs.total_duration_ns.is_none());
        assert!(obs.generation_tokens_per_second.is_none());
    }

    #[test]
    fn response_field_is_discarded_and_never_carried() {
        // The generated `response` text must not surface anywhere on the
        // observation. The struct has no field for it; this asserts no string
        // field carries the secret text.
        let body = r#"{"model": "example:1b", "response": "SECRET-GENERATED-OUTPUT", "done": true, "eval_count": 1, "eval_duration": 1}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        // The only string fields are model, diagnostic_profile, execution_boundary.
        for field in [
            obs.model.as_deref().unwrap_or(""),
            obs.diagnostic_profile,
            obs.execution_boundary,
        ] {
            assert!(
                !field.contains("SECRET-GENERATED-OUTPUT"),
                "generated text leaked: {field:?}"
            );
        }
        // And the requested model is echoed, not the generated text.
        assert_eq!(obs.model.as_deref(), Some("example:1b"));
    }

    #[test]
    fn done_false_is_responded_unexpectedly() {
        let body = r#"{"model": "example:1b", "done": false}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
        assert!(obs.eval_count.is_none());
    }

    #[test]
    fn done_missing_is_responded_unexpectedly() {
        let body = r#"{"model": "example:1b"}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
    }

    #[test]
    fn model_missing_is_responded_unexpectedly() {
        let body = r#"{"done": true, "eval_count": 1, "eval_duration": 1}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
    }

    #[test]
    fn model_whitespace_only_is_responded_unexpectedly() {
        let body = r#"{"model": "   ", "done": true}"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
    }

    #[test]
    fn malformed_json_is_responded_unexpectedly() {
        let obs = interpret(responded(200, "not-json"));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
    }

    #[test]
    fn non_200_status_is_responded_unexpectedly() {
        let obs = interpret(responded(500, r#"{"model": "example:1b", "done": true}"#));
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
        assert!(obs.eval_count.is_none());
    }

    #[test]
    fn connection_refused_is_unreachable() {
        let obs = interpret(GenerateOutcome::ConnectionRefused);
        assert_eq!(obs.state, InferenceObservationState::Unreachable);
        assert!(obs.eval_count.is_none());
        assert_eq!(obs.model.as_deref(), Some("example:1b"));
    }

    #[test]
    fn timeout_is_timed_out() {
        let obs = interpret(GenerateOutcome::Timeout);
        assert_eq!(obs.state, InferenceObservationState::TimedOut);
        assert!(obs.eval_count.is_none());
    }

    #[test]
    fn other_transport_failure_is_unreachable() {
        let obs = interpret(GenerateOutcome::Other);
        assert_eq!(obs.state, InferenceObservationState::Unreachable);
    }

    #[test]
    fn counts_and_durations_reject_negatives_floats_and_non_numbers() {
        let body = r#"{
            "model": "example:1b",
            "done": true,
            "eval_count": -3,
            "eval_duration": 1.5,
            "total_duration": "big",
            "prompt_eval_count": 5
        }"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert!(obs.eval_count.is_none(), "negative eval_count -> None");
        assert!(
            obs.eval_duration_ns.is_none(),
            "float eval_duration -> None"
        );
        assert!(
            obs.total_duration_ns.is_none(),
            "non-numeric total_duration -> None"
        );
        assert_eq!(obs.prompt_eval_count, Some(5));
        // No rate without a valid eval_count/eval_duration.
        assert!(obs.generation_tokens_per_second.is_none());
    }

    #[test]
    fn large_unsigned_duration_is_preserved() {
        let big: u64 = 9_000_000_000;
        let body = format!(
            r#"{{"model": "example:1b", "done": true, "eval_count": 10, "eval_duration": {big}}}"#
        );
        let obs = interpret(responded(200, &body));
        assert_eq!(obs.eval_duration_ns, Some(big));
    }

    #[test]
    fn undocumented_extra_fields_are_ignored() {
        // done_reason, context, other undocumented fields must not affect the
        // mapping. The `response` field is ignored entirely.
        let body = r#"{
            "model": "example:1b",
            "response": "ignored",
            "done": true,
            "done_reason": "stop",
            "context": [1, 2, 3],
            "eval_count": 8,
            "eval_duration": 2000000000
        }"#;
        let obs = interpret(responded(200, body));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert_eq!(obs.eval_count, Some(8));
        assert_eq!(obs.eval_duration_ns, Some(2_000_000_000));
    }

    #[test]
    fn interpret_inference_is_pure_and_carries_requested_model() {
        let a = interpret(responded(200, r#"{"model": "example:1b", "done": true}"#));
        let b = interpret(responded(200, r#"{"model": "example:1b", "done": true}"#));
        assert_eq!(a, b);
        assert_eq!(a.model.as_deref(), Some("example:1b"));
        // The response's own model field is only checked for presence; the
        // echoed model is the requested one even if the response names another.
        let other = interpret_inference(
            responded(200, r#"{"model": "different:9b", "done": true}"#),
            "requested:1b",
        );
        assert_eq!(other.model.as_deref(), Some("requested:1b"));
    }

    // --- Rate calculation (`calc_generation_rate`) ----------------------------

    #[test]
    fn rate_is_count_times_1e9_over_duration_ns() {
        let rate = calc_generation_rate(Some(20), Some(1_500_000_000)).unwrap();
        assert!((rate - 13.333_333_333_333_334).abs() < 1e-6);
    }

    #[test]
    fn rate_is_none_when_eval_count_absent() {
        assert!(calc_generation_rate(None, Some(1_500_000_000)).is_none());
    }

    #[test]
    fn rate_is_none_when_eval_duration_absent() {
        assert!(calc_generation_rate(Some(20), None).is_none());
    }

    #[test]
    fn rate_is_none_when_eval_duration_is_zero() {
        assert!(calc_generation_rate(Some(20), Some(0)).is_none());
    }

    #[test]
    fn rate_handles_eval_count_zero() {
        // Pinned: a zero output-token count is a valid observed run and yields a
        // rate of 0 (not None) when a non-zero duration is present.
        let rate = calc_generation_rate(Some(0), Some(1_000_000_000)).unwrap();
        assert!((rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rate_is_finite_for_realistic_values() {
        let rate = calc_generation_rate(Some(64), Some(10_000_000_000)).unwrap();
        assert!(rate.is_finite());
        assert!((rate - 6.4).abs() < 1e-6);
    }

    // --- Profile / fixed request body -----------------------------------------

    #[test]
    fn diagnostic_prompt_is_the_fixed_synthetic_sentence() {
        assert_eq!(
            DIAGNOSTIC_PROMPT,
            "In one short sentence, explain why leaves appear green."
        );
    }

    #[test]
    fn diagnostic_profile_is_diagnostic_basic_v1() {
        assert_eq!(DIAGNOSTIC_PROFILE, "diagnostic-basic-v1");
    }

    #[test]
    fn profile_constants_are_the_fixed_values() {
        assert_eq!(NUM_PREDICT, 64);
        assert_eq!(TEMPERATURE, 0.0);
        // STREAM is pinned at compile time by the `const _: () = assert!(!STREAM)`
        // item above, and at runtime by the request-body test below.
    }

    #[test]
    fn generate_endpoint_is_documented_loopback() {
        assert_eq!(
            OLLAMA_GENERATE_ENDPOINT,
            "http://127.0.0.1:11434/api/generate"
        );
    }

    #[test]
    fn request_body_contains_only_the_fixed_fields() {
        let body = diagnostic_request_body("example:1b");
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["model"], "example:1b");
        assert_eq!(json["prompt"], DIAGNOSTIC_PROMPT);
        assert_eq!(json["stream"], false);
        assert_eq!(json["options"]["num_predict"], 64);
        // temperature serialises as a float (0.0).
        assert_eq!(json["options"]["temperature"], 0.0);
        // Forbidden fields: no seed, no keep_alive, no system, no tools, no
        // images, no raw.
        assert!(json.get("seed").is_none(), "no seed");
        assert!(json.get("keep_alive").is_none(), "no keep_alive");
        assert!(json.get("system").is_none(), "no system");
        assert!(json.get("tools").is_none(), "no tools");
        assert!(json.get("images").is_none(), "no images");
        // Only the four top-level keys model/prompt/stream/options.
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 4, "exactly four top-level fields");
        let options = json["options"].as_object().unwrap();
        assert_eq!(options.len(), 2, "exactly two options");
    }

    #[test]
    fn request_body_carries_no_user_or_machine_data() {
        // The prompt is synthetic and fixed; the body introduces no user files,
        // repository content, or machine information regardless of the model.
        let body = diagnostic_request_body("anything:1b");
        let lower = body.to_lowercase();
        for forbidden in [
            "home/", "/etc", "user", "password", "token", "secret", "path", "email",
        ] {
            assert!(
                !lower.contains(forbidden),
                "request body contains {forbidden:?}"
            );
        }
    }

    // --- Disclosure + rate-label constants ------------------------------------

    #[test]
    fn disclosure_title_is_run_an_observed_inference() {
        assert_eq!(DISCLOSURE_TITLE, "Run an observed inference?");
    }

    #[test]
    fn disclosure_says_execution_location_cannot_be_determined() {
        let combined = DISCLOSURE_BODY.join(" ").to_lowercase();
        assert!(combined.contains("cannot determine"));
        assert!(combined
            .contains("whether this model will run on this computer or through ollama cloud"));
    }

    #[test]
    fn disclosure_mentions_ollama_cloud() {
        let combined = DISCLOSURE_BODY.join(" ").to_lowercase();
        assert!(combined.contains("ollama cloud"));
    }

    #[test]
    fn disclosure_states_prompt_contains_no_user_repo_or_machine_data() {
        let combined = DISCLOSURE_BODY.join(" ").to_lowercase();
        assert!(combined.contains("no user files"));
        assert!(combined.contains("repository content"));
        assert!(combined.contains("machine information"));
    }

    #[test]
    fn disclosure_states_one_run_only_authorization() {
        let combined = DISCLOSURE_BODY.join(" ");
        assert!(combined.contains("one run only"));
        assert!(combined.contains("another run requires another authorization"));
    }

    #[test]
    fn disclosure_actions_are_run_observation_and_cancel() {
        assert_eq!(DISCLOSURE_PRIMARY_ACTION, "Run observation");
        assert_eq!(DISCLOSURE_SECONDARY_ACTION, "Cancel");
    }

    #[test]
    fn disclosure_has_no_dont_ask_again() {
        for text in std::iter::once(DISCLOSURE_TITLE)
            .chain(DISCLOSURE_BODY.iter().copied())
            .chain([DISCLOSURE_PRIMARY_ACTION, DISCLOSURE_SECONDARY_ACTION])
        {
            assert!(
                !text.to_lowercase().contains("don't ask again"),
                "no 'Don't ask again' in disclosure: {text:?}"
            );
            assert!(
                !text.to_lowercase().contains("dont ask again"),
                "no 'Dont ask again' in disclosure: {text:?}"
            );
        }
    }

    #[test]
    fn disclosure_makes_no_locality_or_install_claim() {
        for text in std::iter::once(DISCLOSURE_TITLE)
            .chain(DISCLOSURE_BODY.iter().copied())
            .chain([DISCLOSURE_PRIMARY_ACTION, DISCLOSURE_SECONDARY_ACTION])
        {
            let lower = text.to_lowercase();
            assert!(
                !lower.contains("local model"),
                "no local-model claim: {text:?}"
            );
            assert!(
                !lower.contains("runs locally"),
                "no runs-locally claim: {text:?}"
            );
            assert!(!lower.contains("install"), "no install claim: {text:?}");
        }
    }

    #[test]
    fn rate_label_and_explanation_are_controlled() {
        assert_eq!(GENERATION_RATE_LABEL, "Observed generation rate");
        assert_eq!(
            GENERATION_RATE_EXPLANATION,
            "Calculated from the output-token count and generation duration Ollama reported for this run."
        );
        // The label does not claim Ollama reported the rate.
        assert!(
            !GENERATION_RATE_LABEL.to_lowercase().contains("ollama"),
            "rate label is ER-named, not Ollama-reported"
        );
    }

    #[test]
    fn execution_boundary_constant_is_undetermined() {
        assert_eq!(EXECUTION_BOUNDARY_UNDETERMINED, "undetermined");
    }

    // --- Controlled text (`ollama_inference_text`) ----------------------------

    fn text_for(state: InferenceObservationState) -> InferenceObservationText {
        let obs = observation_for(state, "example:1b");
        ollama_inference_text(&obs)
    }

    fn all_states() -> Vec<InferenceObservationState> {
        vec![
            InferenceObservationState::NoModelSelected,
            InferenceObservationState::BlockedByExecutionBoundary,
            InferenceObservationState::Running,
            InferenceObservationState::Completed,
            InferenceObservationState::NotDetected,
            InferenceObservationState::Unreachable,
            InferenceObservationState::TimedOut,
            InferenceObservationState::RespondedUnexpectedly,
            InferenceObservationState::ModelUnavailable,
        ]
    }

    #[test]
    fn every_state_has_non_empty_text() {
        for state in all_states() {
            let t = text_for(state);
            assert!(!t.interpretation.is_empty(), "interpretation for {state:?}");
            assert!(!t.why_it_matters.is_empty(), "why_it_matters for {state:?}");
            assert!(!t.limitations.is_empty(), "limitations for {state:?}");
        }
    }

    #[test]
    fn every_state_carries_execution_location_not_determined_limitation() {
        for state in all_states() {
            let t = text_for(state);
            assert_eq!(t.limitations, EXECUTION_LOCATION_NOT_DETERMINED);
        }
    }

    #[test]
    fn completed_text_disclaims_being_a_benchmark() {
        let t = text_for(InferenceObservationState::Completed);
        let combined = format!("{} {}", t.interpretation, t.why_it_matters).to_lowercase();
        assert!(combined.contains("not a general benchmark"));
        assert!(combined.contains("model-quality score"));
    }

    #[test]
    fn blocked_text_explains_authorization_is_required() {
        let t = text_for(InferenceObservationState::BlockedByExecutionBoundary);
        let combined = format!("{} {}", t.interpretation, t.why_it_matters).to_lowercase();
        assert!(combined.contains("authorization"));
        assert!(combined.contains("cannot determine"));
    }

    #[test]
    fn no_state_claims_locality_verified_or_installs() {
        for state in all_states() {
            let t = text_for(state);
            for field in [
                t.interpretation.as_str(),
                t.why_it_matters.as_str(),
                t.limitations.as_str(),
            ] {
                let lower = field.to_lowercase();
                assert!(
                    !lower.contains("local"),
                    "no 'local' in {state:?}: {field:?}"
                );
                assert!(
                    !lower.contains("verified"),
                    "no 'verified' in {state:?}: {field:?}"
                );
                assert!(
                    !lower.contains("install"),
                    "no 'install' in {state:?}: {field:?}"
                );
                // A positive benchmark claim is forbidden; the Completed
                // disclaimer ("not a general benchmark") is the only allowed use.
                if lower.contains("benchmark") {
                    assert!(
                        lower.contains("not a general benchmark"),
                        "only the disclaimer may mention benchmark ({state:?}): {field:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn no_state_leaks_raw_transport_endpoint_or_generated_detail() {
        for state in all_states() {
            let t = text_for(state);
            for field in [
                t.interpretation.as_str(),
                t.why_it_matters.as_str(),
                t.limitations.as_str(),
            ] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
                assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
                assert!(!lower.contains("http"), "no scheme: {field:?}");
                assert!(!lower.contains("404"), "no status code: {field:?}");
                assert!(!lower.contains("refus"), "no transport word: {field:?}");
                assert!(!lower.contains("timeout"), "no transport detail: {field:?}");
                assert!(!lower.contains("digest"), "no digest: {field:?}");
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
    }

    #[test]
    fn no_state_makes_health_security_or_performance_claims() {
        for state in all_states() {
            let t = text_for(state);
            for field in [
                t.interpretation.as_str(),
                t.why_it_matters.as_str(),
                t.limitations.as_str(),
            ] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("health"), "no health claim: {field:?}");
                assert!(!lower.contains("secure"), "no security claim: {field:?}");
                assert!(!lower.contains("fast"), "no performance claim: {field:?}");
                assert!(
                    !lower.contains("perform"),
                    "no performance claim: {field:?}"
                );
            }
        }
    }

    #[test]
    fn states_have_distinct_interpretations() {
        let interpretations: Vec<String> = all_states()
            .iter()
            .map(|s| text_for(*s).interpretation)
            .collect();
        for i in 0..interpretations.len() {
            for j in (i + 1)..interpretations.len() {
                assert_ne!(
                    interpretations[i], interpretations[j],
                    "interpretations {i} and {j} collide"
                );
            }
        }
    }

    // --- Catalogue-unavailable mapping ----------------------------------------

    #[test]
    fn catalogue_unavailable_state_maps_inventory_states() {
        use crate::runtime::inventory::InventoryState;
        assert_eq!(catalogue_unavailable_state(InventoryState::Available), None);
        assert_eq!(
            catalogue_unavailable_state(InventoryState::NotDetected),
            Some(InferenceObservationState::NotDetected)
        );
        assert_eq!(
            catalogue_unavailable_state(InventoryState::Unreachable),
            Some(InferenceObservationState::Unreachable)
        );
        assert_eq!(
            catalogue_unavailable_state(InventoryState::RespondedUnexpectedly),
            Some(InferenceObservationState::RespondedUnexpectedly)
        );
    }

    // --- reqwest error classification (mirrors 1C/1D/1E) ----------------------

    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct WrappedError(std::io::Error);

    impl fmt::Display for WrappedError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "wrapped I/O error")
        }
    }

    impl Error for WrappedError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn finds_connection_refused_in_error_source_chain() {
        let error = WrappedError(std::io::Error::from(std::io::ErrorKind::ConnectionRefused));
        assert_eq!(
            find_source_io_error(&error).map(std::io::Error::kind),
            Some(std::io::ErrorKind::ConnectionRefused)
        );
    }

    #[test]
    fn returns_none_when_error_source_chain_has_no_io_error() {
        assert!(find_source_io_error(&fmt::Error).is_none());
    }

    // --- Adapter wiring with a FakeGenerateProbe ------------------------------
    //
    // `OllamaAdapter::inference_observation(model)` must reduce a fixture probe
    // through the pure `interpret_inference`, exactly as the 1C/1D/1E adapter
    // methods wrap their pure mappings.

    use crate::runtime::inventory::ModelsOutcome;
    use crate::runtime::ollama::OllamaAdapter;
    use crate::runtime::probe::ProbeOutcome;

    /// A test generate transport that returns a fixed [`GenerateOutcome`]. No I/O.
    struct FakeGenerateProbe {
        outcome: GenerateOutcome,
    }

    impl GenerateProbe for FakeGenerateProbe {
        fn probe(&self, _model: &str) -> impl Future<Output = GenerateOutcome> + Send {
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    fn adapter_with_generate(
        outcome: GenerateOutcome,
    ) -> OllamaAdapter<
        crate::runtime::ollama::ReqwestProbe,
        crate::runtime::inventory::ReqwestModelsProbe,
        crate::runtime::loaded_models::ReqwestRunningProbe,
        FakeGenerateProbe,
    > {
        OllamaAdapter::with_all_probes_and_generate(
            crate::runtime::ollama::ReqwestProbe::new(),
            crate::runtime::inventory::ReqwestModelsProbe::new(),
            crate::runtime::loaded_models::ReqwestRunningProbe::new(),
            FakeGenerateProbe { outcome },
        )
    }

    #[test]
    fn inference_observation_maps_a_valid_response_through_the_adapter() {
        let body = r#"{"model": "example:1b", "response": "ignored", "done": true, "eval_count": 10, "eval_duration": 1000000000}"#;
        let a = adapter_with_generate(GenerateOutcome::Responded {
            status: 200,
            body: body.to_string(),
        });
        let obs = tauri::async_runtime::block_on(a.inference_observation("example:1b"));
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert_eq!(obs.eval_count, Some(10));
        assert_eq!(obs.eval_duration_ns, Some(1_000_000_000));
    }

    #[test]
    fn inference_observation_is_consistent_with_pure_interpret_for_a_fixture_probe() {
        let outcome = GenerateOutcome::Responded {
            status: 200,
            body: r#"{"model": "example:1b", "done": true, "eval_count": 5, "eval_duration": 500000000}"#.to_string(),
        };
        let a = adapter_with_generate(outcome.clone());
        let via_adapter = tauri::async_runtime::block_on(a.inference_observation("example:1b"));
        let via_pure = interpret_inference(outcome, "example:1b");
        assert_eq!(via_adapter, via_pure);
    }

    #[test]
    fn inference_observation_reports_unreachable_on_connection_refused() {
        let a = adapter_with_generate(GenerateOutcome::ConnectionRefused);
        let obs = tauri::async_runtime::block_on(a.inference_observation("example:1b"));
        assert_eq!(obs.state, InferenceObservationState::Unreachable);
    }

    #[test]
    fn inference_observation_reports_timed_out_on_timeout() {
        let a = adapter_with_generate(GenerateOutcome::Timeout);
        let obs = tauri::async_runtime::block_on(a.inference_observation("example:1b"));
        assert_eq!(obs.state, InferenceObservationState::TimedOut);
    }

    // Unused import guard: keep the 1C/1D outcome types reachable so the
    // adapter constructor's other generics stay exercised by compile.
    #[test]
    fn adapter_generate_constructor_compiles_with_default_other_probes() {
        let _ = ProbeOutcome::ConnectionRefused;
        let _ = ModelsOutcome::ConnectionRefused;
        let a = adapter_with_generate(GenerateOutcome::Other);
        let obs = tauri::async_runtime::block_on(a.inference_observation("example:1b"));
        assert_eq!(obs.state, InferenceObservationState::Unreachable);
    }

    // --- Command-layer orchestration (§14 authorization / identity / ----------
    //     concurrency) -------------------------------------------------------
    //
    // `run_inference_observation_with_adapter` enforces the fixed order
    // validate → authorize → catalogue identity → concurrency guard → one POST.
    // A counting generate probe proves NO POST on the early failure paths and
    // exactly one POST on the happy path; a counting models probe proves the
    // identity check is one GET and runs only after the authorization gate. No
    // socket, no reqwest, no live Ollama.

    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Mutex};

    use crate::runtime::inventory::ModelsProbe;
    use crate::runtime::loaded_models::RunningProbe;
    use crate::runtime::probe::VersionProbe;

    /// Serializes the tests that touch the process-global [`INFERENCING`] guard
    /// so the single-run flag cannot race between parallel test threads. Tests
    /// that return before the guard (no-model, blocked, catalogue-unavailable,
    /// model-unavailable) never touch it and need no lock.
    static GUARD_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// A test models transport that returns a fixed [`ModelsOutcome`] and counts
    /// `fetch` calls, so tests can assert the identity check is one GET (and that
    /// it does not run before the authorization gate). No I/O.
    struct FakeModelsProbe {
        outcome: ModelsOutcome,
        calls: Arc<AtomicUsize>,
    }

    impl ModelsProbe for FakeModelsProbe {
        fn fetch(&self) -> impl Future<Output = ModelsOutcome> + Send {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    /// A test generate transport that returns a fixed [`GenerateOutcome`] and
    /// counts `probe` calls, so tests can assert "no POST" (calls == 0) or
    /// "exactly one POST" (calls == 1). No I/O.
    struct CountingGenerateProbe {
        outcome: GenerateOutcome,
        calls: Arc<AtomicUsize>,
    }

    impl GenerateProbe for CountingGenerateProbe {
        fn probe(&self, _model: &str) -> impl Future<Output = GenerateOutcome> + Send {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    /// Build an adapter wired to a fake catalogue and a counting generate probe,
    /// returning the adapter plus the generate-call and models-call counters.
    fn wired(
        models_outcome: ModelsOutcome,
        generate_outcome: GenerateOutcome,
    ) -> (
        OllamaAdapter<
            crate::runtime::ollama::ReqwestProbe,
            FakeModelsProbe,
            crate::runtime::loaded_models::ReqwestRunningProbe,
            CountingGenerateProbe,
        >,
        Arc<AtomicUsize>,
        Arc<AtomicUsize>,
    ) {
        let gen_calls = Arc::new(AtomicUsize::new(0));
        let models_calls = Arc::new(AtomicUsize::new(0));
        let adapter = OllamaAdapter::with_all_probes_and_generate(
            crate::runtime::ollama::ReqwestProbe::new(),
            FakeModelsProbe {
                outcome: models_outcome,
                calls: models_calls.clone(),
            },
            crate::runtime::loaded_models::ReqwestRunningProbe::new(),
            CountingGenerateProbe {
                outcome: generate_outcome,
                calls: gen_calls.clone(),
            },
        );
        (adapter, gen_calls, models_calls)
    }

    /// Run the orchestration synchronously for a test adapter.
    fn run<P, M, R, G>(
        adapter: &OllamaAdapter<P, M, R, G>,
        model: &str,
        ack: bool,
    ) -> InferenceObservation
    where
        P: VersionProbe,
        M: ModelsProbe,
        R: RunningProbe,
        G: GenerateProbe,
    {
        tauri::async_runtime::block_on(run_inference_observation_with_adapter(adapter, model, ack))
    }

    /// A catalogue that is `Available` and lists exactly `model`.
    fn catalogue_available_with(model: &str) -> ModelsOutcome {
        ModelsOutcome::Responded {
            status: 200,
            body: format!(r#"{{"models":[{{"name":"{model}"}}]}}"#),
        }
    }

    /// A catalogue that is `Available` and lists nothing.
    fn catalogue_available_empty() -> ModelsOutcome {
        ModelsOutcome::Responded {
            status: 200,
            body: r#"{"models":[]}"#.to_string(),
        }
    }

    /// A completed generate outcome carrying a distinctive `response` secret, to
    /// prove the generated text is discarded end to end.
    fn completed_with_secret_response() -> GenerateOutcome {
        GenerateOutcome::Responded {
            status: 200,
            body: r#"{"model":"example:1b","response":"SECRET-GENERATED-OUTPUT","done":true,"eval_count":4,"eval_duration":1000000000}"#
                .to_string(),
        }
    }

    #[test]
    fn no_model_when_identifier_is_empty() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "", true);
        assert_eq!(obs.state, InferenceObservationState::NoModelSelected);
        assert!(obs.model.is_none(), "no model carried for NoModelSelected");
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 0, "no identity GET");
    }

    #[test]
    fn no_model_when_identifier_is_whitespace() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "   \t  ", true);
        assert_eq!(obs.state, InferenceObservationState::NoModelSelected);
        assert!(obs.model.is_none());
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 0, "no identity GET");
    }

    #[test]
    fn no_model_when_identifier_has_control_chars() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example\0:1b", true);
        assert_eq!(obs.state, InferenceObservationState::NoModelSelected);
        assert!(obs.model.is_none());
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 0, "no identity GET");
    }

    #[test]
    fn blocked_when_not_acknowledged_and_carries_model_with_no_post() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", false);
        assert_eq!(
            obs.state,
            InferenceObservationState::BlockedByExecutionBoundary
        );
        assert_eq!(obs.model.as_deref(), Some("example:1b"), "model carried");
        assert_eq!(
            gen_calls.load(Ordering::SeqCst),
            0,
            "no POST when not authorized"
        );
        assert_eq!(
            models_calls.load(Ordering::SeqCst),
            0,
            "authorization gate runs before the identity GET"
        );
    }

    #[test]
    fn authorization_gate_precedes_catalogue_identity_check() {
        // With the catalogue NotDetected but authorization withheld, the result
        // is BlockedByExecutionBoundary (not NotDetected): the auth gate is
        // before the identity GET, and no GET is issued.
        let (adapter, gen_calls, models_calls) = wired(
            ModelsOutcome::ConnectionRefused,
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", false);
        assert_eq!(
            obs.state,
            InferenceObservationState::BlockedByExecutionBoundary
        );
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 0, "no identity GET");
    }

    #[test]
    fn not_detected_when_catalogue_is_not_detected() {
        let (adapter, gen_calls, models_calls) = wired(
            ModelsOutcome::ConnectionRefused,
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::NotDetected);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn unreachable_when_catalogue_is_unreachable() {
        let (adapter, gen_calls, models_calls) =
            wired(ModelsOutcome::Timeout, completed_with_secret_response());
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::Unreachable);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn responded_unexpectedly_when_catalogue_responded_unexpectedly() {
        let (adapter, gen_calls, models_calls) = wired(
            ModelsOutcome::Responded {
                status: 500,
                body: "internal error".to_string(),
            },
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::RespondedUnexpectedly);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn model_unavailable_when_not_listed_in_an_available_catalogue() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_empty(),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::ModelUnavailable);
        assert_eq!(
            gen_calls.load(Ordering::SeqCst),
            0,
            "no POST for off-catalogue model"
        );
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn model_unavailable_when_a_different_model_is_listed() {
        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("other:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::ModelUnavailable);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 0, "no POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn completed_when_present_and_acknowledged_is_undetermined_not_local() {
        let _lock = GUARD_TEST_LOCK.lock().unwrap();
        super::INFERENCING.store(false, Ordering::SeqCst);

        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::Completed);
        assert_eq!(obs.model.as_deref(), Some("example:1b"));
        assert_eq!(
            obs.execution_boundary, "undetermined",
            "never a locality claim"
        );
        assert_eq!(obs.diagnostic_profile, "diagnostic-basic-v1");
        assert_eq!(gen_calls.load(Ordering::SeqCst), 1, "exactly one POST");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn no_retry_on_timeout_issues_exactly_one_post() {
        let _lock = GUARD_TEST_LOCK.lock().unwrap();
        super::INFERENCING.store(false, Ordering::SeqCst);

        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            GenerateOutcome::Timeout,
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::TimedOut);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 1, "one attempt, no retry");
        assert_eq!(models_calls.load(Ordering::SeqCst), 1, "one identity GET");
    }

    #[test]
    fn generated_response_is_discarded_end_to_end() {
        let _lock = GUARD_TEST_LOCK.lock().unwrap();
        super::INFERENCING.store(false, Ordering::SeqCst);

        let (adapter, _gen_calls, _models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        let obs = run(&adapter, "example:1b", true);
        assert_eq!(obs.state, InferenceObservationState::Completed);
        // No observation field carries the generated secret. The DTO has no
        // response field; the only strings are the model name and (via the view)
        // controlled text — none of which is the generated output.
        let combined = format!(
            "{:?} {:?} {:?} {:?}",
            obs.model, obs.diagnostic_profile, obs.execution_boundary, obs.state
        );
        assert!(
            !combined.contains("SECRET-GENERATED-OUTPUT"),
            "generated response must not surface in the observation"
        );
        // The controlled text for Completed must not echo the generated output.
        let text = ollama_inference_text(&obs);
        let text_combined = format!(
            "{} {} {}",
            text.interpretation, text.why_it_matters, text.limitations
        );
        assert!(
            !text_combined.contains("SECRET-GENERATED-OUTPUT"),
            "generated response must not surface in controlled text"
        );
    }

    #[test]
    fn authorization_is_request_scoped_and_does_not_persist() {
        let _lock = GUARD_TEST_LOCK.lock().unwrap();
        super::INFERENCING.store(false, Ordering::SeqCst);

        let (adapter, _gen_calls, _models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );
        // An authorized run completes.
        let authorized = run(&adapter, "example:1b", true);
        assert_eq!(authorized.state, InferenceObservationState::Completed);
        // A subsequent unacknowledged run is blocked: the prior authorization
        // did not persist (no global/persistent auth state, no allow-list).
        let unacknowledged = run(&adapter, "example:1b", false);
        assert_eq!(
            unacknowledged.state,
            InferenceObservationState::BlockedByExecutionBoundary
        );
    }

    #[test]
    fn guard_serializes_runs_and_releases_after_completion() {
        let _lock = GUARD_TEST_LOCK.lock().unwrap();
        super::INFERENCING.store(false, Ordering::SeqCst);

        let (adapter, gen_calls, models_calls) = wired(
            catalogue_available_with("example:1b"),
            completed_with_secret_response(),
        );

        // First authorized run completes and releases the guard.
        let first = run(&adapter, "example:1b", true);
        assert_eq!(first.state, InferenceObservationState::Completed);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 1);
        assert_eq!(models_calls.load(Ordering::SeqCst), 1);

        // A second authorized run also completes: the guard was released by the
        // first run's drop, so this is NOT Running.
        let second = run(&adapter, "example:1b", true);
        assert_eq!(second.state, InferenceObservationState::Completed);
        assert_eq!(gen_calls.load(Ordering::SeqCst), 2);

        // Simulate a concurrent in-flight run by holding the guard ourselves.
        // The identity GET still runs (the guard is after identity), but no
        // second POST is issued.
        super::INFERENCING.store(true, Ordering::SeqCst);
        let held = run(&adapter, "example:1b", true);
        assert_eq!(held.state, InferenceObservationState::Running);
        assert_eq!(
            gen_calls.load(Ordering::SeqCst),
            2,
            "no second POST while held"
        );
        assert_eq!(
            models_calls.load(Ordering::SeqCst),
            3,
            "identity GET runs before the guard"
        );

        // Release so later tests are not poisoned.
        super::INFERENCING.store(false, Ordering::SeqCst);
    }
}
