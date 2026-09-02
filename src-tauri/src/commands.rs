//! Narrow Tauri command surface for Milestone 1A.
//!
//! The OS observation commands are `current_snapshot` (the structured view of
//! the latest metric states) and `report_preview` (the report-safe
//! human-readable preview). Each command performs its own snapshot acquisition;
//! the Report is not coupled to the latest Overview observation.
//! Platform-specific operating-system acquisition stays behind provider traits.
//!
//! The fixture snapshot that feeds the skeleton is built from the aer-core
//! mock providers and is compiled only under the `dev-fixtures` feature, so the
//! ordinary production/native build never compiles mock provider code. With the
//! feature off, supported production targets use their cfg-gated provider and
//! unsupported targets return an empty snapshot.

use crate::report_save::{
    ReportPreviewError, ReportPreviewResponse, ReportSaveResult, ReportSaveState,
};
use crate::view::{
    inference_observation_view, loaded_models_view, model_inventory_view, runtime_status_view,
    snapshot_view, InferenceObservationRequest, InferenceObservationView, LoadedModelSetView,
    ModelInventoryView, ResourceContextView, RuntimeStatusView, SnapshotView,
};
use aer_core::{project, render_preview, sanitize, Snapshot};

/// The structured view of the current metric snapshot.
#[tauri::command]
pub fn current_snapshot() -> SnapshotView {
    snapshot_view(&build_snapshot())
}

/// One bounded native CPU-architecture observation. This categorical machine
/// metadata has no numeric metric or Report projection.
#[tauri::command]
pub fn current_machine_context() -> crate::machine::MachineContextView {
    crate::machine::current_machine_context()
}

/// The report-safe human-readable preview. Exposes only the allow-listed,
/// sanitized, report-safe representation — no internal ids, raw errors, or
/// diagnostics.
#[tauri::command]
pub fn report_preview(
    state: tauri::State<'_, ReportSaveState>,
) -> Result<ReportPreviewResponse, ReportPreviewError> {
    state.retain_preview(report_preview_of(&build_snapshot()))
}

/// Save only the retained report generation selected by the WebView. The
/// backend owns the native dialog and filesystem boundary; neither a path nor
/// report text is accepted over IPC. The blocking dialog and file operation
/// execute off the main thread.
#[tauri::command]
pub async fn save_report(
    generation: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, ReportSaveState>,
) -> Result<ReportSaveResult, ()> {
    let state = state.inner().clone();
    Ok(tauri::async_runtime::spawn_blocking(move || {
        crate::report_save::save_native(state, app, generation)
    })
    .await
    .unwrap_or(ReportSaveResult::Failed))
}

/// The current runtime detection status for the supported AI runtime (Ollama
/// in 1C): controlled source availability, the verbatim version the runtime
/// reported when present, and pre-computed controlled status text. No raw
/// errors, URLs, hostnames, ports, status codes, or process information cross
/// this boundary. The version is runtime metadata, not a metric, so it is not
/// routed through the snapshot/report path. Async so the bounded local HTTP
/// probe runs without blocking; cross-platform (local HTTP, not cfg-gated).
#[tauri::command]
pub async fn current_runtime_status() -> RuntimeStatusView {
    let adapter = crate::runtime::ollama::OllamaAdapter::new();
    let status = adapter.runtime_status().await;
    runtime_status_view(&status)
}

/// The current model inventory for the supported AI runtime (Ollama in 1D):
/// the read-only catalogue Ollama reports at the documented local `/api/tags`
/// endpoint, projected to a controlled DTO with pre-computed inventory text. No
/// raw errors, URLs, hostnames, ports, status codes, `digest`, or
/// `remote_host` cross this boundary. The inventory is runtime metadata, not a
/// metric, so it is not routed through the snapshot/report path. Read-only — no
/// loading, generation, pull, or delete. Async so the bounded local HTTP probe
/// runs without blocking; cross-platform (local HTTP, not cfg-gated, decoupled
/// from the Linux OS-provider layer), exactly like `current_runtime_status`.
#[tauri::command]
pub async fn current_model_inventory() -> ModelInventoryView {
    let adapter = crate::runtime::ollama::OllamaAdapter::new();
    let inventory = adapter.inventory().await;
    model_inventory_view(&inventory)
}

/// The currently loaded (runtime-resident) models for the supported AI runtime
/// (Ollama in 1E): the read-only set Ollama reports at the documented local
/// `/api/ps` endpoint, projected to a controlled DTO with pre-computed
/// loaded-model text. No raw errors, URLs, hostnames, ports, status codes,
/// `digest`, or `remote_host` cross this boundary. `expires_at` is carried
/// internally but is not displayed. The loaded-model set is runtime metadata,
/// not a metric, so it is not routed through the snapshot/report path, and it
/// is distinct from the 1D catalogue (`current_model_inventory`). Read-only —
/// no loading, unloading, generation, pull, or delete. Async so the bounded
/// local HTTP probe runs without blocking; cross-platform (local HTTP, not
/// cfg-gated), exactly like `current_runtime_status`.
#[tauri::command]
pub async fn current_loaded_models() -> LoadedModelSetView {
    let adapter = crate::runtime::ollama::OllamaAdapter::new();
    let set = adapter.loaded_models_status().await;
    loaded_models_view(&set)
}

/// One read-only native REST v1 catalogue probe supplies LM Studio detection,
/// inventory, and loaded-instance state. The adapter is fixed to numeric
/// loopback and follows no redirects.
#[tauri::command]
pub async fn current_lm_studio_snapshot() -> crate::runtime::lm_studio::LmStudioSnapshot {
    crate::runtime::lm_studio::LmStudioAdapter::new()
        .snapshot()
        .await
}

/// One passive traditional single-model llama-server snapshot. The adapter
/// uses only fixed numeric-loopback GET routes, follows no redirects, and
/// returns a served model only after its provider-reported served-model ID
/// passes the safe-identity privacy gate. It performs no inference or
/// provider/model management.
#[tauri::command]
pub async fn current_llama_cpp_snapshot() -> crate::runtime::llama_cpp::LlamaCppSnapshot {
    crate::runtime::llama_cpp::LlamaCppAdapter::new()
        .snapshot()
        .await
}

/// Compose the resource-context explanation from the views the frontend already
/// acquired (Milestone 1F). PURE: performs no I/O — does not acquire OS memory
/// and does not call `/api/ps`. The available-memory side comes from
/// `current_snapshot` (the single platform-provider acquisition) and the loaded-model
/// side comes from `current_loaded_models` (the single `/api/ps` acquisition); the
/// frontend passes those already-resolved views in, and this command only composes
/// the controlled explanation and nonnumeric resource-evidence distinctions so
/// technical interpretation stays in Rust, not Svelte. Synchronous (no I/O),
/// so it does not need to be `async`. It calls
/// neither `build_snapshot()` nor `OllamaAdapter` and adds zero acquisition of
/// either source.
#[tauri::command]
pub fn compose_resource_context(
    snapshot: SnapshotView,
    loaded: LoadedModelSetView,
) -> ResourceContextView {
    crate::view::compose_resource_context(&snapshot, &loaded)
}

/// Evaluate deterministic provider-aware findings from one bounded bundle of
/// views the frontend already acquired. PURE: performs no provider/OS call,
/// filesystem/process inspection, clipboard action, inference, network access,
/// or mutation. Malformed or unbounded identity input fails closed to the
/// controlled `invalid_input` state.
#[tauri::command]
pub fn diagnose_observation(
    input: crate::diagnosis::DiagnosticInput,
) -> crate::diagnosis::DiagnosisResultView {
    crate::diagnosis::diagnose(&input)
}

/// Run one observed inference against the supported AI runtime (Ollama in 1G):
/// a single non-streaming `POST /api/generate` carrying a fixed synthetic
/// diagnostic prompt to the model the user explicitly selected from the 1D
/// catalogue. Per-run authorization is required: `execution_boundary_acknowledged
/// == false` blocks the run with no POST. The command validates the model
/// identifier, checks the acknowledgement, confirms the model is currently listed
/// in the catalogue (one `GET /api/tags`), enforces a single-run guard, then
/// issues at most one POST (no retry). The generated response text is discarded;
/// no raw errors, endpoints, host/ports, status codes, or generated text cross
/// the boundary — only the controlled DTO with Ollama-reported counts/durations,
/// an Engine-Room-calculated rate, and pre-computed controlled text. The
/// execution boundary is always `"undetermined"`; Engine Room never claims the
/// model ran locally. Async so the bounded local HTTP probes run without
/// blocking; cross-platform (local HTTP, not cfg-gated).
#[tauri::command]
pub async fn run_inference_observation(
    request: InferenceObservationRequest,
) -> InferenceObservationView {
    if request.provider == "lm_studio" {
        let observation = crate::runtime::lm_studio::LmStudioAdapter::new()
            .observe(
                &request.model,
                request.execution_boundary_acknowledged,
                request.provider_behavior_acknowledged,
            )
            .await;
        return InferenceObservationView {
            state: serde_json::from_value(serde_json::Value::String(observation.state.to_string()))
                .unwrap_or(
                    crate::runtime::inference::InferenceObservationState::RespondedUnexpectedly,
                ),
            provider: observation.provider.to_string(),
            model: observation.model,
            diagnostic_profile: observation.diagnostic_profile.map(str::to_string),
            execution_boundary: observation.compute_location.to_string(),
            prompt_eval_count: None,
            eval_count: None,
            load_duration_ns: None,
            prompt_eval_duration_ns: None,
            eval_duration_ns: None,
            total_duration_ns: None,
            generation_tokens_per_second: None,
            input_tokens: observation.input_tokens,
            total_output_tokens: observation.total_output_tokens,
            reasoning_output_tokens: observation.reasoning_output_tokens,
            provider_reported_tokens_per_second: observation.tokens_per_second,
            time_to_first_token_seconds: observation.time_to_first_token_seconds,
            model_load_time_seconds: observation.model_load_time_seconds,
            interpretation: observation.interpretation.to_string(),
            why_it_matters: observation.why_it_matters.to_string(),
            limitations: observation.limitations.to_string(),
        };
    }
    if request.provider != "ollama" {
        let observation = crate::runtime::inference::run_inference_observation_with_adapter(
            &crate::runtime::ollama::OllamaAdapter::new(),
            "",
            false,
        )
        .await;
        return inference_observation_view(&observation);
    }
    let adapter = crate::runtime::ollama::OllamaAdapter::new();
    let observation = crate::runtime::inference::run_inference_observation_with_adapter(
        &adapter,
        &request.model,
        request.execution_boundary_acknowledged,
    )
    .await;
    inference_observation_view(&observation)
}

/// Project a snapshot to the report-safe model, run the defence-in-depth
/// sanitizer, and render the human-readable preview. Pure: takes the internal
/// snapshot and returns only the report-safe string.
pub(crate) fn report_preview_of(snapshot: &Snapshot) -> String {
    let mut report = project(snapshot);
    sanitize(&mut report);
    render_preview(&report)
}

/// Build the snapshot that feeds the application. With `dev-fixtures`, this
/// uses clearly artificial aer-core mock providers on targets without the
/// established Linux production path. Production Linux and Windows builds use
/// their respective narrow operating-system providers.
#[cfg(all(feature = "dev-fixtures", not(target_os = "linux")))]
fn build_snapshot() -> Snapshot {
    use aer_core::mock::{MockOsProvider, MockRuntimeAdapter};
    use aer_core::{
        explain, AiRuntimeAdapter, MetricDescriptor, MetricId, MetricResult, OsMetricsProvider,
        SnapshotEntry, SourceAvailability, SourceClass, Timestamp, Unit,
    };

    fn entry(descriptor: MetricDescriptor, result: MetricResult) -> SnapshotEntry {
        let explanation = explain(&descriptor, &result);
        SnapshotEntry {
            descriptor,
            result,
            explanation,
        }
    }

    let os = MockOsProvider::default();
    let mut entries: Vec<SnapshotEntry> = os
        .list_metrics()
        .into_iter()
        .map(|descriptor| {
            let result = os.acquire(&descriptor.id);
            let explanation = explain(&descriptor, &result);
            SnapshotEntry {
                descriptor,
                result,
                explanation,
            }
        })
        .collect();

    // Runtime not detected: source-not-detected state (row 2).
    let rt = MockRuntimeAdapter::not_detected();
    let rt_desc = MetricDescriptor {
        id: MetricId::new("rt.context.tokens"),
        display_name: String::from("AI runtime context tokens"),
        unit: Unit::Count,
        source_class: SourceClass::AiRuntime,
    };
    let rt_result = rt.runtime_metrics(&rt_desc.id);
    entries.push(entry(rt_desc, rt_result));

    // Runtime detected but stopped: source-not-running state (row 2). The mock
    // adapter exposes only not-detected/ready, so this one entry is constructed
    // directly from the core API at the application boundary.
    let stopped_desc = MetricDescriptor {
        id: MetricId::new("rt.loaded.models"),
        display_name: String::from("AI runtime loaded models"),
        unit: Unit::Count,
        source_class: SourceClass::AiRuntime,
    };
    let stopped_result = MetricResult::source_not_ready(SourceAvailability::NotRunning);
    entries.push(entry(stopped_desc, stopped_result));

    Snapshot {
        at: Timestamp::now(),
        entries,
    }
}

#[cfg(target_os = "linux")]
fn build_snapshot() -> Snapshot {
    use aer_core::{explain, OsMetricsProvider, SnapshotEntry};

    let provider = crate::platform::linux::LinuxMetricsProvider::new();

    let descriptors = provider.list_metrics();
    let entries: Vec<SnapshotEntry> = descriptors
        .into_iter()
        .map(|descriptor| {
            let result = provider.acquire(&descriptor.id);
            let explanation = explain(&descriptor, &result);
            SnapshotEntry {
                descriptor,
                result,
                explanation,
            }
        })
        .collect();

    Snapshot {
        at: aer_core::Timestamp::now(),
        entries,
    }
}

#[cfg(any(target_os = "windows", test))]
fn windows_snapshot_with_provider(provider: &impl aer_core::OsMetricsProvider) -> Snapshot {
    use aer_core::{explain, Outcome, SnapshotEntry};

    let entries = provider
        .list_metrics()
        .into_iter()
        .filter_map(|descriptor| {
            let result = provider.acquire(&descriptor.id);
            if matches!(result.outcome, Outcome::Failed(_)) {
                return None;
            }
            let explanation = explain(&descriptor, &result);
            Some(SnapshotEntry {
                descriptor,
                result,
                explanation,
            })
        })
        .collect();

    Snapshot {
        at: aer_core::Timestamp::now(),
        entries,
    }
}

#[cfg(all(target_os = "windows", not(feature = "dev-fixtures")))]
fn build_snapshot() -> Snapshot {
    let provider = crate::platform::windows::WindowsMetricsProvider::new();
    windows_snapshot_with_provider(&provider)
}

#[cfg(any(
    test,
    all(
        not(target_os = "linux"),
        not(target_os = "windows"),
        not(feature = "dev-fixtures")
    )
))]
fn empty_os_snapshot() -> Snapshot {
    Snapshot {
        at: aer_core::Timestamp::now(),
        entries: Vec::new(),
    }
}

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "windows"),
    not(feature = "dev-fixtures")
))]
fn build_snapshot() -> Snapshot {
    // No operating-system metrics provider is wired on unsupported targets.
    // Keep the snapshot honest and unavailable rather than fabricating data.
    empty_os_snapshot()
}

#[cfg(test)]
mod tests {
    //! Report-preview tests construct domain snapshots directly (no `mock`
    //! feature) and assert the preview exposes only the report-safe
    //! representation: non-allow-listed metrics are absent, unsafe display names
    //! are redacted, and no internal ids or raw errors appear.

    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::{empty_os_snapshot, report_preview_of, windows_snapshot_with_provider};
    use crate::platform::windows::{
        WindowsMemoryProbe, WindowsMemoryStatus, WindowsMetricsProvider,
    };
    use aer_core::{
        explain, MetricDescriptor, MetricId, MetricResult, MetricSample, Provenance, SampleValue,
        Snapshot, SnapshotEntry, SourceClass, Timestamp, Unit,
    };

    fn descriptor(id: &str, name: &str) -> MetricDescriptor {
        MetricDescriptor {
            id: MetricId::new(id),
            display_name: String::from(name),
            unit: Unit::Bytes,
            source_class: SourceClass::OperatingSystem,
        }
    }

    fn ok_sample(value: u64) -> MetricSample {
        MetricSample {
            value: SampleValue::Count(value),
            unit: Unit::Bytes,
            provenance: Provenance::OperatingSystemReported,
            timestamp: Timestamp::from_millis(0),
            limitations: Vec::new(),
        }
    }

    fn entry(descriptor: MetricDescriptor, result: MetricResult) -> SnapshotEntry {
        let explanation = explain(&descriptor, &result);
        SnapshotEntry {
            descriptor,
            result,
            explanation,
        }
    }

    fn snapshot(entries: Vec<SnapshotEntry>) -> Snapshot {
        Snapshot {
            at: Timestamp::from_millis(1),
            entries,
        }
    }

    #[test]
    fn unsupported_platform_snapshot_fallback_is_empty() {
        assert!(empty_os_snapshot().entries.is_empty());
    }

    #[derive(Clone)]
    struct FakeWindowsMemoryProbe {
        result: Result<u64, ()>,
        calls: Arc<AtomicUsize>,
    }

    impl FakeWindowsMemoryProbe {
        fn new(result: Result<u64, ()>) -> (Self, Arc<AtomicUsize>) {
            let calls = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    result,
                    calls: Arc::clone(&calls),
                },
                calls,
            )
        }
    }

    impl WindowsMemoryProbe for FakeWindowsMemoryProbe {
        fn memory_status(&self) -> Result<WindowsMemoryStatus, ()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result
                .map(|available_physical_bytes| WindowsMemoryStatus {
                    available_physical_bytes,
                    total_physical_bytes: available_physical_bytes * 2,
                })
        }
    }

    #[test]
    fn windows_snapshot_contains_available_memory_after_one_successful_probe() {
        let (probe, calls) = FakeWindowsMemoryProbe::new(Ok(8_589_934_592));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let snapshot = windows_snapshot_with_provider(&provider);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(
            snapshot.entries[0].descriptor.id.as_str(),
            "os.ram.available"
        );
        match &snapshot.entries[0].result.outcome {
            aer_core::Outcome::Ok(sample) => {
                assert_eq!(sample.value, SampleValue::Count(8_589_934_592));
            }
            other => panic!("expected available Windows memory, got {other:?}"),
        }
        assert_eq!(snapshot.entries[1].descriptor.id.as_str(), "os.ram.total");
    }

    #[test]
    fn windows_snapshot_excludes_failed_probe_without_fabricating_zero() {
        let (probe, calls) = FakeWindowsMemoryProbe::new(Err(()));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let snapshot = windows_snapshot_with_provider(&provider);

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(snapshot.entries.is_empty());
    }

    #[test]
    fn windows_available_memory_uses_the_existing_report_projection() {
        let (probe, _) = FakeWindowsMemoryProbe::new(Ok(4096));
        let provider = WindowsMetricsProvider::from_probe(probe);
        let snapshot = windows_snapshot_with_provider(&provider);

        let preview = report_preview_of(&snapshot);

        assert!(preview.contains("Available memory"));
        assert!(preview.contains("4.10 KB"));
        assert!(preview.contains("4,096 bytes"));
        assert!(preview.contains("bytes"));
        assert!(!preview.contains("Windows"));
        assert!(!preview.contains("GlobalMemoryStatusEx"));
    }

    #[test]
    fn report_preview_exposes_only_allowlisted_safe_content() {
        let allowed = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::ok(ok_sample(1024)),
        );
        // A non-allow-listed metric whose display name is also a private path:
        // it must be dropped by the allow-list, not merely redacted.
        let private = entry(
            descriptor("internal.debug.path", "/home/alice/secret"),
            MetricResult::ok(ok_sample(1)),
        );
        let preview = report_preview_of(&snapshot(vec![allowed, private]));
        assert!(preview.contains("Memory in use"));
        assert!(preview.contains("Status: Available"));
        assert!(preview.contains("1.02 KB"));
        assert!(preview.contains("1,024 bytes"));
        assert!(!preview.contains("/home/"));
        assert!(!preview.contains("alice"));
        assert!(!preview.contains("internal.debug.path"));
        assert!(!preview.contains("os.ram.used"));
    }

    #[test]
    fn report_preview_does_not_leak_raw_errors() {
        let failed = entry(
            descriptor("os.ram.used", "Memory in use"),
            MetricResult::failed(aer_core::AcquisitionError::Timeout),
        );
        let preview = report_preview_of(&snapshot(vec![failed]));
        // The preview is report-safe: a failed entry shows its status only, not
        // the raw acquisition error message.
        assert!(preview.contains("Status: Failed"));
        assert!(!preview.contains("timed out"));
    }

    #[test]
    fn report_preview_redacts_unsafe_allowlisted_display_name() {
        // Defence-in-depth: even an allow-listed metric whose display name
        // unexpectedly looks unsafe is redacted by the sanitizer.
        let hostile = entry(
            descriptor("os.ram.used", "/home/alice/secret"),
            MetricResult::ok(ok_sample(1)),
        );
        let preview = report_preview_of(&snapshot(vec![hostile]));
        assert!(preview.contains("[redacted]"));
        assert!(!preview.contains("/home/"));
        assert!(!preview.contains("alice"));
    }
}
