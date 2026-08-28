//! Pure provider-aware deterministic diagnosis for Milestone 1T.
//!
//! The frontend supplies one bounded projection of views it already acquired.
//! This module performs no provider/OS acquisition, filesystem or process
//! inspection, clipboard action, inference, network access, or mutation. It
//! validates all variable input before producing a fixed controlled finding.

use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

const MAX_IDENTITIES: usize = 256;
const MAX_INSTANCES: usize = 512;
const MAX_ID_CHARS: usize = 256;
const MAX_VERSION_CHARS: usize = 128;
const MAX_SUPPORTED_CHANGES: u32 = 64;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticAvailableMemoryInput {
    Value,
    ControlledGap,
    SourceGap,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticOllamaRuntimeState {
    Ready,
    NotDetected,
    NotRunning,
    Unreachable,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticOllamaRuntimeInput {
    Gap,
    Observed {
        state: DiagnosticOllamaRuntimeState,
        version: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticOllamaModelSetState {
    Available,
    RespondedUnexpectedly,
    NotDetected,
    Unreachable,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticOllamaModelSetInput {
    Gap,
    Observed {
        state: DiagnosticOllamaModelSetState,
        model_ids: Vec<String>,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiagnosticLmStudioModelInput {
    pub model_id: String,
    pub inference_eligible: bool,
    pub loaded_instance_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLmStudioState {
    Available,
    NotDetected,
    Unreachable,
    AuthenticationRequired,
    UnsupportedApi,
    RespondedUnexpectedly,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticLmStudioInput {
    Gap,
    Observed {
        state: DiagnosticLmStudioState,
        models: Vec<DiagnosticLmStudioModelInput>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLlamaCppState {
    Unreachable,
    Loading,
    Ready,
    AuthenticationRequired,
    Incompatible,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticLlamaCppInput {
    Gap,
    Observed {
        state: DiagnosticLlamaCppState,
        served_model_id: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiagnosticInput {
    pub available_memory: DiagnosticAvailableMemoryInput,
    pub ollama_runtime: DiagnosticOllamaRuntimeInput,
    pub ollama_catalogue: DiagnosticOllamaModelSetInput,
    pub ollama_loaded: DiagnosticOllamaModelSetInput,
    pub lm_studio: DiagnosticLmStudioInput,
    pub llama_cpp: DiagnosticLlamaCppInput,
    pub supported_change_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticFindingSource {
    Ollama,
    LmStudio,
    LlamaCpp,
    Observation,
    CrossProvider,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DiagnosticFindingView {
    pub code: String,
    pub source: DiagnosticFindingSource,
    pub observation: String,
    pub meaning: String,
    pub safe_next_check: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisState {
    Available,
    InvalidInput,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DiagnosisResultView {
    pub state: DiagnosisState,
    pub findings: Vec<DiagnosticFindingView>,
}

fn finding(
    code: &str,
    source: DiagnosticFindingSource,
    observation: impl Into<String>,
    meaning: impl Into<String>,
    safe_next_check: impl Into<String>,
) -> DiagnosticFindingView {
    DiagnosticFindingView {
        code: code.to_string(),
        source,
        observation: observation.into(),
        meaning: meaning.into(),
        safe_next_check: safe_next_check.into(),
    }
}

fn looks_like_private_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.starts_with('/')
        || value.starts_with('~')
        || value.starts_with("\\\\")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
        || value.split(['/', '\\']).any(|component| component == "..")
        || value.contains("://")
}

fn safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.chars().count() <= MAX_ID_CHARS
        && !value.chars().any(char::is_control)
        && !looks_like_private_path(value)
}

fn valid_ids(values: &[String], maximum: usize) -> bool {
    values.len() <= maximum && values.iter().all(|value| safe_identity(value))
}

fn valid_input(input: &DiagnosticInput) -> bool {
    if input.supported_change_count > MAX_SUPPORTED_CHANGES {
        return false;
    }
    if let DiagnosticOllamaRuntimeInput::Observed {
        version: Some(v), ..
    } = &input.ollama_runtime
    {
        if v.is_empty()
            || v.trim() != v
            || v.chars().count() > MAX_VERSION_CHARS
            || v.chars().any(char::is_control)
            || looks_like_private_path(v)
        {
            return false;
        }
    }
    for source in [&input.ollama_catalogue, &input.ollama_loaded] {
        if let DiagnosticOllamaModelSetInput::Observed { model_ids, .. } = source {
            if !valid_ids(model_ids, MAX_IDENTITIES) {
                return false;
            }
        }
    }
    if let DiagnosticLmStudioInput::Observed { models, .. } = &input.lm_studio {
        if models.len() > MAX_IDENTITIES {
            return false;
        }
        let mut instances = 0usize;
        for model in models {
            if !safe_identity(&model.model_id)
                || !valid_ids(&model.loaded_instance_ids, MAX_IDENTITIES)
            {
                return false;
            }
            instances = instances.saturating_add(model.loaded_instance_ids.len());
            if instances > MAX_INSTANCES {
                return false;
            }
        }
    }
    if let DiagnosticLlamaCppInput::Observed {
        served_model_id: Some(model_id),
        ..
    } = &input.llama_cpp
    {
        if !safe_identity(model_id) {
            return false;
        }
    }
    true
}

fn available_ollama_ids(input: &DiagnosticOllamaModelSetInput) -> Option<BTreeSet<&str>> {
    match input {
        DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids,
        } => Some(model_ids.iter().map(String::as_str).collect()),
        _ => None,
    }
}

fn available_lm_models(input: &DiagnosticLmStudioInput) -> Option<&[DiagnosticLmStudioModelInput]> {
    match input {
        DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            models,
        } => Some(models),
        _ => None,
    }
}

fn source_gaps(input: &DiagnosticInput) -> Vec<&'static str> {
    let mut gaps = Vec::new();
    if matches!(
        input.available_memory,
        DiagnosticAvailableMemoryInput::SourceGap
    ) {
        gaps.push("Available memory");
    }
    if matches!(input.ollama_runtime, DiagnosticOllamaRuntimeInput::Gap) {
        gaps.push("Ollama runtime");
    }
    if matches!(input.ollama_catalogue, DiagnosticOllamaModelSetInput::Gap) {
        gaps.push("Ollama catalogue");
    }
    if matches!(input.ollama_loaded, DiagnosticOllamaModelSetInput::Gap) {
        gaps.push("Ollama loaded state");
    }
    if matches!(input.lm_studio, DiagnosticLmStudioInput::Gap) {
        gaps.push("LM Studio");
    }
    if matches!(input.llama_cpp, DiagnosticLlamaCppInput::Gap) {
        gaps.push("llama.cpp");
    }
    gaps
}

fn same_id_across_providers(input: &DiagnosticInput) -> bool {
    let mut provider_sets: Vec<BTreeSet<&str>> = Vec::new();
    if let Some(ids) = available_ollama_ids(&input.ollama_catalogue) {
        provider_sets.push(ids);
    }
    if let Some(models) = available_lm_models(&input.lm_studio) {
        provider_sets.push(models.iter().map(|model| model.model_id.as_str()).collect());
    }
    if let DiagnosticLlamaCppInput::Observed {
        state: DiagnosticLlamaCppState::Ready,
        served_model_id: Some(model_id),
    } = &input.llama_cpp
    {
        provider_sets.push(BTreeSet::from([model_id.as_str()]));
    }
    for left in 0..provider_sets.len() {
        for right in (left + 1)..provider_sets.len() {
            if provider_sets[left]
                .intersection(&provider_sets[right])
                .next()
                .is_some()
            {
                return true;
            }
        }
    }
    false
}

/// Evaluate a bounded input with fixed provider-aware rules. Pure and no-I/O.
pub fn diagnose(input: &DiagnosticInput) -> DiagnosisResultView {
    if !valid_input(input) {
        return DiagnosisResultView {
            state: DiagnosisState::InvalidInput,
            findings: Vec::new(),
        };
    }

    let mut findings = Vec::new();
    match &input.ollama_runtime {
        DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::NotDetected
                | DiagnosticOllamaRuntimeState::NotRunning,
            ..
        } => findings.push(finding(
            "ollama_not_detected",
            DiagnosticFindingSource::Ollama,
            "Nothing answered the fixed Ollama endpoint in this observation.",
            "This observation does not establish whether Ollama is installed or why it did not answer.",
            "Confirm Ollama is running using Ollama's normal controls.",
        )),
        DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Unreachable,
            ..
        } => findings.push(finding(
            "ollama_unreachable",
            DiagnosticFindingSource::Ollama,
            "The bounded local Ollama request failed or timed out.",
            "The exact transport, firewall, or proxy cause is not established.",
            "Recheck Ollama's own status and fixed local API availability.",
        )),
        DiagnosticOllamaRuntimeInput::Gap
        | DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Ready,
            ..
        } => {}
    }

    let ollama_version_responded_unexpectedly = matches!(
        &input.ollama_runtime,
        DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Ready,
            version: None,
        }
    );
    let ollama_responded_unexpectedly = ollama_version_responded_unexpectedly
        || [&input.ollama_catalogue, &input.ollama_loaded]
            .iter()
            .any(|source| {
                matches!(
                    source,
                    DiagnosticOllamaModelSetInput::Observed {
                        state: DiagnosticOllamaModelSetState::RespondedUnexpectedly,
                        ..
                    }
                )
            });
    if ollama_responded_unexpectedly {
        findings.push(finding(
            "ollama_unexpected_response",
            DiagnosticFindingSource::Ollama,
            "An Ollama response did not match the bounded catalogue or running-model contract.",
            "The response was unusable, but this does not establish a malicious service, corrupt installation, or exact incompatibility.",
            "Confirm the runtime API using Ollama documentation and explicitly Refresh after resolving it.",
        ));
    }

    let catalogue = available_ollama_ids(&input.ollama_catalogue);
    let loaded = available_ollama_ids(&input.ollama_loaded);
    if catalogue.as_ref().is_some_and(BTreeSet::is_empty) {
        findings.push(finding(
            "ollama_catalogue_empty",
            DiagnosticFindingSource::Ollama,
            "Ollama responded but reported no catalogue models.",
            "This does not establish that no model files exist or that a model was deleted.",
            "Inspect the catalogue using Ollama's normal controls.",
        ));
    }
    if let (Some(catalogue), Some(loaded)) = (&catalogue, &loaded) {
        for model_id in catalogue.difference(loaded) {
            findings.push(finding(
                "ollama_catalogued_not_loaded",
                DiagnosticFindingSource::Ollama,
                format!(
                    "Ollama model {model_id} is catalogued but was not reported in the running-model set."
                ),
                "The exact same-provider identity was absent from Ollama's valid running-model observation; this does not establish that it cannot run or why it is not loaded.",
                "Confirm loaded state in Ollama if that distinction matters.",
            ));
        }
    }

    match &input.lm_studio {
        DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::NotDetected,
            ..
        } => findings.push(finding(
            "lm_studio_not_detected",
            DiagnosticFindingSource::LmStudio,
            "LM Studio native REST v1 did not answer with a usable observation.",
            "This does not establish whether LM Studio is installed, stopped, or why it did not answer.",
            "Confirm LM Studio's server is running through its normal controls.",
        )),
        DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Unreachable,
            ..
        } => findings.push(finding(
            "lm_studio_unreachable",
            DiagnosticFindingSource::LmStudio,
            "The bounded local LM Studio native REST v1 request failed or timed out.",
            "The exact transport or service cause is not established.",
            "Confirm LM Studio's server is running through its normal controls.",
        )),
        DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::AuthenticationRequired,
            ..
        } => findings.push(finding(
            "lm_studio_authentication_required",
            DiagnosticFindingSource::LmStudio,
            "LM Studio answered but requires an unsupported authentication contract.",
            "AI Engine Room does not collect credentials or change the server's security settings.",
            "Review LM Studio's server authentication setting.",
        )),
        DiagnosticLmStudioInput::Observed {
            state:
                DiagnosticLmStudioState::UnsupportedApi
                | DiagnosticLmStudioState::RespondedUnexpectedly,
            ..
        } => findings.push(finding(
            "lm_studio_native_v1_required",
            DiagnosticFindingSource::LmStudio,
            "The fixed endpoint did not provide the required native REST v1 model contract.",
            "This does not establish the exact installed version or why the contract was unavailable.",
            "Confirm the supported native REST v1 configuration using LM Studio documentation.",
        )),
        DiagnosticLmStudioInput::Gap
        | DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            ..
        } => {}
    }

    let lm_models = available_lm_models(&input.lm_studio);
    if catalogue.as_ref().is_some_and(BTreeSet::is_empty)
        && lm_models.is_some_and(|models| !models.iter().any(|model| model.inference_eligible))
    {
        findings.push(finding(
            "no_eligible_model_for_observed_inference",
            DiagnosticFindingSource::Observation,
            "Valid Ollama and LM Studio observations reported no model that meets the existing observed-inference eligibility contract.",
            "No provider action was taken, and this does not establish why no eligible model was reported.",
            "Review provider catalogue, type, and loaded state using provider controls.",
        ));
    }
    if let Some(models) = lm_models {
        for model in models
            .iter()
            .filter(|model| model.inference_eligible && model.loaded_instance_ids.is_empty())
        {
            findings.push(finding(
                "lm_studio_model_not_loaded",
                DiagnosticFindingSource::LmStudio,
                format!(
                    "LM Studio model {} is eligible for observed inference but has no reported loaded instance.",
                    model.model_id
                ),
                "LM Studio reported the model but no loaded instance in this observation; this does not establish incompatibility or insufficient memory.",
                "If you later choose observed inference, review the existing JIT-load disclosure first.",
            ));
        }
    }

    match &input.llama_cpp {
        DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Loading,
            ..
        } => findings.push(finding(
            "llama_cpp_loading",
            DiagnosticFindingSource::LlamaCpp,
            "The passive llama.cpp health endpoint reported loading.",
            "AI Engine Room does not estimate loading progress or completion.",
            "Wait using the server's normal controls and explicitly Refresh later.",
        )),
        DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Ready,
            served_model_id: Some(model_id),
        } => findings.push(finding(
            "llama_cpp_passive_only_ready",
            DiagnosticFindingSource::LlamaCpp,
            format!(
                "llama.cpp passively reported one served model: {model_id}."
            ),
            "The model is served under the bounded passive contract; AI Engine Room does not establish inference eligibility, router support, or compute placement.",
            "Use the server's own controls for further checks; AI Engine Room will not run llama.cpp inference.",
        )),
        _ => {}
    }

    let gaps = source_gaps(input);
    if !gaps.is_empty() {
        findings.push(finding(
            "observation_source_gap",
            DiagnosticFindingSource::Observation,
            format!("Source gaps in this observation: {}.", gaps.join(", ")),
            "Those sources could not support a factual set or value comparison; a gap is not an empty set or numeric zero.",
            "Check the affected source and explicitly Refresh later.",
        ));
    }
    if input.supported_change_count > 0 {
        findings.push(finding(
            "explicit_observation_changed",
            DiagnosticFindingSource::Observation,
            format!(
                "{} supported factual change{} differed from the previous explicit observation.",
                input.supported_change_count,
                if input.supported_change_count == 1 {
                    ""
                } else {
                    "s"
                }
            ),
            "The listed facts differed; AI Engine Room does not infer why they changed.",
            "Review the affected provider or resource section.",
        ));
    }
    if same_id_across_providers(input) {
        findings.push(finding(
            "cross_provider_model_equivalence_unavailable",
            DiagnosticFindingSource::CrossProvider,
            "The same-looking model ID appeared in more than one provider-qualified source.",
            "AI Engine Room has not established that those models, weights, configurations, quantization, or reporting semantics are equivalent.",
            "Review each provider entry separately.",
        ));
    }

    let mut seen = HashSet::new();
    findings.retain(|item| seen.insert((item.code.clone(), item.observation.clone())));
    DiagnosisResultView {
        state: DiagnosisState::Available,
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> DiagnosticInput {
        DiagnosticInput {
            available_memory: DiagnosticAvailableMemoryInput::Value,
            ollama_runtime: DiagnosticOllamaRuntimeInput::Observed {
                state: DiagnosticOllamaRuntimeState::Ready,
                version: Some("1.0.0".to_string()),
            },
            ollama_catalogue: DiagnosticOllamaModelSetInput::Observed {
                state: DiagnosticOllamaModelSetState::Available,
                model_ids: vec!["ollama:one".to_string()],
            },
            ollama_loaded: DiagnosticOllamaModelSetInput::Observed {
                state: DiagnosticOllamaModelSetState::Available,
                model_ids: vec!["ollama:one".to_string()],
            },
            lm_studio: DiagnosticLmStudioInput::Observed {
                state: DiagnosticLmStudioState::Available,
                models: vec![DiagnosticLmStudioModelInput {
                    model_id: "lm/one".to_string(),
                    inference_eligible: true,
                    loaded_instance_ids: vec!["instance-one".to_string()],
                }],
            },
            llama_cpp: DiagnosticLlamaCppInput::Observed {
                state: DiagnosticLlamaCppState::Ready,
                served_model_id: Some("llama-one".to_string()),
            },
            supported_change_count: 0,
        }
    }

    fn codes(input: &DiagnosticInput) -> Vec<String> {
        diagnose(input)
            .findings
            .into_iter()
            .map(|finding| finding.code)
            .collect()
    }

    fn expected_finding(
        code: &str,
        source: DiagnosticFindingSource,
        observation: &str,
        meaning: &str,
        safe_next_check: &str,
    ) -> DiagnosticFindingView {
        DiagnosticFindingView {
            code: code.to_string(),
            source,
            observation: observation.to_string(),
            meaning: meaning.to_string(),
            safe_next_check: safe_next_check.to_string(),
        }
    }

    fn assert_exact_finding(input: &DiagnosticInput, expected: DiagnosticFindingView) {
        let result = diagnose(input);
        let matching = result
            .findings
            .iter()
            .filter(|finding| finding.code == expected.code)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "finding count for {}", expected.code);
        assert_eq!(matching[0], &expected);
    }

    fn assert_no_finding(input: &DiagnosticInput, code: &str) {
        assert!(
            !diagnose(input)
                .findings
                .iter()
                .any(|finding| finding.code == code),
            "unexpected finding: {code}"
        );
    }

    #[test]
    fn every_rule_has_an_exact_controlled_user_visible_contract() {
        let mut ollama_not_detected = input();
        ollama_not_detected.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::NotDetected,
            version: None,
        };
        assert_exact_finding(
            &ollama_not_detected,
            expected_finding(
                "ollama_not_detected",
                DiagnosticFindingSource::Ollama,
                "Nothing answered the fixed Ollama endpoint in this observation.",
                "This observation does not establish whether Ollama is installed or why it did not answer.",
                "Confirm Ollama is running using Ollama's normal controls.",
            ),
        );

        let mut ollama_unreachable = input();
        ollama_unreachable.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Unreachable,
            version: None,
        };
        assert_exact_finding(
            &ollama_unreachable,
            expected_finding(
                "ollama_unreachable",
                DiagnosticFindingSource::Ollama,
                "The bounded local Ollama request failed or timed out.",
                "The exact transport, firewall, or proxy cause is not established.",
                "Recheck Ollama's own status and fixed local API availability.",
            ),
        );

        let mut ollama_unexpected = input();
        ollama_unexpected.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::RespondedUnexpectedly,
            model_ids: Vec::new(),
        };
        assert_exact_finding(
            &ollama_unexpected,
            expected_finding(
                "ollama_unexpected_response",
                DiagnosticFindingSource::Ollama,
                "An Ollama response did not match the bounded catalogue or running-model contract.",
                "The response was unusable, but this does not establish a malicious service, corrupt installation, or exact incompatibility.",
                "Confirm the runtime API using Ollama documentation and explicitly Refresh after resolving it.",
            ),
        );

        let mut ollama_empty = input();
        ollama_empty.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        assert_exact_finding(
            &ollama_empty,
            expected_finding(
                "ollama_catalogue_empty",
                DiagnosticFindingSource::Ollama,
                "Ollama responded but reported no catalogue models.",
                "This does not establish that no model files exist or that a model was deleted.",
                "Inspect the catalogue using Ollama's normal controls.",
            ),
        );

        let mut ollama_not_loaded = input();
        ollama_not_loaded.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        assert_exact_finding(
            &ollama_not_loaded,
            expected_finding(
                "ollama_catalogued_not_loaded",
                DiagnosticFindingSource::Ollama,
                "Ollama model ollama:one is catalogued but was not reported in the running-model set.",
                "The exact same-provider identity was absent from Ollama's valid running-model observation; this does not establish that it cannot run or why it is not loaded.",
                "Confirm loaded state in Ollama if that distinction matters.",
            ),
        );

        let mut lm_not_detected = input();
        lm_not_detected.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::NotDetected,
            models: Vec::new(),
        };
        assert_exact_finding(
            &lm_not_detected,
            expected_finding(
                "lm_studio_not_detected",
                DiagnosticFindingSource::LmStudio,
                "LM Studio native REST v1 did not answer with a usable observation.",
                "This does not establish whether LM Studio is installed, stopped, or why it did not answer.",
                "Confirm LM Studio's server is running through its normal controls.",
            ),
        );

        let mut lm_unreachable = input();
        lm_unreachable.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Unreachable,
            models: Vec::new(),
        };
        assert_exact_finding(
            &lm_unreachable,
            expected_finding(
                "lm_studio_unreachable",
                DiagnosticFindingSource::LmStudio,
                "The bounded local LM Studio native REST v1 request failed or timed out.",
                "The exact transport or service cause is not established.",
                "Confirm LM Studio's server is running through its normal controls.",
            ),
        );

        let mut lm_auth = input();
        lm_auth.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::AuthenticationRequired,
            models: Vec::new(),
        };
        assert_exact_finding(
            &lm_auth,
            expected_finding(
                "lm_studio_authentication_required",
                DiagnosticFindingSource::LmStudio,
                "LM Studio answered but requires an unsupported authentication contract.",
                "AI Engine Room does not collect credentials or change the server's security settings.",
                "Review LM Studio's server authentication setting.",
            ),
        );

        let mut lm_native_v1 = input();
        lm_native_v1.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::UnsupportedApi,
            models: Vec::new(),
        };
        assert_exact_finding(
            &lm_native_v1,
            expected_finding(
                "lm_studio_native_v1_required",
                DiagnosticFindingSource::LmStudio,
                "The fixed endpoint did not provide the required native REST v1 model contract.",
                "This does not establish the exact installed version or why the contract was unavailable.",
                "Confirm the supported native REST v1 configuration using LM Studio documentation.",
            ),
        );

        let mut no_eligible = input();
        no_eligible.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        no_eligible.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        no_eligible.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            models: Vec::new(),
        };
        assert_exact_finding(
            &no_eligible,
            expected_finding(
                "no_eligible_model_for_observed_inference",
                DiagnosticFindingSource::Observation,
                "Valid Ollama and LM Studio observations reported no model that meets the existing observed-inference eligibility contract.",
                "No provider action was taken, and this does not establish why no eligible model was reported.",
                "Review provider catalogue, type, and loaded state using provider controls.",
            ),
        );

        let mut lm_not_loaded = input();
        if let DiagnosticLmStudioInput::Observed { models, .. } = &mut lm_not_loaded.lm_studio {
            models[0].loaded_instance_ids.clear();
        }
        assert_exact_finding(
            &lm_not_loaded,
            expected_finding(
                "lm_studio_model_not_loaded",
                DiagnosticFindingSource::LmStudio,
                "LM Studio model lm/one is eligible for observed inference but has no reported loaded instance.",
                "LM Studio reported the model but no loaded instance in this observation; this does not establish incompatibility or insufficient memory.",
                "If you later choose observed inference, review the existing JIT-load disclosure first.",
            ),
        );

        let mut llama_loading = input();
        llama_loading.llama_cpp = DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Loading,
            served_model_id: None,
        };
        assert_exact_finding(
            &llama_loading,
            expected_finding(
                "llama_cpp_loading",
                DiagnosticFindingSource::LlamaCpp,
                "The passive llama.cpp health endpoint reported loading.",
                "AI Engine Room does not estimate loading progress or completion.",
                "Wait using the server's normal controls and explicitly Refresh later.",
            ),
        );

        assert_exact_finding(
            &input(),
            expected_finding(
                "llama_cpp_passive_only_ready",
                DiagnosticFindingSource::LlamaCpp,
                "llama.cpp passively reported one served model: llama-one.",
                "The model is served under the bounded passive contract; AI Engine Room does not establish inference eligibility, router support, or compute placement.",
                "Use the server's own controls for further checks; AI Engine Room will not run llama.cpp inference.",
            ),
        );

        let mut source_gap = input();
        source_gap.available_memory = DiagnosticAvailableMemoryInput::SourceGap;
        assert_exact_finding(
            &source_gap,
            expected_finding(
                "observation_source_gap",
                DiagnosticFindingSource::Observation,
                "Source gaps in this observation: Available memory.",
                "Those sources could not support a factual set or value comparison; a gap is not an empty set or numeric zero.",
                "Check the affected source and explicitly Refresh later.",
            ),
        );

        let mut changed = input();
        changed.supported_change_count = 2;
        assert_exact_finding(
            &changed,
            expected_finding(
                "explicit_observation_changed",
                DiagnosticFindingSource::Observation,
                "2 supported factual changes differed from the previous explicit observation.",
                "The listed facts differed; AI Engine Room does not infer why they changed.",
                "Review the affected provider or resource section.",
            ),
        );

        let mut cross_provider = input();
        cross_provider.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: vec!["shared/model".to_string()],
        };
        cross_provider.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: vec!["shared/model".to_string()],
        };
        cross_provider.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            models: vec![DiagnosticLmStudioModelInput {
                model_id: "shared/model".to_string(),
                inference_eligible: true,
                loaded_instance_ids: vec!["instance-one".to_string()],
            }],
        };
        assert_exact_finding(
            &cross_provider,
            expected_finding(
                "cross_provider_model_equivalence_unavailable",
                DiagnosticFindingSource::CrossProvider,
                "The same-looking model ID appeared in more than one provider-qualified source.",
                "AI Engine Room has not established that those models, weights, configurations, quantization, or reporting semantics are equivalent.",
                "Review each provider entry separately.",
            ),
        );
    }

    #[test]
    fn every_rule_has_a_material_insufficient_evidence_non_trigger() {
        assert_no_finding(&input(), "ollama_not_detected");

        let mut not_unreachable = input();
        not_unreachable.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::NotDetected,
            version: None,
        };
        assert_no_finding(&not_unreachable, "ollama_unreachable");

        assert_no_finding(&input(), "ollama_unexpected_response");
        assert_no_finding(&input(), "ollama_catalogue_empty");

        let mut loaded_gap = input();
        loaded_gap.ollama_loaded = DiagnosticOllamaModelSetInput::Gap;
        assert_no_finding(&loaded_gap, "ollama_catalogued_not_loaded");

        assert_no_finding(&input(), "lm_studio_not_detected");
        assert_no_finding(&input(), "lm_studio_unreachable");
        assert_no_finding(&input(), "lm_studio_authentication_required");
        assert_no_finding(&input(), "lm_studio_native_v1_required");

        let mut eligibility_gap = input();
        eligibility_gap.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        eligibility_gap.lm_studio = DiagnosticLmStudioInput::Gap;
        assert_no_finding(&eligibility_gap, "no_eligible_model_for_observed_inference");

        let mut ineligible_unloaded = input();
        if let DiagnosticLmStudioInput::Observed { models, .. } = &mut ineligible_unloaded.lm_studio
        {
            models[0].inference_eligible = false;
            models[0].loaded_instance_ids.clear();
        }
        assert_no_finding(&ineligible_unloaded, "lm_studio_model_not_loaded");

        let mut llama_without_id = input();
        llama_without_id.llama_cpp = DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Ready,
            served_model_id: None,
        };
        assert_no_finding(&llama_without_id, "llama_cpp_loading");
        assert_no_finding(&llama_without_id, "llama_cpp_passive_only_ready");

        let mut controlled_memory_gap = input();
        controlled_memory_gap.available_memory = DiagnosticAvailableMemoryInput::ControlledGap;
        assert_no_finding(&controlled_memory_gap, "observation_source_gap");

        assert_no_finding(&input(), "explicit_observation_changed");

        let mut same_provider_only = input();
        same_provider_only.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: vec!["shared/model".to_string()],
        };
        same_provider_only.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: vec!["shared/model".to_string()],
        };
        assert_no_finding(
            &same_provider_only,
            "cross_provider_model_equivalence_unavailable",
        );
    }

    #[test]
    fn multi_rule_order_remains_fixed_and_deterministic() {
        let mut candidate = input();
        candidate.available_memory = DiagnosticAvailableMemoryInput::SourceGap;
        candidate.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::NotDetected,
            version: None,
        };
        candidate.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        candidate.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        candidate.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::AuthenticationRequired,
            models: Vec::new(),
        };
        candidate.llama_cpp = DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Loading,
            served_model_id: None,
        };
        candidate.supported_change_count = 1;

        let expected = vec![
            "ollama_not_detected",
            "ollama_catalogue_empty",
            "lm_studio_authentication_required",
            "llama_cpp_loading",
            "observation_source_gap",
            "explicit_observation_changed",
        ];
        assert_eq!(codes(&candidate), expected);
        assert_eq!(codes(&candidate), expected);
    }

    #[test]
    fn normal_available_input_has_only_the_passive_llama_finding() {
        assert_eq!(codes(&input()), vec!["llama_cpp_passive_only_ready"]);
    }

    #[test]
    fn ollama_controlled_state_rules_are_exact_and_do_not_infer_root_cause() {
        let mut not_detected = input();
        not_detected.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::NotDetected,
            version: None,
        };
        let result = diagnose(&not_detected);
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.code == "ollama_not_detected"));
        let text = serde_json::to_string(&result).unwrap();
        assert!(!text.contains("crashed"));

        let mut unreachable = input();
        unreachable.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Unreachable,
            version: None,
        };
        assert!(codes(&unreachable).contains(&"ollama_unreachable".to_string()));
    }

    #[test]
    fn ollama_unexpected_empty_and_catalogued_not_loaded_rules_require_valid_sets() {
        let mut unexpected = input();
        unexpected.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::RespondedUnexpectedly,
            model_ids: Vec::new(),
        };
        unexpected.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::RespondedUnexpectedly,
            model_ids: Vec::new(),
        };
        assert_eq!(
            codes(&unexpected)
                .iter()
                .filter(|code| code.as_str() == "ollama_unexpected_response")
                .count(),
            1
        );

        let mut version_unexpected = input();
        version_unexpected.ollama_runtime = DiagnosticOllamaRuntimeInput::Observed {
            state: DiagnosticOllamaRuntimeState::Ready,
            version: None,
        };
        assert!(codes(&version_unexpected).contains(&"ollama_unexpected_response".to_string()));

        let mut empty = input();
        empty.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        assert!(codes(&empty).contains(&"ollama_catalogue_empty".to_string()));

        let mut not_loaded = input();
        not_loaded.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        assert!(codes(&not_loaded).contains(&"ollama_catalogued_not_loaded".to_string()));
        not_loaded.ollama_loaded = DiagnosticOllamaModelSetInput::Gap;
        assert!(!codes(&not_loaded).contains(&"ollama_catalogued_not_loaded".to_string()));
    }

    #[test]
    fn lm_studio_state_rules_map_only_existing_controlled_states() {
        for (state, expected) in [
            (
                DiagnosticLmStudioState::NotDetected,
                "lm_studio_not_detected",
            ),
            (
                DiagnosticLmStudioState::Unreachable,
                "lm_studio_unreachable",
            ),
            (
                DiagnosticLmStudioState::AuthenticationRequired,
                "lm_studio_authentication_required",
            ),
            (
                DiagnosticLmStudioState::UnsupportedApi,
                "lm_studio_native_v1_required",
            ),
            (
                DiagnosticLmStudioState::RespondedUnexpectedly,
                "lm_studio_native_v1_required",
            ),
        ] {
            let mut candidate = input();
            candidate.lm_studio = DiagnosticLmStudioInput::Observed {
                state,
                models: Vec::new(),
            };
            assert!(codes(&candidate).contains(&expected.to_string()));
        }
    }

    #[test]
    fn eligibility_and_lm_loaded_rules_fail_closed() {
        let mut none = input();
        none.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        none.ollama_loaded = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: Vec::new(),
        };
        none.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            models: Vec::new(),
        };
        assert!(codes(&none).contains(&"no_eligible_model_for_observed_inference".to_string()));
        none.lm_studio = DiagnosticLmStudioInput::Gap;
        assert!(!codes(&none).contains(&"no_eligible_model_for_observed_inference".to_string()));

        let mut unloaded = input();
        if let DiagnosticLmStudioInput::Observed { models, .. } = &mut unloaded.lm_studio {
            models[0].loaded_instance_ids.clear();
        }
        assert!(codes(&unloaded).contains(&"lm_studio_model_not_loaded".to_string()));
    }

    #[test]
    fn llama_rules_preserve_loading_and_passive_only_semantics() {
        let mut loading = input();
        loading.llama_cpp = DiagnosticLlamaCppInput::Observed {
            state: DiagnosticLlamaCppState::Loading,
            served_model_id: None,
        };
        assert!(codes(&loading).contains(&"llama_cpp_loading".to_string()));
        let ready = diagnose(&input());
        let finding = ready
            .findings
            .iter()
            .find(|finding| finding.code == "llama_cpp_passive_only_ready")
            .unwrap();
        assert!(finding
            .meaning
            .contains("does not establish inference eligibility"));
    }

    #[test]
    fn source_gap_and_explicit_change_rules_are_controlled_and_deterministic() {
        let mut candidate = input();
        candidate.available_memory = DiagnosticAvailableMemoryInput::SourceGap;
        candidate.ollama_catalogue = DiagnosticOllamaModelSetInput::Gap;
        candidate.supported_change_count = 2;
        let first = diagnose(&candidate);
        let second = diagnose(&candidate);
        assert_eq!(first, second);
        assert!(codes(&candidate).contains(&"observation_source_gap".to_string()));
        assert!(codes(&candidate).contains(&"explicit_observation_changed".to_string()));
        let text = serde_json::to_string(&first).unwrap();
        assert!(text.contains("does not infer why"));
        assert!(!text.contains("health score"));
    }

    #[test]
    fn same_looking_cross_provider_ids_are_qualified_not_merged() {
        let mut candidate = input();
        candidate.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: vec!["shared/model".to_string()],
        };
        candidate.lm_studio = DiagnosticLmStudioInput::Observed {
            state: DiagnosticLmStudioState::Available,
            models: vec![DiagnosticLmStudioModelInput {
                model_id: "shared/model".to_string(),
                inference_eligible: true,
                loaded_instance_ids: Vec::new(),
            }],
        };
        let result = diagnose(&candidate);
        let finding = result
            .findings
            .iter()
            .find(|finding| finding.code == "cross_provider_model_equivalence_unavailable")
            .unwrap();
        assert!(finding.meaning.contains("not established"));
    }

    #[test]
    fn malformed_or_unbounded_input_returns_only_controlled_invalid_state() {
        for invalid_id in ["/home/owner/model", "../private", "https://private"] {
            let mut candidate = input();
            candidate.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
                state: DiagnosticOllamaModelSetState::Available,
                model_ids: vec![invalid_id.to_string()],
            };
            assert_eq!(
                diagnose(&candidate),
                DiagnosisResultView {
                    state: DiagnosisState::InvalidInput,
                    findings: Vec::new(),
                }
            );
        }
        let mut too_many = input();
        too_many.ollama_catalogue = DiagnosticOllamaModelSetInput::Observed {
            state: DiagnosticOllamaModelSetState::Available,
            model_ids: (0..=MAX_IDENTITIES)
                .map(|index| format!("model-{index}"))
                .collect(),
        };
        assert_eq!(diagnose(&too_many).state, DiagnosisState::InvalidInput);
    }

    #[test]
    fn output_uses_only_controlled_hierarchy_and_prohibited_claims_are_absent() {
        let mut candidate = input();
        candidate.supported_change_count = 1;
        let result = diagnose(&candidate);
        for finding in result.findings {
            assert!(!finding.observation.is_empty());
            assert!(!finding.meaning.is_empty());
            assert!(!finding.safe_next_check.is_empty());
            let text = format!(
                "{} {} {}",
                finding.observation, finding.meaning, finding.safe_next_check
            )
            .to_lowercase();
            for prohibited in [
                "memory pressure",
                "model fit",
                "headroom",
                "benchmark result",
                "ran locally",
                "accelerator placement",
            ] {
                assert!(!text.contains(prohibited), "unexpected claim: {prohibited}");
            }
        }
    }
}
