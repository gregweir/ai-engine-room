//! Milestone 1C runtime detection — application layer, not `aer-core`.
//!
//! Ollama detection is a thin, local-HTTP probe of the documented
//! `http://127.0.0.1:11434/api/version` endpoint. It flows through the existing,
//! unchanged `AiRuntimeAdapter::detect()` for source availability, and the
//! runtime version Ollama reports flows through a small, runtime-neutral
//! `RuntimeStatus` representation that lives here in the application layer — it
//! is runtime metadata, not a metric, so `aer-core`, `SampleValue`, `MetricSample`,
//! metric `Provenance`, the report boundary, and the `AiRuntimeAdapter` trait
//! are all left untouched. `reqwest` types are confined to this module.
//!
//! Submodules:
//! - [`status`] — `RuntimeType`, `VersionProvenance`, `RuntimeVersion`, `RuntimeStatus`.
//! - [`probe`] — `ProbeOutcome`, the transport-neutral `VersionProbe` trait, and
//!   the pure `interpret_probe` mapping.
//! - [`ollama`] — `OllamaAdapter` (unchanged trait + concrete `runtime_status()`)
//!   and the `ReqwestProbe` production transport.
//! - [`inventory`] (Milestone 1D) — `ModelLocality`, `RuntimeModelSummary`,
//!   `InventoryState`, `ModelInventory`, the transport-neutral `ModelsProbe`
//!   trait, the pure `interpret_inventory` mapping, and the `ReqwestModelsProbe`
//!   production transport. The inventory flows through the concrete
//!   `OllamaAdapter::inventory()` method; `list_models()` stays empty.
//! - [`loaded_models`] (Milestone 1E) — `LoadedModelSummary`,
//!   `LoadedModelState`, `LoadedModelSet`, the transport-neutral `RunningProbe`
//!   trait, the pure `interpret_loaded_models` mapping, and the
//!   `ReqwestRunningProbe` production transport. The loaded-model set flows
//!   through the concrete `OllamaAdapter::loaded_models_status()` method;
//!   `loaded_models()` stays empty. 1D (catalogue) and 1E (currently loaded)
//!   answer distinct questions and are not merged.

pub mod inference;
pub mod inventory;
pub mod llama_cpp;
pub mod lm_studio;
pub mod loaded_models;
pub mod ollama;
pub mod probe;
pub mod status;

#[cfg(test)]
mod tests {
    //! `detect()` is a synchronous `AiRuntimeAdapter` trait method, but the
    //! production transport is async `reqwest` (no `blocking` feature; `tokio`
    //! is not a direct dependency). The sync `detect()` path drives the async
    //! probe through Tauri 2's lazily-initialised async runtime. This confirms
    //! that entry point is usable from a plain sync caller without a running
    //! Tauri application — the precondition for the live Ollama test.

    #[test]
    fn async_runtime_block_on_works_without_running_app() {
        let value = tauri::async_runtime::block_on(async { 42 });
        assert_eq!(value, 42);
    }
}
