//! Passive llama.cpp single-model server integration (Milestone 1O).
//!
//! The adapter uses two fixed numeric-loopback GET routes, follows no
//! redirects, and exposes only a validated provider-reported model ID. It has no
//! inference, process, filesystem, service-control, or model-management path.

use std::future::Future;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const HEALTH_ENDPOINT: &str = "http://127.0.0.1:8080/health";
pub const MODELS_ENDPOINT: &str = "http://127.0.0.1:8080/v1/models";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_RESPONSE_BYTES: usize = 16 * 1024;
const MAX_MODEL_ID_CHARS: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LlamaCppState {
    Unreachable,
    Loading,
    Ready,
    AuthenticationRequired,
    Incompatible,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LlamaCppServedModel {
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LlamaCppSnapshot {
    pub provider: &'static str,
    pub state: LlamaCppState,
    pub served_model: Option<LlamaCppServedModel>,
    pub api_scope: &'static str,
    pub compute_location: &'static str,
    pub state_label: &'static str,
    pub interpretation: &'static str,
    pub why_it_matters: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpOutcome {
    Responded { status: u16, body: String },
    ConnectionRefused,
    Timeout,
    ResponseTooLarge,
    Other,
}

pub trait LlamaCppProbe: Send + Sync {
    fn health(&self) -> impl Future<Output = HttpOutcome> + Send;
    fn models(&self) -> impl Future<Output = HttpOutcome> + Send;
}

pub struct ReqwestLlamaCppProbe {
    client: reqwest::Client,
}

impl ReqwestLlamaCppProbe {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .connect_timeout(CONNECT_TIMEOUT)
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("bounded passive llama.cpp client"),
        }
    }
}

impl Default for ReqwestLlamaCppProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl LlamaCppProbe for ReqwestLlamaCppProbe {
    fn health(&self) -> impl Future<Output = HttpOutcome> + Send {
        let client = self.client.clone();
        async move { reduce(client.get(HEALTH_ENDPOINT).send().await).await }
    }

    fn models(&self) -> impl Future<Output = HttpOutcome> + Send {
        let client = self.client.clone();
        async move { reduce(client.get(MODELS_ENDPOINT).send().await).await }
    }
}

async fn reduce(result: Result<reqwest::Response, reqwest::Error>) -> HttpOutcome {
    match result {
        Ok(mut response) => {
            let status = response.status().as_u16();
            if response
                .content_length()
                .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
            {
                return HttpOutcome::ResponseTooLarge;
            }
            let mut bytes = Vec::new();
            loop {
                match response.chunk().await {
                    Ok(Some(chunk)) if append_bounded(&mut bytes, &chunk) => {}
                    Ok(Some(_)) => return HttpOutcome::ResponseTooLarge,
                    Ok(None) => break,
                    Err(error) => return classify(error),
                }
            }
            let body = String::from_utf8(bytes).unwrap_or_default();
            HttpOutcome::Responded { status, body }
        }
        Err(error) => classify(error),
    }
}

fn append_bounded(bytes: &mut Vec<u8>, chunk: &[u8]) -> bool {
    if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
        bytes.clear();
        return false;
    }
    bytes.extend_from_slice(chunk);
    true
}

fn classify(error: reqwest::Error) -> HttpOutcome {
    if error.is_timeout() {
        return HttpOutcome::Timeout;
    }
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(&error);
    while let Some(value) = source {
        if let Some(io) = value.downcast_ref::<std::io::Error>() {
            if io.kind() == std::io::ErrorKind::ConnectionRefused {
                return HttpOutcome::ConnectionRefused;
            }
        }
        source = value.source();
    }
    HttpOutcome::Other
}

fn controlled_text(state: &LlamaCppState) -> (&'static str, &'static str, &'static str) {
    match state {
        LlamaCppState::Unreachable => (
            "Unreachable",
            "The bounded passive request could not complete.",
            "No retry or server-management action was attempted.",
        ),
        LlamaCppState::Loading => (
            "Loading",
            "llama-server reports that its model is loading.",
            "AI Engine Room is observing the developer-operated server and does not manage the model.",
        ),
        LlamaCppState::Ready => (
            "Ready",
            "llama-server reports ready with one compatible safe served-model ID.",
            "Loopback availability does not independently verify compute placement.",
        ),
        LlamaCppState::AuthenticationRequired => (
            "Authentication required",
            "llama-server answered but requires authentication.",
            "Authenticated llama.cpp access is not supported in this version.",
        ),
        LlamaCppState::Incompatible => (
            "Incompatible",
            "llama-server responded, but its traditional single-model identity contract could not be used safely.",
            "The response must contain one non-sensitive served-model ID; no rejected identity is displayed.",
        ),
    }
}

fn snapshot(state: LlamaCppState, model_id: Option<String>) -> LlamaCppSnapshot {
    let served_model = (state == LlamaCppState::Ready)
        .then(|| model_id.map(|model_id| LlamaCppServedModel { model_id }))
        .flatten();
    let (state_label, interpretation, why_it_matters) = controlled_text(&state);
    LlamaCppSnapshot {
        provider: "llama_cpp",
        state,
        served_model,
        api_scope: "same_machine_loopback",
        compute_location: "not_independently_verified",
        state_label,
        interpretation,
        why_it_matters,
    }
}

#[derive(Deserialize)]
struct HealthWire {
    status: String,
}

#[derive(Deserialize)]
struct ModelsResponseWire {
    data: Vec<DataWire>,
}

#[derive(Deserialize)]
struct DataWire {
    id: String,
    owned_by: String,
}

fn is_ready_health(body: &str) -> bool {
    serde_json::from_str::<HealthWire>(body)
        .map(|health| health.status == "ok")
        .unwrap_or(false)
}

fn safe_model_id(body: &str) -> Option<String> {
    let response = serde_json::from_str::<ModelsResponseWire>(body).ok()?;
    let [data] = response.data.as_slice() else {
        return None;
    };
    if data.owned_by != "llamacpp" || !is_safe_model_id(&data.id) {
        return None;
    }
    Some(data.id.clone())
}

fn is_safe_model_id(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= MAX_MODEL_ID_CHARS
        && !value.chars().any(char::is_control)
        && !value.contains(['/', '\\', ':'])
        && !value.starts_with('~')
}

fn map_non_ready(outcome: HttpOutcome) -> LlamaCppState {
    match outcome {
        HttpOutcome::ConnectionRefused => LlamaCppState::Unreachable,
        HttpOutcome::Timeout | HttpOutcome::Other => LlamaCppState::Unreachable,
        HttpOutcome::ResponseTooLarge => LlamaCppState::Incompatible,
        HttpOutcome::Responded {
            status: 401 | 403, ..
        } => LlamaCppState::AuthenticationRequired,
        HttpOutcome::Responded { status: 503, .. } => LlamaCppState::Loading,
        HttpOutcome::Responded { .. } => LlamaCppState::Incompatible,
    }
}

fn map_models_failure(outcome: HttpOutcome) -> LlamaCppState {
    match outcome {
        HttpOutcome::ConnectionRefused | HttpOutcome::Timeout | HttpOutcome::Other => {
            LlamaCppState::Unreachable
        }
        HttpOutcome::ResponseTooLarge => LlamaCppState::Incompatible,
        HttpOutcome::Responded {
            status: 401 | 403, ..
        } => LlamaCppState::AuthenticationRequired,
        HttpOutcome::Responded { .. } => LlamaCppState::Incompatible,
    }
}

pub struct LlamaCppAdapter<P: LlamaCppProbe = ReqwestLlamaCppProbe> {
    probe: P,
}

impl LlamaCppAdapter<ReqwestLlamaCppProbe> {
    pub fn new() -> Self {
        Self {
            probe: ReqwestLlamaCppProbe::new(),
        }
    }
}

impl Default for LlamaCppAdapter<ReqwestLlamaCppProbe> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: LlamaCppProbe> LlamaCppAdapter<P> {
    #[cfg(test)]
    fn with_probe(probe: P) -> Self {
        Self { probe }
    }

    pub async fn snapshot(&self) -> LlamaCppSnapshot {
        let health = self.probe.health().await;
        match health {
            HttpOutcome::Responded {
                status: 200,
                ref body,
            } if is_ready_health(body) => {}
            outcome => return snapshot(map_non_ready(outcome), None),
        }

        match self.probe.models().await {
            HttpOutcome::Responded { status: 200, body } => match safe_model_id(&body) {
                Some(model_id) => snapshot(LlamaCppState::Ready, Some(model_id)),
                None => snapshot(LlamaCppState::Incompatible, None),
            },
            outcome => snapshot(map_models_failure(outcome), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct FakeProbe {
        health: HttpOutcome,
        models: HttpOutcome,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl LlamaCppProbe for FakeProbe {
        fn health(&self) -> impl Future<Output = HttpOutcome> + Send {
            let outcome = self.health.clone();
            let calls = self.calls.clone();
            async move {
                calls.lock().unwrap().push("health");
                outcome
            }
        }

        fn models(&self) -> impl Future<Output = HttpOutcome> + Send {
            let outcome = self.models.clone();
            let calls = self.calls.clone();
            async move {
                calls.lock().unwrap().push("models");
                outcome
            }
        }
    }

    fn response(model_id: &str) -> String {
        serde_json::json!({
            "data": [{"id": model_id, "owned_by": "llamacpp"}]
        })
        .to_string()
    }

    fn fake(
        health: HttpOutcome,
        models: HttpOutcome,
    ) -> (LlamaCppAdapter<FakeProbe>, Arc<Mutex<Vec<&'static str>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (
            LlamaCppAdapter::with_probe(FakeProbe {
                health,
                models,
                calls: calls.clone(),
            }),
            calls,
        )
    }

    #[test]
    fn endpoints_are_fixed_numeric_loopback_get_routes() {
        assert_eq!(HEALTH_ENDPOINT, "http://127.0.0.1:8080/health");
        assert_eq!(MODELS_ENDPOINT, "http://127.0.0.1:8080/v1/models");
    }

    #[test]
    fn safe_model_id_policy_rejects_path_like_and_ambiguous_values() {
        assert!(is_safe_model_id("leaf-model"));
        for value in [
            "",
            "   ",
            " leaf",
            "leaf ",
            "/home/owner/model.gguf",
            "folder/model",
            r"C:\\Models\\model.gguf",
            r"\\server\\share\\model.gguf",
            "file://model.gguf",
            "leaf:model",
            "~model",
            "leaf\nmodel",
        ] {
            assert!(!is_safe_model_id(value), "unexpectedly safe: {value:?}");
        }
        assert!(!is_safe_model_id(&"x".repeat(MAX_MODEL_ID_CHARS + 1)));
    }

    #[test]
    fn model_contract_requires_one_safe_owned_id_and_ignores_extra_fields() {
        assert_eq!(
            safe_model_id(&response("leaf-model")),
            Some("leaf-model".into())
        );
        assert_eq!(
            safe_model_id(
                r#"{"data":[{"id":"leaf-model","owned_by":"llamacpp","model_path":"/private/ignored.gguf","aliases":["ignored"]}],"models":[{"name":"ignored"}],"other":"ignored"}"#
            ),
            Some("leaf-model".into())
        );
        for body in [
            r#"{}"#.to_string(),
            r#"{"data":null}"#.to_string(),
            r#"{"data":[]}"#.to_string(),
            r#"{"data":[{"id":"a","owned_by":"llamacpp"},{"id":"b","owned_by":"llamacpp"}]}"#
                .to_string(),
            r#"{"data":[{"owned_by":"llamacpp"}]}"#.to_string(),
            r#"{"data":[{"id":7,"owned_by":"llamacpp"}]}"#.to_string(),
            r#"{"data":[{"id":"","owned_by":"llamacpp"}]}"#.to_string(),
            r#"{"data":[{"id":"a"}]}"#.to_string(),
            r#"{"data":[{"id":"a","owned_by":7}]}"#.to_string(),
            r#"{"data":[{"id":"a","owned_by":"LlamaCpp"}]}"#.to_string(),
            r#"{"data":[{"id":"a","owned_by":"another-provider"}]}"#.to_string(),
            "malformed".to_string(),
            response("/private/model.gguf"),
        ] {
            assert_eq!(safe_model_id(&body), None, "unexpectedly accepted: {body}");
        }
    }

    #[test]
    fn incremental_body_bound_discards_accumulated_bytes_on_overflow() {
        let mut bytes = vec![b'x'; MAX_RESPONSE_BYTES - 1];
        assert!(append_bounded(&mut bytes, b"x"));
        assert_eq!(bytes.len(), MAX_RESPONSE_BYTES);
        assert!(!append_bounded(&mut bytes, b"y"));
        assert!(bytes.is_empty());
    }

    #[test]
    fn controlled_snapshots_never_serialize_rejected_identity() {
        let secret = "/home/private/model.gguf";
        let result = snapshot(
            LlamaCppState::Incompatible,
            safe_model_id(&response(secret)),
        );
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains(secret));
        assert_eq!(result.served_model, None);
        assert_eq!(result.api_scope, "same_machine_loopback");
        assert_eq!(result.compute_location, "not_independently_verified");
    }

    #[test]
    fn all_non_ready_health_outcomes_stop_before_models() {
        let cases = [
            (HttpOutcome::ConnectionRefused, LlamaCppState::Unreachable),
            (HttpOutcome::Timeout, LlamaCppState::Unreachable),
            (HttpOutcome::Other, LlamaCppState::Unreachable),
            (HttpOutcome::ResponseTooLarge, LlamaCppState::Incompatible),
            (
                HttpOutcome::Responded {
                    status: 503,
                    body: "loading".into(),
                },
                LlamaCppState::Loading,
            ),
            (
                HttpOutcome::Responded {
                    status: 401,
                    body: "private".into(),
                },
                LlamaCppState::AuthenticationRequired,
            ),
            (
                HttpOutcome::Responded {
                    status: 403,
                    body: "private".into(),
                },
                LlamaCppState::AuthenticationRequired,
            ),
            (
                HttpOutcome::Responded {
                    status: 302,
                    body: "redirect".into(),
                },
                LlamaCppState::Incompatible,
            ),
            (
                HttpOutcome::Responded {
                    status: 200,
                    body: "{}".into(),
                },
                LlamaCppState::Incompatible,
            ),
            (
                HttpOutcome::Responded {
                    status: 200,
                    body: r#"{"status":"busy"}"#.into(),
                },
                LlamaCppState::Incompatible,
            ),
            (
                HttpOutcome::Responded {
                    status: 500,
                    body: "private".into(),
                },
                LlamaCppState::Incompatible,
            ),
        ];
        for (health, state) in cases {
            let (adapter, calls) = fake(health, HttpOutcome::Other);
            let result = tauri::async_runtime::block_on(adapter.snapshot());
            assert_eq!(result.state, state);
            assert_eq!(*calls.lock().unwrap(), vec!["health"]);
        }
    }

    #[test]
    fn ready_health_uses_models_once_and_returns_only_safe_model_id() {
        let (adapter, calls) = fake(
            HttpOutcome::Responded {
                status: 200,
                body: r#"{"status":"ok"}"#.into(),
            },
            HttpOutcome::Responded {
                status: 200,
                body: response("leaf-model"),
            },
        );
        let result = tauri::async_runtime::block_on(adapter.snapshot());
        assert_eq!(result.state, LlamaCppState::Ready);
        assert_eq!(result.served_model.unwrap().model_id, "leaf-model");
        assert_eq!(*calls.lock().unwrap(), vec!["health", "models"]);
    }

    #[test]
    fn models_failures_are_controlled_without_retry() {
        let outcomes = [
            (HttpOutcome::ConnectionRefused, LlamaCppState::Unreachable),
            (HttpOutcome::Timeout, LlamaCppState::Unreachable),
            (HttpOutcome::Other, LlamaCppState::Unreachable),
            (HttpOutcome::ResponseTooLarge, LlamaCppState::Incompatible),
            (
                HttpOutcome::Responded {
                    status: 401,
                    body: "secret".into(),
                },
                LlamaCppState::AuthenticationRequired,
            ),
            (
                HttpOutcome::Responded {
                    status: 403,
                    body: "secret".into(),
                },
                LlamaCppState::AuthenticationRequired,
            ),
            (
                HttpOutcome::Responded {
                    status: 500,
                    body: "secret".into(),
                },
                LlamaCppState::Incompatible,
            ),
            (
                HttpOutcome::Responded {
                    status: 200,
                    body: "malformed".into(),
                },
                LlamaCppState::Incompatible,
            ),
        ];
        for (models, state) in outcomes {
            let (adapter, calls) = fake(
                HttpOutcome::Responded {
                    status: 200,
                    body: r#"{"status":"ok"}"#.into(),
                },
                models,
            );
            let result = tauri::async_runtime::block_on(adapter.snapshot());
            assert_eq!(result.state, state);
            assert_eq!(*calls.lock().unwrap(), vec!["health", "models"]);
        }
    }
}
