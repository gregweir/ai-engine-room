//! LM Studio native REST v1 integration (Milestone 1L).
//!
//! This module is deliberately application-layer and provider-specific. It
//! probes only numeric loopback, follows no redirects, performs no discovery or
//! model-management action, and discards generated/reasoning output while
//! parsing an explicitly authorized observation.

use std::future::Future;
use std::time::Duration;

use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

use crate::runtime::inference::{InferenceGuard, DIAGNOSTIC_PROFILE, DIAGNOSTIC_PROMPT};

pub const MODELS_ENDPOINT: &str = "http://127.0.0.1:1234/api/v1/models";
pub const CHAT_ENDPOINT: &str = "http://127.0.0.1:1234/api/v1/chat";
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const MODELS_TIMEOUT: Duration = Duration::from_secs(2);
const CHAT_TIMEOUT: Duration = Duration::from_secs(120);
pub const MAX_OUTPUT_TOKENS: u64 = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LmStudioState {
    Available,
    NotDetected,
    Unreachable,
    AuthenticationRequired,
    UnsupportedApi,
    RespondedUnexpectedly,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LmStudioLoadedInstance {
    pub instance_id: String,
    pub context_length: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LmStudioModel {
    pub model_id: String,
    pub display_name: String,
    pub model_type: String,
    pub inference_eligible: bool,
    pub publisher: Option<String>,
    pub architecture: Option<String>,
    pub quantization: Option<String>,
    pub bits_per_weight: Option<f64>,
    pub size_bytes: Option<u64>,
    pub params_string: Option<String>,
    pub max_context_length: Option<u64>,
    pub format: Option<String>,
    pub loaded_instances: Vec<LmStudioLoadedInstance>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LmStudioSnapshot {
    pub provider: &'static str,
    pub state: LmStudioState,
    pub models: Vec<LmStudioModel>,
    pub skipped: usize,
    pub state_label: &'static str,
    pub interpretation: &'static str,
    pub why_it_matters: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpOutcome {
    Responded { status: u16, body: String },
    ConnectionRefused,
    Timeout,
    Other,
}

pub trait LmStudioProbe: Send + Sync {
    fn models(&self) -> impl Future<Output = HttpOutcome> + Send;
    fn chat(&self, body: String) -> impl Future<Output = HttpOutcome> + Send;
}

pub struct ReqwestLmStudioProbe {
    models_client: reqwest::Client,
    chat_client: reqwest::Client,
}

impl ReqwestLmStudioProbe {
    pub fn new() -> Self {
        let base = || {
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .connect_timeout(CONNECT_TIMEOUT)
        };
        Self {
            models_client: base()
                .timeout(MODELS_TIMEOUT)
                .build()
                .expect("bounded LM Studio client"),
            chat_client: base()
                .timeout(CHAT_TIMEOUT)
                .build()
                .expect("bounded LM Studio chat client"),
        }
    }
}

impl Default for ReqwestLmStudioProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl LmStudioProbe for ReqwestLmStudioProbe {
    fn models(&self) -> impl Future<Output = HttpOutcome> + Send {
        let client = self.models_client.clone();
        async move { reduce(client.get(MODELS_ENDPOINT).send().await).await }
    }
    fn chat(&self, body: String) -> impl Future<Output = HttpOutcome> + Send {
        let client = self.chat_client.clone();
        async move {
            reduce(
                client
                    .post(CHAT_ENDPOINT)
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .send()
                    .await,
            )
            .await
        }
    }
}

async fn reduce(result: Result<reqwest::Response, reqwest::Error>) -> HttpOutcome {
    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            match response.text().await {
                Ok(body) => HttpOutcome::Responded { status, body },
                Err(e) => classify(e),
            }
        }
        Err(e) => classify(e),
    }
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

fn text(state: &LmStudioState) -> (&'static str, &'static str, &'static str) {
    match state {
        LmStudioState::Available => ("Available", "LM Studio native REST v1 is available on this machine.", "Its downloaded models and loaded instances are shown separately."),
        LmStudioState::NotDetected => ("Not detected", "LM Studio's server was not detected.", "Nothing answered the fixed same-machine endpoint; this does not determine whether LM Studio is installed."),
        LmStudioState::Unreachable => ("Unreachable", "LM Studio could not be reached right now.", "The bounded local request timed out or failed; no retry was attempted."),
        LmStudioState::AuthenticationRequired => ("Authentication required", "LM Studio answered but requires authentication.", "Authenticated LM Studio access is not supported in this version."),
        LmStudioState::UnsupportedApi => ("Native v1 required", "The endpoint did not provide the required native REST v1 contract.", "AI Engine Room requires LM Studio 0.4.0 or newer and does not fall back to older or OpenAI-compatible APIs."),
        LmStudioState::RespondedUnexpectedly => ("Unexpected response", "LM Studio answered in a form AI Engine Room could not use.", "No raw response or transport details are displayed."),
    }
}

pub fn interpret_models(outcome: HttpOutcome) -> LmStudioSnapshot {
    let (state, models, skipped) = match outcome {
        HttpOutcome::ConnectionRefused => (LmStudioState::NotDetected, vec![], 0),
        HttpOutcome::Timeout | HttpOutcome::Other => (LmStudioState::Unreachable, vec![], 0),
        HttpOutcome::Responded {
            status: 401 | 403, ..
        } => (LmStudioState::AuthenticationRequired, vec![], 0),
        HttpOutcome::Responded { status: 404, .. } => (LmStudioState::UnsupportedApi, vec![], 0),
        HttpOutcome::Responded { status, .. } if status != 200 => {
            (LmStudioState::RespondedUnexpectedly, vec![], 0)
        }
        HttpOutcome::Responded { body, .. } => match parse_models(&body) {
            Some((models, skipped)) => (LmStudioState::Available, models, skipped),
            None => (LmStudioState::UnsupportedApi, vec![], 0),
        },
    };
    let (state_label, interpretation, why_it_matters) = text(&state);
    LmStudioSnapshot {
        provider: "lm_studio",
        state,
        models,
        skipped,
        state_label,
        interpretation,
        why_it_matters,
    }
}

fn parse_models(body: &str) -> Option<(Vec<LmStudioModel>, usize)> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let array = value.get("models")?.as_array()?;
    let mut models = Vec::new();
    let mut skipped = 0;
    for item in array {
        let key = item
            .get("key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let model_type = item
            .get("type")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let (Some(key), Some(model_type)) = (key, model_type) else {
            skipped += 1;
            continue;
        };
        let display_name = string(item, "display_name").unwrap_or_else(|| key.to_string());
        let loaded_instances = item
            .get("loaded_instances")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| {
                        let id = string(v, "id")?;
                        let context_length =
                            v.get("config").and_then(|c| uint(c, "context_length"));
                        Some(LmStudioLoadedInstance {
                            instance_id: id,
                            context_length,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let quant = item.get("quantization");
        models.push(LmStudioModel {
            model_id: key.to_string(),
            display_name,
            model_type: model_type.to_string(),
            inference_eligible: model_type == "llm",
            publisher: string(item, "publisher"),
            architecture: string(item, "architecture"),
            quantization: quant.and_then(|q| string(q, "name")),
            bits_per_weight: quant.and_then(|q| finite(q, "bits_per_weight")),
            size_bytes: uint(item, "size_bytes"),
            params_string: string(item, "params_string"),
            max_context_length: uint(item, "max_context_length"),
            format: string(item, "format"),
            loaded_instances,
        });
    }
    Some((models, skipped))
}

fn string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}
fn uint(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key)?.as_u64()
}
fn finite(value: &serde_json::Value, key: &str) -> Option<f64> {
    let n = value.get(key)?.as_f64()?;
    (n.is_finite() && n >= 0.0).then_some(n)
}

pub fn chat_request(model: &str) -> String {
    serde_json::json!({"model": model, "input": DIAGNOSTIC_PROMPT, "stream": false, "store": false, "temperature": 0, "max_output_tokens": MAX_OUTPUT_TOKENS}).to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LmStudioObservation {
    pub state: &'static str,
    pub provider: &'static str,
    pub model: Option<String>,
    pub diagnostic_profile: Option<&'static str>,
    pub api_scope: &'static str,
    pub compute_location: &'static str,
    pub input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub tokens_per_second: Option<f64>,
    pub time_to_first_token_seconds: Option<f64>,
    pub model_load_time_seconds: Option<f64>,
    pub interpretation: &'static str,
    pub why_it_matters: &'static str,
    pub limitations: &'static str,
}

fn empty_observation(
    state: &'static str,
    model: Option<&str>,
    interpretation: &'static str,
) -> LmStudioObservation {
    LmStudioObservation {
        state,
        provider: "lm_studio",
        model: model.map(str::to_string),
        diagnostic_profile: model.map(|_| DIAGNOSTIC_PROFILE),
        api_scope: "same_machine_loopback",
        compute_location: "not_independently_verified",
        input_tokens: None,
        total_output_tokens: None,
        reasoning_output_tokens: None,
        tokens_per_second: None,
        time_to_first_token_seconds: None,
        model_load_time_seconds: None,
        interpretation,
        why_it_matters: "This is one observed run, not a benchmark or model-quality score.",
        limitations:
            "The API is on this machine; exact compute placement is not independently verified.",
    }
}

pub fn interpret_chat(outcome: HttpOutcome, model: &str) -> LmStudioObservation {
    let HttpOutcome::Responded { status: 200, body } = outcome else {
        return match outcome {
            HttpOutcome::Timeout => empty_observation(
                "timed_out",
                Some(model),
                "The observation did not finish within the time limit.",
            ),
            HttpOutcome::ConnectionRefused | HttpOutcome::Other => empty_observation(
                "unreachable",
                Some(model),
                "LM Studio could not be reached for the observation.",
            ),
            _ => empty_observation(
                "responded_unexpectedly",
                Some(model),
                "LM Studio responded in a way AI Engine Room could not use.",
            ),
        };
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
        return empty_observation(
            "responded_unexpectedly",
            Some(model),
            "LM Studio responded in a way AI Engine Room could not use.",
        );
    };
    let Some(stats) = value.get("stats").and_then(|v| v.as_object()) else {
        return empty_observation(
            "responded_unexpectedly",
            Some(model),
            "LM Studio responded in a way AI Engine Room could not use.",
        );
    };
    // `output`, reasoning content, response id, and instance id are deliberately never read.
    LmStudioObservation {
        state: "completed",
        provider: "lm_studio",
        model: Some(model.to_string()),
        diagnostic_profile: Some(DIAGNOSTIC_PROFILE),
        api_scope: "same_machine_loopback",
        compute_location: "not_independently_verified",
        input_tokens: stats.get("input_tokens").and_then(|v| v.as_u64()),
        total_output_tokens: stats.get("total_output_tokens").and_then(|v| v.as_u64()),
        reasoning_output_tokens: stats
            .get("reasoning_output_tokens")
            .and_then(|v| v.as_u64()),
        tokens_per_second: stats
            .get("tokens_per_second")
            .and_then(|v| v.as_f64())
            .filter(|v| v.is_finite() && *v >= 0.0),
        time_to_first_token_seconds: stats
            .get("time_to_first_token_seconds")
            .and_then(|v| v.as_f64())
            .filter(|v| v.is_finite() && *v >= 0.0),
        model_load_time_seconds: stats
            .get("model_load_time_seconds")
            .and_then(|v| v.as_f64())
            .filter(|v| v.is_finite() && *v >= 0.0),
        interpretation:
            "One LM Studio observed inference completed with provider-reported statistics.",
        why_it_matters: "This is one observed run, not a benchmark or model-quality score.",
        limitations:
            "The API is on this machine; exact compute placement is not independently verified.",
    }
}

pub struct LmStudioAdapter<P: LmStudioProbe = ReqwestLmStudioProbe> {
    probe: P,
}
impl LmStudioAdapter<ReqwestLmStudioProbe> {
    pub fn new() -> Self {
        Self {
            probe: ReqwestLmStudioProbe::new(),
        }
    }
}
impl Default for LmStudioAdapter<ReqwestLmStudioProbe> {
    fn default() -> Self {
        Self::new()
    }
}
impl<P: LmStudioProbe> LmStudioAdapter<P> {
    pub fn with_probe(probe: P) -> Self {
        Self { probe }
    }
    pub async fn snapshot(&self) -> LmStudioSnapshot {
        interpret_models(self.probe.models().await)
    }
    pub async fn observe(
        &self,
        model: &str,
        execution_ack: bool,
        behavior_ack: bool,
    ) -> LmStudioObservation {
        if model.trim().is_empty() || model.chars().any(char::is_control) {
            return empty_observation(
                "no_model_selected",
                None,
                "No model is selected for an observed inference.",
            );
        }
        if !execution_ack || !behavior_ack {
            return empty_observation("blocked_by_execution_boundary", Some(model), "AI Engine Room will not run an LM Studio observation without explicit authorization of the compute and JIT boundaries.");
        }
        let snapshot = self.snapshot().await;
        if snapshot.state != LmStudioState::Available {
            return empty_observation(
                match snapshot.state {
                    LmStudioState::NotDetected => "not_detected",
                    LmStudioState::Unreachable => "unreachable",
                    _ => "responded_unexpectedly",
                },
                Some(model),
                "LM Studio is not available for this observation.",
            );
        }
        if !snapshot
            .models
            .iter()
            .any(|m| m.model_id == model && m.inference_eligible)
        {
            return empty_observation(
                "model_unavailable",
                Some(model),
                "The selected LLM was not found in the LM Studio catalogue.",
            );
        }
        let Some(_guard) = InferenceGuard::try_acquire() else {
            return empty_observation(
                "running",
                Some(model),
                "One observed inference is already in progress.",
            );
        };
        interpret_chat(self.probe.chat(chat_request(model)).await, model)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use super::*;

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

    fn redirected_request(
        client: reqwest::Client,
        request_body: Option<String>,
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
            match request_body {
                Some(body) => {
                    client
                        .post(source_url)
                        .header(CONTENT_TYPE, "application/json")
                        .body(body)
                        .send()
                        .await
                }
                None => client.get(source_url).send().await,
            }
        })
        .expect("receive original redirect response");
        let status = response.status().as_u16();
        let body = tauri::async_runtime::block_on(response.text()).expect("read redirect body");
        let (request, extra_source_request) = server.join().expect("join redirect source");
        let target_request = target.accept().is_ok();
        (status, body, request, extra_source_request, target_request)
    }

    #[derive(Clone)]
    struct Fake {
        models: HttpOutcome,
        chat: HttpOutcome,
        posts: Arc<Mutex<Vec<String>>>,
    }
    impl LmStudioProbe for Fake {
        fn models(&self) -> impl Future<Output = HttpOutcome> + Send {
            let v = self.models.clone();
            async move { v }
        }
        fn chat(&self, body: String) -> impl Future<Output = HttpOutcome> + Send {
            let v = self.chat.clone();
            let p = self.posts.clone();
            async move {
                p.lock().unwrap().push(body);
                v
            }
        }
    }
    fn models_body() -> String {
        r#"{"models":[{"type":"llm","publisher":"artificial","key":"example/leaf","display_name":"Leaf Model","architecture":"test","quantization":{"name":"Q4","bits_per_weight":4.5},"size_bytes":123,"params_string":"1B","max_context_length":4096,"format":"gguf","loaded_instances":[{"id":"instance-a","config":{"context_length":2048}},{"id":"instance-b","config":{"context_length":1024}}]},{"type":"embedding","key":"example/embed","display_name":"Embed","loaded_instances":[]}]}"#.to_string()
    }
    #[test]
    fn models_client_rejects_redirect_without_retry_or_target_request() {
        let (status, body, request, extra_source_request, target_request) =
            redirected_request(ReqwestLmStudioProbe::new().models_client, None);

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
        assert_eq!(
            interpret_models(HttpOutcome::Responded { status, body }).state,
            LmStudioState::RespondedUnexpectedly
        );
    }

    #[test]
    fn chat_client_rejects_redirect_without_retry_or_target_request() {
        let request_body = chat_request("example/leaf");
        let (status, body, request, extra_source_request, target_request) = redirected_request(
            ReqwestLmStudioProbe::new().chat_client,
            Some(request_body.clone()),
        );

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
            interpret_chat(HttpOutcome::Responded { status, body }, "example/leaf").state,
            "responded_unexpectedly"
        );
    }

    #[test]
    fn maps_inventory_and_instances_without_paths() {
        let s = interpret_models(HttpOutcome::Responded {
            status: 200,
            body: models_body(),
        });
        assert_eq!(s.state, LmStudioState::Available);
        assert_eq!(s.models.len(), 2);
        assert_eq!(s.models[0].loaded_instances.len(), 2);
        assert!(!s.models[1].inference_eligible);
        assert!(!serde_json::to_string(&s).unwrap().contains("/home/"));
    }
    #[test]
    fn states_are_controlled() {
        assert_eq!(
            interpret_models(HttpOutcome::ConnectionRefused).state,
            LmStudioState::NotDetected
        );
        assert_eq!(
            interpret_models(HttpOutcome::Timeout).state,
            LmStudioState::Unreachable
        );
        assert_eq!(
            interpret_models(HttpOutcome::Responded {
                status: 401,
                body: "secret".into()
            })
            .state,
            LmStudioState::AuthenticationRequired
        );
        assert_eq!(
            interpret_models(HttpOutcome::Responded {
                status: 404,
                body: "".into()
            })
            .state,
            LmStudioState::UnsupportedApi
        );
        assert_eq!(
            interpret_models(HttpOutcome::Responded {
                status: 200,
                body: "{}".into()
            })
            .state,
            LmStudioState::UnsupportedApi
        );
    }
    #[test]
    fn request_is_exact_and_stateless() {
        let v: serde_json::Value = serde_json::from_str(&chat_request("example/leaf")).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"model":"example/leaf","input":DIAGNOSTIC_PROMPT,"stream":false,"store":false,"temperature":0,"max_output_tokens":32})
        );
    }
    #[test]
    fn chat_discards_generated_and_reasoning_output() {
        let body = r#"{"output":[{"content":"PRIVATE-GENERATED"}],"reasoning":"PRIVATE-REASONING","response_id":"PRIVATE-ID","stats":{"input_tokens":10,"total_output_tokens":4,"reasoning_output_tokens":1,"tokens_per_second":2.5,"time_to_first_token_seconds":0.2}}"#;
        let o = interpret_chat(
            HttpOutcome::Responded {
                status: 200,
                body: body.into(),
            },
            "example/leaf",
        );
        let wire = serde_json::to_string(&o).unwrap();
        assert_eq!(o.state, "completed");
        assert!(!wire.contains("PRIVATE"));
    }
    #[test]
    fn authorization_and_model_kind_block_post() {
        let posts = Arc::new(Mutex::new(vec![]));
        let adapter = LmStudioAdapter::with_probe(Fake {
            models: HttpOutcome::Responded {
                status: 200,
                body: models_body(),
            },
            chat: HttpOutcome::Other,
            posts: posts.clone(),
        });
        let a = tauri::async_runtime::block_on(adapter.observe("example/leaf", false, true));
        assert_eq!(a.state, "blocked_by_execution_boundary");
        let b = tauri::async_runtime::block_on(adapter.observe("example/embed", true, true));
        assert_eq!(b.state, "model_unavailable");
        assert!(posts.lock().unwrap().is_empty());
    }
    #[test]
    fn exactly_one_authorized_post_and_no_retry() {
        let posts = Arc::new(Mutex::new(vec![]));
        let adapter = LmStudioAdapter::with_probe(Fake {
            models: HttpOutcome::Responded {
                status: 200,
                body: models_body(),
            },
            chat: HttpOutcome::Timeout,
            posts: posts.clone(),
        });
        let o = tauri::async_runtime::block_on(adapter.observe("example/leaf", true, true));
        assert_eq!(o.state, "timed_out");
        assert_eq!(posts.lock().unwrap().len(), 1);
    }
    #[test]
    fn endpoints_are_fixed_numeric_loopback() {
        assert_eq!(MODELS_ENDPOINT, "http://127.0.0.1:1234/api/v1/models");
        assert_eq!(CHAT_ENDPOINT, "http://127.0.0.1:1234/api/v1/chat");
    }
}
