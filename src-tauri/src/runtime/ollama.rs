//! The Ollama runtime adapter and its `reqwest` production transport.
//!
//! The adapter is a thin layer over Ollama's documented local `/api/version`
//! endpoint. Detection flows through the existing, unchanged
//! `AiRuntimeAdapter::detect()` for source availability; the runtime version
//! Ollama reports flows through the concrete `runtime_status()` method as
//! application-layer [`RuntimeStatus`] metadata — never through
//! `runtime_metrics()` (which stays numeric-metric-oriented and returns
//! `Unsupported` for every id in 1C). `detect()` and `runtime_status()` share
//! one private probe, so there is exactly one HTTP round trip per status fetch.
//!
//! In Milestone 1D the adapter additionally exposes a read-only model
//! inventory via the concrete `inventory()` method, which probes the
//! documented local `/api/tags` endpoint through a separate transport-neutral
//! `ModelsProbe` (default [`ReqwestModelsProbe`]) and reduces the result with
//! the pure [`interpret_inventory`] mapping to application-layer
//! [`ModelInventory`] metadata. Like the version, the inventory is runtime
//! metadata, not a metric: `list_models()` stays empty for 1D, and
//! `runtime_metrics()` is unchanged.
//!
//! In Milestone 1E the adapter additionally exposes a read-only loaded-model /
//! runtime-residency view via the concrete `loaded_models_status()` method,
//! which probes the documented local `/api/ps` endpoint through a separate
//! transport-neutral `RunningProbe` (default [`ReqwestRunningProbe`]) and
//! reduces the result with the pure [`interpret_loaded_models`] mapping to
//! application-layer [`LoadedModelSet`] metadata. Like the version and the
//! inventory, the loaded-model set is runtime metadata, not a metric:
//! `loaded_models()` stays empty for 1E, and `runtime_metrics()` is unchanged.
//! 1D (catalogue, `/api/tags`) and 1E (currently loaded, `/api/ps`) answer
//! distinct questions and are not merged.
//!
//! In Milestone 1G the adapter additionally exposes an observed-inference path
//! via the concrete `inference_observation(model)` method, which performs one
//! non-streaming `POST /api/generate` with a fixed synthetic diagnostic prompt
//! through a transport-neutral `GenerateProbe` (default [`ReqwestGenerateProbe`])
//! and reduces the result with the pure [`interpret_inference`] mapping to
//! application-layer [`InferenceObservation`] metadata. Like the version,
//! inventory, and loaded-model set, the observation is runtime metadata, not a
//! metric: `runtime_metrics()` is unchanged, and no `AiRuntimeAdapter` trait
//! method is added. The generated `response` text is discarded at the pure
//! boundary; the execution boundary is always `undetermined`. 1G reuses the 1D
//! `inventory()` for a catalogue identity/availability check before any POST.
//!
//! `reqwest` types are confined to [`ReqwestProbe`], [`ReqwestModelsProbe`],
//! [`ReqwestRunningProbe`], and [`ReqwestGenerateProbe`]; the trait surface
//! stays domain-only. The adapter is cross-platform (local HTTP, not cfg-gated,
//! not coupled to the Linux OS-provider layer).

use std::future::Future;
use std::time::Duration;

use aer_core::{
    AiRuntimeAdapter, LimitationCode, MetricId, MetricResult, RuntimeModel, SourceAvailability,
};

use crate::runtime::inventory::{
    interpret_inventory, InventoryState, ModelInventory, ModelsProbe, ReqwestModelsProbe,
};
use crate::runtime::loaded_models::{
    interpret_loaded_models, LoadedModelSet, LoadedModelState, ReqwestRunningProbe, RunningProbe,
};
use crate::runtime::probe::{interpret_probe, ProbeOutcome, VersionProbe};
use crate::runtime::status::{RuntimeStatus, RuntimeType};

// Milestone 1G: the generate probe + pure interpretation live in `inference`.
// Re-exported here so the adapter's constructor defaults read like the 1C/1D/1E
// defaults (`ReqwestProbe`, `ReqwestModelsProbe`, `ReqwestRunningProbe`).
pub use crate::runtime::inference::{
    interpret_inference, GenerateOutcome, GenerateProbe, InferenceObservation, ReqwestGenerateProbe,
};

/// Ollama's documented local default version endpoint, used explicitly as the
/// loopback IP to avoid `localhost` IPv6/IPv4 resolution ambiguity. No
/// configurable endpoint in 1C; no hostname, LAN address, or model name.
pub const OLLAMA_DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434/api/version";

/// Connect deadline for the local probe: 500 ms. The local endpoint should
/// accept the connection immediately or refuse it; this bounds the attempt.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

/// Overall request deadline (connect + read): 2 seconds. A hanging responder
/// cannot stall the UI beyond this.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Ollama adapter. Generic over a [`VersionProbe`] transport (the version
/// path, 1C), a [`ModelsProbe`] transport (the inventory path, 1D), a
/// [`RunningProbe`] transport (the loaded-models path, 1E), and a
/// [`GenerateProbe`] transport (the observed-inference path, 1G) so the
/// production `reqwest` transports and test fixtures share the same pure
/// interpretation. Defaults to [`ReqwestProbe`], [`ReqwestModelsProbe`],
/// [`ReqwestRunningProbe`], and [`ReqwestGenerateProbe`]. The 1G fourth generic
/// cascades through defaults so 1C's `with_probe`, 1D's `with_probes`, and 1E's
/// `with_all_probes` signatures remain valid and their tests stay green.
pub struct OllamaAdapter<
    P: VersionProbe = ReqwestProbe,
    M: ModelsProbe = ReqwestModelsProbe,
    R: RunningProbe = ReqwestRunningProbe,
    G: GenerateProbe = ReqwestGenerateProbe,
> {
    probe: P,
    models_probe: M,
    running_probe: R,
    generate_probe: G,
}

impl OllamaAdapter<ReqwestProbe, ReqwestModelsProbe, ReqwestRunningProbe, ReqwestGenerateProbe> {
    /// Construct an adapter that probes the real local Ollama endpoint with
    /// bounded timeouts (the version probe, the models probe, the loaded-models
    /// probe, and the generate probe).
    pub fn new() -> Self {
        Self {
            probe: ReqwestProbe::new(),
            models_probe: ReqwestModelsProbe::new(),
            running_probe: ReqwestRunningProbe::new(),
            generate_probe: ReqwestGenerateProbe::new(),
        }
    }
}

impl<P: VersionProbe>
    OllamaAdapter<P, ReqwestModelsProbe, ReqwestRunningProbe, ReqwestGenerateProbe>
{
    /// Construct an adapter with an explicit version probe transport (used by
    /// tests with a fixture probe; also the seam for any future transport).
    /// The models, running, and generate probes default to [`ReqwestModelsProbe`],
    /// [`ReqwestRunningProbe`], and [`ReqwestGenerateProbe`], keeping 1C's
    /// `with_probe` signature valid.
    pub fn with_probe(probe: P) -> Self {
        Self {
            probe,
            models_probe: ReqwestModelsProbe::new(),
            running_probe: ReqwestRunningProbe::new(),
            generate_probe: ReqwestGenerateProbe::new(),
        }
    }
}

impl<P: VersionProbe, M: ModelsProbe>
    OllamaAdapter<P, M, ReqwestRunningProbe, ReqwestGenerateProbe>
{
    /// Construct an adapter with explicit version and models probe transports
    /// (the seam for tests with fixture probes for both paths). The running and
    /// generate probes default to [`ReqwestRunningProbe`] and
    /// [`ReqwestGenerateProbe`], keeping 1D's `with_probes` signature valid.
    pub fn with_probes(probe: P, models_probe: M) -> Self {
        Self {
            probe,
            models_probe,
            running_probe: ReqwestRunningProbe::new(),
            generate_probe: ReqwestGenerateProbe::new(),
        }
    }
}

impl<P: VersionProbe, M: ModelsProbe, R: RunningProbe>
    OllamaAdapter<P, M, R, ReqwestGenerateProbe>
{
    /// Construct an adapter with explicit version, models, and running probe
    /// transports (the seam for tests with fixture probes for all three paths).
    /// The generate probe defaults to [`ReqwestGenerateProbe`], keeping 1E's
    /// `with_all_probes` signature valid.
    pub fn with_all_probes(probe: P, models_probe: M, running_probe: R) -> Self {
        Self {
            probe,
            models_probe,
            running_probe,
            generate_probe: ReqwestGenerateProbe::new(),
        }
    }
}

impl<P: VersionProbe, M: ModelsProbe, R: RunningProbe, G: GenerateProbe> OllamaAdapter<P, M, R, G> {
    /// Construct an adapter with explicit version, models, running, and generate
    /// probe transports (the seam for 1G tests with a fixture generate probe and
    /// fixture catalogue probe, so the authorization/identity/concurrency
    /// orchestration is exercised without a socket).
    pub fn with_all_probes_and_generate(
        probe: P,
        models_probe: M,
        running_probe: R,
        generate_probe: G,
    ) -> Self {
        Self {
            probe,
            models_probe,
            running_probe,
            generate_probe,
        }
    }

    /// The runtime status for one probe: source availability plus the optional
    /// verbatim version Ollama reported. One HTTP round trip. Async so the
    /// production `reqwest` transport runs without blocking on the Tauri async
    /// runtime. Uses only the version probe; unchanged by 1D, 1E, and 1G.
    pub async fn runtime_status(&self) -> RuntimeStatus {
        let outcome = self.probe.probe().await;
        interpret_probe(outcome, RuntimeType::Ollama)
    }

    /// The read-only model inventory for one probe: the catalogue Ollama
    /// reports at the documented local `/api/tags` endpoint, reduced to
    /// application-layer [`ModelInventory`] metadata. One HTTP round trip. Async
    /// so the bounded local probe runs without blocking on the Tauri async
    /// runtime. Uses only the models probe; unchanged by 1E and 1G. Read-only —
    /// no loading, generation, pull, or delete. 1G reuses this for the catalogue
    /// identity/availability check before a generate POST.
    pub async fn inventory(&self) -> ModelInventory {
        let outcome = self.models_probe.fetch().await;
        interpret_inventory(outcome, RuntimeType::Ollama)
    }

    /// The read-only loaded-model set for one probe: what Ollama reports as
    /// currently loaded at the documented local `/api/ps` endpoint, reduced to
    /// application-layer [`LoadedModelSet`] metadata. One HTTP round trip. Async
    /// so the bounded local probe runs without blocking on the Tauri async
    /// runtime. Uses only the running probe; independent of the 1C version path,
    /// the 1D inventory path, and the 1G generate path. Read-only — no loading,
    /// unloading, prompting, generation, or keep-alive change.
    pub async fn loaded_models_status(&self) -> LoadedModelSet {
        let outcome = self.running_probe.fetch().await;
        interpret_loaded_models(outcome, RuntimeType::Ollama)
    }

    /// One observed inference for `model`: a single non-streaming
    /// `POST /api/generate` with the fixed diagnostic prompt/profile, reduced to
    /// application-layer [`InferenceObservation`] metadata by the pure
    /// [`interpret_inference`] mapping. One HTTP round trip, no retry. Async so
    /// the bounded local probe runs without blocking on the Tauri async runtime.
    /// Uses only the generate probe; independent of the 1C/1D/1E paths. The
    /// generated `response` text is discarded by the pure layer and never
    /// surfaces here. The execution boundary is always `undetermined`. This is a
    /// concrete adapter method (parallel to `inventory()`/`loaded_models_status()`)
    /// — NOT an `AiRuntimeAdapter` trait method — so the trait stays unchanged.
    pub async fn inference_observation(&self, model: &str) -> InferenceObservation {
        let outcome = self.generate_probe.probe(model).await;
        interpret_inference(outcome, model)
    }
}

impl<P: VersionProbe, M: ModelsProbe, R: RunningProbe, G: GenerateProbe> AiRuntimeAdapter
    for OllamaAdapter<P, M, R, G>
{
    fn detect(&self) -> SourceAvailability {
        // Sync trait method: drive the one async probe through Tauri's
        // lazily-initialised async runtime. This is for a plain synchronous
        // caller and must not be called from inside an async Tauri command; the
        // 1C command path uses `runtime_status()` instead. Returns the same
        // source availability `runtime_status()` would for the same probe.
        let outcome = tauri::async_runtime::block_on(self.probe.probe());
        interpret_probe(outcome, RuntimeType::Ollama).source_availability
    }

    fn list_models(&self) -> Vec<RuntimeModel> {
        // Out of scope for 1C; 1D routes the catalogue through the concrete
        // `inventory()` method instead, so this stays empty.
        Vec::new()
    }

    fn loaded_models(&self) -> Vec<RuntimeModel> {
        // Out of scope for 1C/1D/1E: 1E routes the currently-loaded set through
        // the concrete `loaded_models_status()` method instead, so this stays
        // empty (the core `RuntimeModel { id, display_name }` is too minimal to
        // carry rich loaded-model metadata).
        Vec::new()
    }

    fn runtime_metrics(&self, _id: &MetricId) -> MetricResult {
        // Not used to carry the version (a version is metadata, not a metric).
        // 1C exposes no numeric runtime metrics; any id is unsupported here.
        MetricResult::unsupported(LimitationCode::NotImplementedOnThisPlatform)
    }
}

impl<P: VersionProbe, M: ModelsProbe, R: RunningProbe, G: GenerateProbe> Default
    for OllamaAdapter<P, M, R, G>
where
    P: Default,
    M: Default,
    R: Default,
    G: Default,
{
    fn default() -> Self {
        Self {
            probe: P::default(),
            models_probe: M::default(),
            running_probe: R::default(),
            generate_probe: G::default(),
        }
    }
}

/// Production transport: a `reqwest::Client` with bounded timeouts, used only
/// for the documented local `GET /api/version`. All `reqwest` types stay in
/// this struct and its `VersionProbe` impl.
pub struct ReqwestProbe {
    client: reqwest::Client,
}

impl ReqwestProbe {
    /// Build a client with the bounded 1C timeouts. The client is reused across
    /// probes (connection pool, no per-probe construction cost).
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

impl Default for ReqwestProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionProbe for ReqwestProbe {
    fn probe(&self) -> impl Future<Output = ProbeOutcome> + Send {
        let client = self.client.clone();
        async move {
            match client.get(OLLAMA_DEFAULT_ENDPOINT).send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    match response.text().await {
                        Ok(body) => ProbeOutcome::Responded { status, body },
                        // Body read failed: classify the transport error honestly
                        // (a read timeout is a read timeout) rather than fabricating
                        // a response. No raw error crosses the boundary.
                        Err(error) => classify_reqwest_error(error),
                    }
                }
                Err(error) => classify_reqwest_error(error),
            }
        }
    }
}

/// Reduce a `reqwest` error to a controlled `ProbeOutcome`. No raw error text
/// leaves this function. Timeout is checked first, and only an underlying
/// `std::io::ErrorKind::ConnectionRefused` maps to `ConnectionRefused`
/// (NotDetected). Everything else is `Other` (Unreachable) — conservative, no
/// fabricated certainty.
fn classify_reqwest_error(error: reqwest::Error) -> ProbeOutcome {
    if error.is_timeout() {
        return ProbeOutcome::Timeout;
    }

    if let Some(io_error) = find_source_io_error(&error) {
        if io_error.kind() == std::io::ErrorKind::ConnectionRefused {
            return ProbeOutcome::ConnectionRefused;
        }
    }

    ProbeOutcome::Other
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

/// Controlled, Ollama-named display text for one runtime status. Deterministic
/// and application-layer (not `aer-core`, not Svelte): the frontend renders these
/// pre-computed strings and does not infer detection semantics from raw errors,
/// status codes, or transport kinds. The four intended user-facing states are
/// distinguished by `(source_availability, version.is_some())`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaStatusText {
    pub status_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a runtime status to controlled Ollama-named display text. No health,
/// compatibility, security, or performance claims; no "not installed" claim; no
/// raw errors, URLs, hostnames, or status codes. A refused connection is worded
/// as "not detected at the local endpoint" and explicitly does not claim
/// Ollama is not installed.
pub fn ollama_status_text(status: &RuntimeStatus) -> OllamaStatusText {
    match (&status.source_availability, status.version.is_some()) {
        (SourceAvailability::Ready, true) => OllamaStatusText {
            status_label: "Ollama available".to_string(),
            interpretation: "Ollama is responding at the local endpoint and reported its version."
                .to_string(),
            why_it_matters:
                "Engine Room can read the version Ollama reports for this local runtime."
                    .to_string(),
        },
        (SourceAvailability::Ready, false) => OllamaStatusText {
            status_label: "Ollama responded unexpectedly".to_string(),
            interpretation: "Ollama answered at the local endpoint, but the response did not include usable version information.".to_string(),
            why_it_matters: "The endpoint was reached, so Ollama is present, but its version could not be read from the response.".to_string(),
        },
        (SourceAvailability::NotDetected, _) => OllamaStatusText {
            status_label: "Ollama was not detected at the local endpoint".to_string(),
            interpretation: "Nothing responded at the local Ollama endpoint.".to_string(),
            why_it_matters: "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.".to_string(),
        },
        (SourceAvailability::Unreachable, _) => OllamaStatusText {
            status_label: "Ollama could not be reached".to_string(),
            interpretation: "Ollama could not be reached at the local endpoint.".to_string(),
            why_it_matters: "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.".to_string(),
        },
        // NotRunning is never produced in 1C (an HTTP probe cannot establish
        // installed-but-stopped). Defensive, honest fallback if one ever
        // appears: worded as not-detected, never "not installed".
        (SourceAvailability::NotRunning, _) => OllamaStatusText {
            status_label: "Ollama was not detected at the local endpoint".to_string(),
            interpretation: "Ollama was not detected at the local endpoint.".to_string(),
            why_it_matters: "No service answered the local endpoint.".to_string(),
        },
    }
}

/// Controlled, Ollama-named display text for one model inventory. Deterministic
/// and application-layer (not `aer-core`, not Svelte): the frontend renders these
/// pre-computed strings and does not infer inventory state from raw errors,
/// status codes, or transport kinds. `Available` is split into a populated and
/// an empty case (a valid empty catalogue is not an error). Locality is
/// `Undetermined` for every entry and is not described per model here — no
/// per-entry locality claim, mirroring the §5/§15 decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryText {
    pub state_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a model inventory to controlled Ollama-named display text. No health,
/// compatibility, security, or performance claims; no "not installed" claim; no
/// raw errors, URLs, hostnames, ports, or status codes. Size is described as
/// the size Ollama reports (not exact disk usage); no aggregate total. The
/// `NotDetected`/`Unreachable` wording mirrors 1C's `ollama_status_text`. The
/// populated-Available wording is neutral about locality — it does not claim
/// the listed models are physically local (§5/§15); locality is `Undetermined`
/// for every entry and is not described per model.
pub fn ollama_inventory_text(inventory: &ModelInventory) -> InventoryText {
    match inventory.state {
        InventoryState::Available if !inventory.models.is_empty() => InventoryText {
            state_label: "Ollama model catalogue".to_string(),
            interpretation: "These models are listed by the Ollama runtime.".to_string(),
            why_it_matters:
                "Each entry shows the model name and the size Ollama reports for it.".to_string(),
        },
        InventoryState::Available => InventoryText {
            // A valid empty list is Available with no models — not an error.
            state_label: "No models in the catalogue".to_string(),
            interpretation: "Ollama lists no models right now.".to_string(),
            why_it_matters:
                "This is not an error — the catalogue simply contains no models right now."
                    .to_string(),
        },
        InventoryState::RespondedUnexpectedly => InventoryText {
            state_label: "Ollama responded unexpectedly".to_string(),
            interpretation: "Ollama answered at the local endpoint, but the model list could not be read.".to_string(),
            why_it_matters: "The endpoint was reached, so Ollama is present, but its catalogue response was not usable.".to_string(),
        },
        InventoryState::NotDetected => InventoryText {
            state_label: "Ollama was not detected at the local endpoint".to_string(),
            interpretation: "Nothing responded at the local Ollama endpoint.".to_string(),
            why_it_matters: "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.".to_string(),
        },
        InventoryState::Unreachable => InventoryText {
            state_label: "Ollama could not be reached".to_string(),
            interpretation: "Ollama could not be reached at the local endpoint.".to_string(),
            why_it_matters: "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.".to_string(),
        },
    }
}

/// Controlled, Ollama-named display text for one loaded-model set (Milestone
/// 1E). Deterministic and application-layer (not `aer-core`, not Svelte): the
/// frontend renders these pre-computed strings and does not infer loaded-model
/// state from raw errors, status codes, or transport kinds. `Available` is split
/// into a populated and an empty case (a valid empty loaded-model list is not an
/// error). `size` is described as the loaded size Ollama reports (not exact RAM
/// usage); no aggregate total. No locality claim is made for any loaded entry,
/// and 1E makes no "inventory means loaded" claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedModelsText {
    pub state_label: String,
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map a loaded-model set to controlled Ollama-named display text. No health,
/// compatibility, security, or performance claims; no "not installed" claim; no
/// raw errors, URLs, hostnames, ports, status codes, `digest`, or transport
/// kinds. `size` is described as the loaded size Ollama reports (not exact RAM,
/// disk, or total footprint); no aggregate total. `size_vram` and
/// `context_length` are not interpreted as physical measurements, theoretical
/// maxima, or performance figures. The `NotDetected`/`Unreachable` wording
/// mirrors 1C/1D. The populated-Available wording is neutral about locality and
/// does not claim a loaded entry is in the 1D catalogue.
pub fn ollama_loaded_models_text(set: &LoadedModelSet) -> LoadedModelsText {
    match set.state {
        LoadedModelState::Available if !set.models.is_empty() => LoadedModelsText {
            state_label: "Currently loaded in Ollama".to_string(),
            interpretation: "Ollama reports these models as currently loaded.".to_string(),
            why_it_matters:
                "Each entry shows the model name and the loaded size Ollama reports for it."
                    .to_string(),
        },
        LoadedModelState::Available => LoadedModelsText {
            // A valid empty list is Available with no models — not an error.
            state_label: "No models currently running in Ollama".to_string(),
            interpretation: "Ollama reports no models as loaded right now.".to_string(),
            why_it_matters:
                "This is not an error — it means no models are currently running in Ollama."
                    .to_string(),
        },
        LoadedModelState::RespondedUnexpectedly => LoadedModelsText {
            state_label: "Ollama responded unexpectedly".to_string(),
            interpretation: "Ollama answered at the local endpoint, but the loaded-model list could not be read.".to_string(),
            why_it_matters: "The endpoint was reached, so Ollama is present, but its loaded-model response was not usable.".to_string(),
        },
        LoadedModelState::NotDetected => LoadedModelsText {
            state_label: "Ollama was not detected at the local endpoint".to_string(),
            interpretation: "Nothing responded at the local Ollama endpoint.".to_string(),
            why_it_matters: "No service answered the local endpoint. This only shows that nothing answered there right now — not whether Ollama is running on this machine.".to_string(),
        },
        LoadedModelState::Unreachable => LoadedModelsText {
            state_label: "Ollama could not be reached".to_string(),
            interpretation: "Ollama could not be reached at the local endpoint.".to_string(),
            why_it_matters: "The local endpoint did not respond in time. Ollama may be unreachable or too slow to answer right now.".to_string(),
        },
    }
}

/// Controlled, deterministic text for the resource-context explanation (Milestone
/// 1F). Pure mapping from two derived booleans — whether OS-reported available
/// memory has a value (`ram_has_value`) and whether the Ollama-reported
/// loaded-model set is available (`loaded_is_available`) — to one of four
/// controlled variants. No I/O, no value interpolation, no randomisation.
/// Application-layer (not `aer-core`, not Svelte): the frontend renders these
/// strings verbatim and does not pick the variant or infer a relationship.
///
/// The explanation places OS-reported available memory and Ollama-reported
/// loaded-model sizes side by side (contextual juxtaposition only) and explicitly
/// states they must not be combined to attribute exact RAM or VRAM use. No sum of
/// loaded sizes, no sum of VRAM sizes, no `size - size_vram`, no
/// `available - loaded`, no ratio, no percentage, no "fit", no headroom, and no
/// health/threshold/good/bad/safe claim is ever produced. Variant 1 alone carries
/// the developer-approved qualitative context-length line
/// ("A larger configured context can require more memory."); variants 2–4 do not,
/// because the line would be misleading or disconnected where the full picture is
/// absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceContextText {
    pub interpretation: String,
    pub why_it_matters: String,
}

/// Map the two derived booleans to the controlled resource-context explanation.
/// Four variants; variant 1 includes the approved context-length line. Pure and
/// deterministic: the same inputs always yield the same strings.
pub fn resource_context_text(
    ram_has_value: bool,
    loaded_is_available: bool,
) -> ResourceContextText {
    match (ram_has_value, loaded_is_available) {
        (true, true) => ResourceContextText {
            interpretation: "These values come from different sources. Available memory is the operating system's platform-native observation of memory available to the whole system; operating systems can define it differently. Loaded size and VRAM size are values Ollama reports for each running model. Engine Room does not treat them as an exact account of RAM or VRAM use.".to_string(),
            why_it_matters: "They should not be added to or subtracted from available memory to estimate how much memory a model is using. Available memory already reflects current system state, and the model sizes are runtime-reported values, not independently measured consumption. A larger configured context can require more memory.".to_string(),
        },
        (true, false) => ResourceContextText {
            interpretation: "Available memory is the operating system's platform-native observation of memory available to the whole system; operating systems can define it differently. The running-model information from Ollama is not available right now, so there are no Ollama-reported model sizes to place alongside it.".to_string(),
            why_it_matters: "Available memory reflects the whole system. It is not an account of what any single model is using.".to_string(),
        },
        (false, true) => ResourceContextText {
            interpretation: "The available-memory reading is not available right now. Loaded size and VRAM size are values Ollama reports for each running model. Engine Room does not treat them as an exact account of RAM or VRAM use.".to_string(),
            why_it_matters: "Model sizes are runtime-reported values, not independently measured consumption. Without the available-memory reading they cannot be placed alongside system memory.".to_string(),
        },
        (false, false) => ResourceContextText {
            interpretation: "Neither the available-memory reading nor the running-model information is available right now.".to_string(),
            why_it_matters: "There is nothing to place alongside until at least one of them is available.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    //! Deterministic adapter tests with fixture probes and one ephemeral-loopback
    //! transport check — no live Ollama. The pure mapping is covered in
    //! `probe::tests`; these tests cover the adapter wiring: `detect()`
    //! consistency with `runtime_status()`, `runtime_metrics()` not carrying the
    //! version, and the out-of-scope model listings.

    use std::error::Error;
    use std::fmt;
    use std::future::Future;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;
    use crate::runtime::probe::ProbeOutcome;
    use crate::runtime::status::{RuntimeType, RuntimeVersion, VersionProvenance};
    use aer_core::{AiRuntimeAdapter, MetricId, Outcome, SourceAvailability, Support};

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

    /// A test transport that returns a fixed [`ProbeOutcome`]. No I/O.
    struct FakeProbe {
        outcome: ProbeOutcome,
    }

    impl VersionProbe for FakeProbe {
        fn probe(&self) -> impl Future<Output = ProbeOutcome> + Send {
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    fn adapter(outcome: ProbeOutcome) -> OllamaAdapter<FakeProbe> {
        OllamaAdapter::with_probe(FakeProbe { outcome })
    }

    fn status_of(adapter: &OllamaAdapter<FakeProbe>) -> RuntimeStatus {
        tauri::async_runtime::block_on(adapter.runtime_status())
    }

    #[test]
    fn runtime_client_rejects_redirect_without_retry_or_target_request() {
        let (status, body, request, extra_source_request, target_request) =
            redirected_get(ReqwestProbe::new().client);

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

        let mapped = interpret_probe(
            ProbeOutcome::Responded { status, body },
            RuntimeType::Ollama,
        );
        assert_eq!(mapped.source_availability, SourceAvailability::Ready);
        assert_eq!(mapped.version, None);
    }

    fn all_outcomes() -> Vec<ProbeOutcome> {
        vec![
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":""}"#.to_string(),
            },
            ProbeOutcome::Responded {
                status: 404,
                body: String::new(),
            },
            ProbeOutcome::ConnectionRefused,
            ProbeOutcome::Timeout,
            ProbeOutcome::Other,
        ]
    }

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

    #[test]
    fn detect_agrees_with_runtime_status_source_availability() {
        for outcome in all_outcomes() {
            let a = adapter(outcome.clone());
            let via_detect = a.detect();
            let via_status = status_of(&a).source_availability;
            assert_eq!(
                via_detect, via_status,
                "detect() must agree with runtime_status().source_availability for {outcome:?}"
            );
        }
    }

    #[test]
    fn runtime_status_reports_ready_with_runtime_reported_version_on_valid_200() {
        let a = adapter(ProbeOutcome::Responded {
            status: 200,
            body: r#"{"version":"0.1.32"}"#.to_string(),
        });
        let s = status_of(&a);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        let v = s.version.expect("version present");
        assert_eq!(v.value, "0.1.32");
        assert_eq!(v.provenance, VersionProvenance::RuntimeReported);
    }

    #[test]
    fn runtime_status_is_not_detected_on_connection_refused() {
        let a = adapter(ProbeOutcome::ConnectionRefused);
        let s = status_of(&a);
        assert_eq!(s.source_availability, SourceAvailability::NotDetected);
        assert!(s.version.is_none());
    }

    #[test]
    fn runtime_status_is_unreachable_on_timeout() {
        let a = adapter(ProbeOutcome::Timeout);
        let s = status_of(&a);
        assert_eq!(s.source_availability, SourceAvailability::Unreachable);
        assert!(s.version.is_none());
    }

    #[test]
    fn runtime_metrics_is_unsupported_and_does_not_carry_version() {
        let a = adapter(ProbeOutcome::Responded {
            status: 200,
            body: r#"{"version":"0.1.32"}"#.to_string(),
        });
        let result = a.runtime_metrics(&MetricId::new("rt.context.tokens"));
        // Unsupported capability, not a failure; no value; no version path here.
        assert!(matches!(result.support, Support::Unsupported { .. }));
        assert!(matches!(result.outcome, Outcome::NoValue));
    }

    #[test]
    fn list_models_and_loaded_models_are_empty_in_1c() {
        let a = adapter(ProbeOutcome::ConnectionRefused);
        assert!(a.list_models().is_empty());
        assert!(a.loaded_models().is_empty());
    }

    #[test]
    fn default_endpoint_is_documented_loopback_version_endpoint() {
        assert_eq!(
            OLLAMA_DEFAULT_ENDPOINT,
            "http://127.0.0.1:11434/api/version"
        );
    }

    // --- Controlled status text (`ollama_status_text`) ------------------------

    fn text_for(status: RuntimeStatus) -> OllamaStatusText {
        ollama_status_text(&status)
    }

    #[test]
    fn ready_with_version_is_labeled_available() {
        let s = RuntimeStatus::new(
            RuntimeType::Ollama,
            SourceAvailability::Ready,
            Some(RuntimeVersion {
                value: "0.1.32".to_string(),
                provenance: VersionProvenance::RuntimeReported,
            }),
        );
        let t = text_for(s);
        assert_eq!(t.status_label, "Ollama available");
        assert!(!t.interpretation.is_empty());
        assert!(!t.why_it_matters.is_empty());
    }

    #[test]
    fn ready_without_version_is_labeled_responded_unexpectedly() {
        let s = RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Ready, None);
        let t = text_for(s);
        assert_eq!(t.status_label, "Ollama responded unexpectedly");
    }

    #[test]
    fn not_detected_is_labeled_not_detected_at_local_endpoint() {
        let s = RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotDetected, None);
        let t = text_for(s);
        assert_eq!(
            t.status_label,
            "Ollama was not detected at the local endpoint"
        );
    }

    #[test]
    fn unreachable_is_labeled_could_not_be_reached() {
        let s = RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Unreachable, None);
        let t = text_for(s);
        assert_eq!(t.status_label, "Ollama could not be reached");
    }

    /// Never claim "not installed" (or imply install knowledge) in any state.
    #[test]
    fn no_state_claims_not_installed() {
        let states = [
            RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Ready,
                Some(RuntimeVersion {
                    value: "0.1.32".to_string(),
                    provenance: VersionProvenance::RuntimeReported,
                }),
            ),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Ready, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotDetected, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Unreachable, None),
            // Defensive NotRunning fallback (never produced in 1C).
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotRunning, None),
        ];
        for s in states {
            let t = ollama_status_text(&s);
            for field in [t.status_label, t.interpretation, t.why_it_matters] {
                assert!(
                    !field.to_lowercase().contains("install"),
                    "text must not claim install state: {field:?}"
                );
            }
        }
    }

    /// No raw endpoint URL, host, port, status code, or error kind leaks into
    /// the controlled text. The frontend sees only curated wording.
    #[test]
    fn no_raw_endpoint_or_transport_details_in_text() {
        let states = [
            RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Ready,
                Some(RuntimeVersion {
                    value: "0.1.32".to_string(),
                    provenance: VersionProvenance::RuntimeReported,
                }),
            ),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Ready, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotDetected, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Unreachable, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotRunning, None),
        ];
        for s in states {
            let t = ollama_status_text(&s);
            for field in [t.status_label, t.interpretation, t.why_it_matters] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
                assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
                assert!(!lower.contains("http"), "no scheme: {field:?}");
                assert!(!lower.contains("404"), "no status code: {field:?}");
                assert!(
                    !lower.contains("refus"),
                    "no transport error word: {field:?}"
                );
                assert!(!lower.contains("timeout"), "no transport detail: {field:?}");
            }
        }
    }

    /// No health, compatibility, security, or performance claims.
    #[test]
    fn no_health_compat_security_performance_claims_in_text() {
        let states = [
            RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Ready,
                Some(RuntimeVersion {
                    value: "0.1.32".to_string(),
                    provenance: VersionProvenance::RuntimeReported,
                }),
            ),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Ready, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotDetected, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::Unreachable, None),
            RuntimeStatus::new(RuntimeType::Ollama, SourceAvailability::NotRunning, None),
        ];
        for s in states {
            let t = ollama_status_text(&s);
            for field in [t.status_label, t.interpretation, t.why_it_matters] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("health"), "no health claim: {field:?}");
                assert!(!lower.contains("compat"), "no compat claim: {field:?}");
                assert!(!lower.contains("secure"), "no security claim: {field:?}");
                assert!(!lower.contains("fast"), "no performance claim: {field:?}");
                assert!(
                    !lower.contains("perform"),
                    "no performance claim: {field:?}"
                );
            }
        }
    }

    /// Four user-facing states map to four distinct status labels.
    #[test]
    fn four_user_facing_states_have_distinct_labels() {
        let labels = [
            text_for(RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Ready,
                Some(RuntimeVersion {
                    value: "0.1.32".to_string(),
                    provenance: VersionProvenance::RuntimeReported,
                }),
            ))
            .status_label,
            text_for(RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Ready,
                None,
            ))
            .status_label,
            text_for(RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::NotDetected,
                None,
            ))
            .status_label,
            text_for(RuntimeStatus::new(
                RuntimeType::Ollama,
                SourceAvailability::Unreachable,
                None,
            ))
            .status_label,
        ];
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "labels {i} and {j} collide");
            }
        }
    }

    // --- Model inventory (Milestone 1D) ---------------------------------------
    //
    // The pure mapping is covered in `inventory::tests`; these tests cover the
    // adapter wiring: `inventory()` reduces a fixture probe through the pure
    // `interpret_inventory`, and the 1C version path stays unchanged when the
    // models probe is wired (1C tests above still pass).

    use crate::runtime::inventory::{
        ModelLocality, ModelsOutcome, RuntimeModelSummary, OLLAMA_TAGS_ENDPOINT,
    };

    /// A test models transport that returns a fixed [`ModelsOutcome`]. No I/O.
    struct FakeModelsProbe {
        outcome: ModelsOutcome,
    }

    impl ModelsProbe for FakeModelsProbe {
        fn fetch(&self) -> impl Future<Output = ModelsOutcome> + Send {
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    fn adapter_with_models(
        probe_outcome: ProbeOutcome,
        models_outcome: ModelsOutcome,
    ) -> OllamaAdapter<FakeProbe, FakeModelsProbe> {
        OllamaAdapter::with_probes(
            FakeProbe {
                outcome: probe_outcome,
            },
            FakeModelsProbe {
                outcome: models_outcome,
            },
        )
    }

    fn inventory_of(adapter: &OllamaAdapter<FakeProbe, FakeModelsProbe>) -> ModelInventory {
        tauri::async_runtime::block_on(adapter.inventory())
    }

    #[test]
    fn inventory_maps_a_valid_tags_response_through_the_adapter() {
        let body = r#"{"models": [{"name": "example:1b", "size": 1024, "details": {"parameter_size": "1.0B", "quantization_level": "Q4_0", "family": "llama"}}]}"#;
        let a = adapter_with_models(
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ModelsOutcome::Responded {
                status: 200,
                body: body.to_string(),
            },
        );
        let inv = inventory_of(&a);
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(inv.source_availability, SourceAvailability::Ready);
        assert_eq!(inv.models.len(), 1);
        let m = &inv.models[0];
        assert_eq!(m.display_name, "example:1b");
        assert_eq!(m.locality, ModelLocality::Undetermined);
        assert_eq!(m.size_bytes, Some(1024));
        assert_eq!(m.parameter_size.as_deref(), Some("1.0B"));
        assert_eq!(m.quantization.as_deref(), Some("Q4_0"));
        assert_eq!(m.families, vec!["llama".to_string()]);
    }

    #[test]
    fn inventory_reports_not_detected_when_connection_refused() {
        let a = adapter_with_models(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::ConnectionRefused,
        );
        let inv = inventory_of(&a);
        assert_eq!(inv.state, InventoryState::NotDetected);
        assert_eq!(inv.source_availability, SourceAvailability::NotDetected);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn inventory_reports_unreachable_on_timeout() {
        let a = adapter_with_models(ProbeOutcome::Timeout, ModelsOutcome::Timeout);
        let inv = inventory_of(&a);
        assert_eq!(inv.state, InventoryState::Unreachable);
        assert_eq!(inv.source_availability, SourceAvailability::Unreachable);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn inventory_path_does_not_change_version_path_when_models_probe_wired() {
        // The 1C version path must remain correct when the 1D models probe is
        // present: runtime_status() uses only the version probe.
        let a = adapter_with_models(
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ModelsOutcome::ConnectionRefused,
        );
        let s = tauri::async_runtime::block_on(a.runtime_status());
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        let v = s.version.expect("version present");
        assert_eq!(v.value, "0.1.32");
        // Meanwhile the inventory reflects the (refused) models probe, not the
        // version probe — the two paths are independent.
        let inv = inventory_of(&a);
        assert_eq!(inv.state, InventoryState::NotDetected);
    }

    #[test]
    fn inventory_and_list_models_are_consistent_for_a_fixture_probe() {
        // list_models()/loaded_models() stay empty (unchanged); the inventory
        // is the authoritative 1D path.
        let a = adapter_with_models(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::Responded {
                status: 200,
                body: r#"{"models": []}"#.to_string(),
            },
        );
        assert!(a.list_models().is_empty());
        assert!(a.loaded_models().is_empty());
        let inv = inventory_of(&a);
        assert_eq!(inv.state, InventoryState::Available);
        assert!(inv.models.is_empty());
    }

    #[test]
    fn inventory_runtime_metrics_still_unsupported() {
        let a = adapter_with_models(
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ModelsOutcome::Responded {
                status: 200,
                body: r#"{"models": []}"#.to_string(),
            },
        );
        let result = a.runtime_metrics(&MetricId::new("rt.context.tokens"));
        assert!(matches!(result.support, Support::Unsupported { .. }));
        assert!(matches!(result.outcome, Outcome::NoValue));
    }

    #[test]
    fn runtime_model_summary_carries_only_controlled_fields() {
        // The summary type exposes no digest/endpoint/host/remote_host field;
        // a digest present in the body must not surface anywhere.
        let body = r#"{"models": [{"name": "x:1b", "size": 1, "digest": "deadbeef", "remote_host": "example.invalid"}]}"#;
        let a = adapter_with_models(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::Responded {
                status: 200,
                body: body.to_string(),
            },
        );
        let inv = inventory_of(&a);
        assert_eq!(inv.models.len(), 1);
        let m = &inv.models[0];
        // No field on RuntimeModelSummary can carry the digest or remote_host.
        let _: &RuntimeModelSummary = m;
        assert_eq!(m.display_name, "x:1b");
        assert_eq!(m.size_bytes, Some(1));
    }

    #[test]
    fn tags_endpoint_constant_is_documented_loopback() {
        assert_eq!(OLLAMA_TAGS_ENDPOINT, "http://127.0.0.1:11434/api/tags");
    }

    // --- Controlled inventory text (`ollama_inventory_text`) ------------------

    fn inventory_with(state: InventoryState, models_len: usize) -> ModelInventory {
        ModelInventory {
            runtime_type: RuntimeType::Ollama,
            state,
            source_availability: match state {
                InventoryState::Available | InventoryState::RespondedUnexpectedly => {
                    SourceAvailability::Ready
                }
                InventoryState::NotDetected => SourceAvailability::NotDetected,
                InventoryState::Unreachable => SourceAvailability::Unreachable,
            },
            models: (0..models_len)
                .map(|i| RuntimeModelSummary {
                    display_name: format!("example:{i}b"),
                    locality: ModelLocality::Undetermined,
                    size_bytes: Some(1024),
                    parameter_size: None,
                    quantization: None,
                    families: Vec::new(),
                })
                .collect(),
            skipped: 0,
        }
    }

    #[test]
    fn inventory_text_available_with_models_is_ollama_model_catalogue() {
        let t = ollama_inventory_text(&inventory_with(InventoryState::Available, 2));
        assert_eq!(t.state_label, "Ollama model catalogue");
        assert_eq!(
            t.interpretation,
            "These models are listed by the Ollama runtime."
        );
        assert!(!t.why_it_matters.is_empty());
    }

    #[test]
    fn inventory_text_available_with_models_does_not_claim_locality() {
        // The populated-Available wording is neutral about locality: it must not
        // claim the listed models are physically local. Locality is `Undetermined`
        // for every entry (§5/§15) and is not described per model. The forbidden
        // synonyms here are the catalogue-locality claims (not the "not installed"
        // claim, which a different test guards).
        let t = ollama_inventory_text(&inventory_with(InventoryState::Available, 1));
        for field in [
            t.state_label.as_str(),
            t.interpretation.as_str(),
            t.why_it_matters.as_str(),
        ] {
            let lower = field.to_lowercase();
            assert!(!lower.contains("local"), "no locality claim: {field:?}");
            assert!(!lower.contains("locally"), "no locality claim: {field:?}");
            assert!(
                !lower.contains("downloaded"),
                "no locality claim: {field:?}"
            );
            assert!(
                !lower.contains("on this computer"),
                "no locality claim: {field:?}"
            );
            assert!(
                !lower.contains("stored here"),
                "no locality claim: {field:?}"
            );
        }
    }

    #[test]
    fn inventory_text_available_empty_is_not_an_error() {
        let t = ollama_inventory_text(&inventory_with(InventoryState::Available, 0));
        assert_eq!(t.state_label, "No models in the catalogue");
        assert_eq!(t.interpretation, "Ollama lists no models right now.");
        assert_eq!(
            t.why_it_matters,
            "This is not an error — the catalogue simply contains no models right now."
        );

        for field in [t.state_label, t.interpretation, t.why_it_matters] {
            let lower = field.to_lowercase();
            for forbidden in [
                "local",
                "locally",
                "installed",
                "downloaded",
                "stored here",
                "on this computer",
            ] {
                assert!(
                    !lower.contains(forbidden),
                    "empty-state text must not claim locality: {field:?}"
                );
            }
        }
    }

    #[test]
    fn inventory_text_responded_unexpectedly_distinguishes_from_not_detected() {
        let t = ollama_inventory_text(&inventory_with(InventoryState::RespondedUnexpectedly, 0));
        assert_eq!(t.state_label, "Ollama responded unexpectedly");
        assert!(t.why_it_matters.contains("Ollama is present"));
    }

    #[test]
    fn inventory_text_not_detected_mirrors_1c_wording() {
        let t = ollama_inventory_text(&inventory_with(InventoryState::NotDetected, 0));
        assert_eq!(
            t.state_label,
            "Ollama was not detected at the local endpoint"
        );
    }

    #[test]
    fn inventory_text_unreachable_mirrors_1c_wording() {
        let t = ollama_inventory_text(&inventory_with(InventoryState::Unreachable, 0));
        assert_eq!(t.state_label, "Ollama could not be reached");
    }

    #[test]
    fn inventory_text_has_distinct_state_labels_per_state() {
        let labels = [
            ollama_inventory_text(&inventory_with(InventoryState::Available, 1)).state_label,
            ollama_inventory_text(&inventory_with(InventoryState::Available, 0)).state_label,
            ollama_inventory_text(&inventory_with(InventoryState::RespondedUnexpectedly, 0))
                .state_label,
            ollama_inventory_text(&inventory_with(InventoryState::NotDetected, 0)).state_label,
            ollama_inventory_text(&inventory_with(InventoryState::Unreachable, 0)).state_label,
        ];
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "labels {i} and {j} collide");
            }
        }
    }

    #[test]
    fn inventory_text_never_claims_install_state() {
        let states = [
            inventory_with(InventoryState::Available, 1),
            inventory_with(InventoryState::Available, 0),
            inventory_with(InventoryState::RespondedUnexpectedly, 0),
            inventory_with(InventoryState::NotDetected, 0),
            inventory_with(InventoryState::Unreachable, 0),
        ];
        for s in states {
            let t = ollama_inventory_text(&s);
            for field in [t.state_label, t.interpretation, t.why_it_matters] {
                let lower = field.to_lowercase();
                assert!(
                    !lower.contains("not installed"),
                    "must not claim the runtime is not installed: {field:?}"
                );
            }
        }
    }

    #[test]
    fn inventory_text_never_leaks_raw_transport_or_endpoint_details() {
        let states = [
            inventory_with(InventoryState::Available, 1),
            inventory_with(InventoryState::Available, 0),
            inventory_with(InventoryState::RespondedUnexpectedly, 0),
            inventory_with(InventoryState::NotDetected, 0),
            inventory_with(InventoryState::Unreachable, 0),
        ];
        for s in states {
            let t = ollama_inventory_text(&s);
            for field in [t.state_label, t.interpretation, t.why_it_matters] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("127.0.0.1"), "no endpoint IP: {field:?}");
                assert!(!lower.contains("11434"), "no endpoint port: {field:?}");
                assert!(!lower.contains("http"), "no scheme: {field:?}");
                assert!(!lower.contains("refus"), "no transport word: {field:?}");
                assert!(!lower.contains("digest"), "no digest: {field:?}");
                assert!(!lower.contains("remote_host"), "no remote_host: {field:?}");
            }
        }
    }

    #[test]
    fn inventory_text_makes_no_health_compat_security_performance_claims() {
        let states = [
            inventory_with(InventoryState::Available, 1),
            inventory_with(InventoryState::Available, 0),
            inventory_with(InventoryState::RespondedUnexpectedly, 0),
            inventory_with(InventoryState::NotDetected, 0),
            inventory_with(InventoryState::Unreachable, 0),
        ];
        for s in states {
            let t = ollama_inventory_text(&s);
            for field in [t.state_label, t.interpretation, t.why_it_matters] {
                let lower = field.to_lowercase();
                assert!(!lower.contains("health"), "no health claim: {field:?}");
                assert!(!lower.contains("compat"), "no compat claim: {field:?}");
                assert!(!lower.contains("secure"), "no security claim: {field:?}");
                assert!(!lower.contains("fast"), "no performance claim: {field:?}");
                assert!(
                    !lower.contains("perform"),
                    "no performance claim: {field:?}"
                );
            }
        }
    }

    // --- Loaded models (Milestone 1E) -----------------------------------------
    //
    // The pure mapping is covered in `loaded_models::tests`; these tests cover
    // the adapter wiring: `loaded_models_status()` reduces a fixture probe
    // through the pure `interpret_loaded_models`, the 1C/1D paths stay
    // unchanged when the running probe is wired, and the trait methods stay
    // empty/unsupported.

    use crate::runtime::loaded_models::{
        interpret_loaded_models, LoadedModelSet, LoadedModelState, RunningOutcome,
    };

    /// A test running transport that returns a fixed [`RunningOutcome`]. No I/O.
    struct FakeRunningProbe {
        outcome: RunningOutcome,
    }

    impl RunningProbe for FakeRunningProbe {
        fn fetch(&self) -> impl Future<Output = RunningOutcome> + Send {
            let outcome = self.outcome.clone();
            async move { outcome }
        }
    }

    fn adapter_with_running(
        probe_outcome: ProbeOutcome,
        models_outcome: ModelsOutcome,
        running_outcome: RunningOutcome,
    ) -> OllamaAdapter<FakeProbe, FakeModelsProbe, FakeRunningProbe> {
        OllamaAdapter::with_all_probes(
            FakeProbe {
                outcome: probe_outcome,
            },
            FakeModelsProbe {
                outcome: models_outcome,
            },
            FakeRunningProbe {
                outcome: running_outcome,
            },
        )
    }

    fn loaded_of(
        adapter: &OllamaAdapter<FakeProbe, FakeModelsProbe, FakeRunningProbe>,
    ) -> LoadedModelSet {
        tauri::async_runtime::block_on(adapter.loaded_models_status())
    }

    #[test]
    fn loaded_models_status_maps_a_valid_ps_response_through_the_adapter() {
        let body = r#"{"models": [{"name": "example-chat:8b", "size": 4000000000, "size_vram": 2000000000, "context_length": 8192}]}"#;
        let a = adapter_with_running(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::ConnectionRefused,
            RunningOutcome::Responded {
                status: 200,
                body: body.to_string(),
            },
        );
        let set = loaded_of(&a);
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert_eq!(set.models.len(), 1);
        let m = &set.models[0];
        assert_eq!(m.display_name, "example-chat:8b");
        assert_eq!(m.reported_size_bytes, Some(4_000_000_000));
        assert_eq!(m.reported_vram_bytes, Some(2_000_000_000));
        assert_eq!(m.reported_context_length, Some(8192));
    }

    #[test]
    fn loaded_models_status_reports_not_detected_when_connection_refused() {
        let a = adapter_with_running(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::ConnectionRefused,
            RunningOutcome::ConnectionRefused,
        );
        let set = loaded_of(&a);
        assert_eq!(set.state, LoadedModelState::NotDetected);
        assert_eq!(set.source_availability, SourceAvailability::NotDetected);
        assert!(set.models.is_empty());
    }

    #[test]
    fn loaded_models_status_reports_unreachable_on_timeout() {
        let a = adapter_with_running(
            ProbeOutcome::Timeout,
            ModelsOutcome::Timeout,
            RunningOutcome::Timeout,
        );
        let set = loaded_of(&a);
        assert_eq!(set.state, LoadedModelState::Unreachable);
        assert_eq!(set.source_availability, SourceAvailability::Unreachable);
        assert!(set.models.is_empty());
    }

    #[test]
    fn loaded_models_status_treats_valid_empty_as_available() {
        let a = adapter_with_running(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::ConnectionRefused,
            RunningOutcome::Responded {
                status: 200,
                body: r#"{"models": []}"#.to_string(),
            },
        );
        let set = loaded_of(&a);
        assert_eq!(set.state, LoadedModelState::Available);
        assert_eq!(set.source_availability, SourceAvailability::Ready);
        assert!(set.models.is_empty());
    }

    #[test]
    fn loaded_models_path_does_not_change_version_or_inventory_paths() {
        // The 1C version path and 1D inventory path must remain correct when the
        // 1E running probe is present: each path uses only its own probe.
        let a = adapter_with_running(
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ModelsOutcome::Responded {
                status: 200,
                body: r#"{"models": [{"name": "catalogue:1b", "size": 1}]}"#.to_string(),
            },
            RunningOutcome::ConnectionRefused,
        );
        let s = tauri::async_runtime::block_on(a.runtime_status());
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert_eq!(s.version.expect("version present").value, "0.1.32");

        // The inventory path uses only the models probe and is independent of
        // the (refused) running probe wired into the same adapter.
        let inv = tauri::async_runtime::block_on(a.inventory());
        assert_eq!(inv.state, InventoryState::Available);
        assert_eq!(inv.models.len(), 1);

        // Meanwhile the loaded-model set reflects the (refused) running probe,
        // not the version or models probe — the three paths are independent.
        let set = loaded_of(&a);
        assert_eq!(set.state, LoadedModelState::NotDetected);
    }

    #[test]
    fn loaded_models_and_list_models_remain_empty_runtime_metrics_unsupported() {
        let a = adapter_with_running(
            ProbeOutcome::Responded {
                status: 200,
                body: r#"{"version":"0.1.32"}"#.to_string(),
            },
            ModelsOutcome::Responded {
                status: 200,
                body: r#"{"models": []}"#.to_string(),
            },
            RunningOutcome::Responded {
                status: 200,
                body: r#"{"models": []}"#.to_string(),
            },
        );
        assert!(a.list_models().is_empty());
        assert!(a.loaded_models().is_empty());
        let result = a.runtime_metrics(&MetricId::new("rt.context.tokens"));
        assert!(matches!(result.support, Support::Unsupported { .. }));
        assert!(matches!(result.outcome, Outcome::NoValue));
    }

    #[test]
    fn loaded_models_status_is_consistent_with_pure_interpret_for_a_fixture_probe() {
        // The adapter method is a thin wrapper: it must produce exactly what the
        // pure mapping produces for the same outcome.
        let outcome = RunningOutcome::Responded {
            status: 200,
            body: r#"{"models": [{"name": "x:1b", "size": 1}]}"#.to_string(),
        };
        let a = OllamaAdapter::with_all_probes(
            FakeProbe {
                outcome: ProbeOutcome::ConnectionRefused,
            },
            FakeModelsProbe {
                outcome: ModelsOutcome::ConnectionRefused,
            },
            FakeRunningProbe {
                outcome: outcome.clone(),
            },
        );
        let via_adapter = loaded_of(&a);
        let via_pure = interpret_loaded_models(outcome, RuntimeType::Ollama);
        assert_eq!(via_adapter, via_pure);
    }

    #[test]
    fn loaded_model_set_summary_carries_no_digest_or_endpoint_field() {
        // The loaded-model summary exposes no digest/endpoint/host/details
        // field; a digest and details present in the body must not surface.
        let body = r#"{"models": [{"name": "x:1b", "size": 1, "digest": "deadbeef", "details": {"parameter_size": "1.0B"}, "remote_host": "example.invalid"}]}"#;
        let a = adapter_with_running(
            ProbeOutcome::ConnectionRefused,
            ModelsOutcome::ConnectionRefused,
            RunningOutcome::Responded {
                status: 200,
                body: body.to_string(),
            },
        );
        let set = loaded_of(&a);
        assert_eq!(set.models.len(), 1);
        let m = &set.models[0];
        let _: &crate::runtime::loaded_models::LoadedModelSummary = m;
        assert_eq!(m.display_name, "x:1b");
        assert_eq!(m.reported_size_bytes, Some(1));
    }

    // --- Resource context text (`resource_context_text`, Milestone 1F) --------
    //
    // Pure mapping from `(ram_has_value, loaded_is_available)` to the four
    // controlled variants. The variant-1 strings (including the developer-approved
    // context-length line) are asserted verbatim, the line is qualitative only,
    // and no variant makes a forbidden arithmetic/health/fit claim or leaks raw
    // transport detail. Whole-word forbidden checks avoid false positives on
    // legitimate words like "Loaded".

    /// Tokenise text into lowercase alphanumeric words so forbidden claims are
    /// checked as whole words, not accidental substrings (e.g. "Loaded" must not
    /// be flagged as the verb "load").
    fn resource_words(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Forbidden whole-word claims (fit/headroom/health/low/safe/good/bad/...).
    /// "load"/"unload" are action verbs — "Loaded" tokenises to "loaded", not
    /// "load", so the size label is not flagged.
    const RESOURCE_FORBIDDEN_WORDS: &[&str] = &[
        "fit",
        "headroom",
        "health",
        "healthy",
        "low",
        "dangerous",
        "sufficient",
        "insufficient",
        "good",
        "bad",
        "safe",
        "performance",
        "fast",
        "slow",
        "total",
        "recommend",
        "unload",
        "load",
    ];

    /// Forbidden phrases/symbols checked as substrings of the lowercased text.
    const RESOURCE_FORBIDDEN_SUBSTRINGS: &[&str] = &["%", "used by ollama", "percent of ram"];

    fn assert_no_forbidden_resource_claim(text: &str) {
        let lower = text.to_lowercase();
        let tokens = resource_words(text);
        for forbidden in RESOURCE_FORBIDDEN_WORDS {
            assert!(
                !tokens.iter().any(|t| t.as_str() == *forbidden),
                "forbidden word claim {:?} in: {text:?}",
                forbidden
            );
        }
        for forbidden in RESOURCE_FORBIDDEN_SUBSTRINGS {
            assert!(
                !lower.contains(forbidden),
                "forbidden substring {:?} in: {text:?}",
                forbidden
            );
        }
    }

    fn assert_no_resource_raw_leakage(text: &str) {
        let lower = text.to_lowercase();
        for leak in ["digest", "127.0.0.1", "11434", "http", "endpoint", "host"] {
            assert!(!lower.contains(leak), "raw leakage {leak:?} in: {text:?}");
        }
    }

    #[test]
    fn variant_1_both_available_has_exact_strings_with_context_length_line() {
        let t = resource_context_text(true, true);
        assert_eq!(
            t.interpretation,
            "These values come from different sources. Available memory is the operating system's platform-native observation of memory available to the whole system; operating systems can define it differently. Loaded size and VRAM size are values Ollama reports for each running model. Engine Room does not treat them as an exact account of RAM or VRAM use."
        );
        assert_eq!(
            t.why_it_matters,
            "They should not be added to or subtracted from available memory to estimate how much memory a model is using. Available memory already reflects current system state, and the model sizes are runtime-reported values, not independently measured consumption. A larger configured context can require more memory."
        );
    }

    #[test]
    fn variant_2_ram_available_loaded_unavailable_has_exact_strings_without_context_line() {
        let t = resource_context_text(true, false);
        assert_eq!(
            t.interpretation,
            "Available memory is the operating system's platform-native observation of memory available to the whole system; operating systems can define it differently. The running-model information from Ollama is not available right now, so there are no Ollama-reported model sizes to place alongside it."
        );
        assert_eq!(
            t.why_it_matters,
            "Available memory reflects the whole system. It is not an account of what any single model is using."
        );
        assert!(!t.interpretation.contains("context"));
        assert!(!t.why_it_matters.contains("context"));
    }

    #[test]
    fn variant_3_ram_unavailable_loaded_available_has_exact_strings_without_context_line() {
        let t = resource_context_text(false, true);
        assert_eq!(
            t.interpretation,
            "The available-memory reading is not available right now. Loaded size and VRAM size are values Ollama reports for each running model. Engine Room does not treat them as an exact account of RAM or VRAM use."
        );
        assert_eq!(
            t.why_it_matters,
            "Model sizes are runtime-reported values, not independently measured consumption. Without the available-memory reading they cannot be placed alongside system memory."
        );
        assert!(!t.interpretation.contains("context"));
        assert!(!t.why_it_matters.contains("context"));
    }

    #[test]
    fn variant_4_both_unavailable_has_exact_strings_without_context_line() {
        let t = resource_context_text(false, false);
        assert_eq!(
            t.interpretation,
            "Neither the available-memory reading nor the running-model information is available right now."
        );
        assert_eq!(
            t.why_it_matters,
            "There is nothing to place alongside until at least one of them is available."
        );
        assert!(!t.interpretation.contains("context"));
        assert!(!t.why_it_matters.contains("context"));
    }

    #[test]
    fn context_length_line_is_qualitative_only_in_variant_1() {
        let t = resource_context_text(true, true);
        let line = "A larger configured context can require more memory.";
        assert!(
            t.why_it_matters.contains(line),
            "approved context-length line present in variant 1"
        );
        // Qualitative only: no per-token/per-memory figure, no "better", no
        // performance claim, no numeric conversion, no future estimate.
        let lower = t.why_it_matters.to_lowercase();
        assert!(!lower.contains("per token"));
        assert!(!lower.contains("per-token"));
        assert!(!lower.contains("better"));
        assert!(!lower.contains("performance"));
        assert!(!lower.contains("faster"));
        assert!(!lower.contains("slower"));
        assert!(
            !line.chars().any(|c| c.is_ascii_digit()),
            "context-length line is qualitative (no digits): {line:?}"
        );
    }

    #[test]
    fn context_length_line_appears_in_variant_1_only() {
        let line = "A larger configured context can require more memory.";
        assert!(resource_context_text(true, true)
            .why_it_matters
            .contains(line));
        assert!(!resource_context_text(true, false)
            .why_it_matters
            .contains(line));
        assert!(!resource_context_text(false, true)
            .why_it_matters
            .contains(line));
        assert!(!resource_context_text(false, false)
            .why_it_matters
            .contains(line));
    }

    #[test]
    fn no_variant_makes_a_forbidden_arithmetic_or_health_claim() {
        for (ram, loaded) in [(true, true), (true, false), (false, true), (false, false)] {
            let t = resource_context_text(ram, loaded);
            assert_no_forbidden_resource_claim(&t.interpretation);
            assert_no_forbidden_resource_claim(&t.why_it_matters);
        }
    }

    #[test]
    fn no_variant_leaks_raw_transport_or_endpoint_detail() {
        for (ram, loaded) in [(true, true), (true, false), (false, true), (false, false)] {
            let t = resource_context_text(ram, loaded);
            assert_no_resource_raw_leakage(&t.interpretation);
            assert_no_resource_raw_leakage(&t.why_it_matters);
        }
    }

    #[test]
    fn variant_1_distinguishes_os_reported_from_ollama_reported_sources() {
        let t = resource_context_text(true, true);
        let combined = format!("{} {}", t.interpretation, t.why_it_matters).to_lowercase();
        assert!(
            combined.contains("operating system"),
            "operating-system source named"
        );
        assert!(combined.contains("ollama"), "runtime source named (Ollama)");
    }

    #[test]
    fn variant_1_states_values_must_not_be_combined_to_attribute_exact_use() {
        let t = resource_context_text(true, true);
        let combined = format!("{} {}", t.interpretation, t.why_it_matters).to_lowercase();
        assert!(combined.contains("different sources"));
        assert!(combined.contains("not be added to or subtracted from"));
        assert!(combined.contains("an exact account"));
    }

    #[test]
    fn four_variants_have_distinct_interpretations() {
        let all = [
            resource_context_text(true, true).interpretation,
            resource_context_text(true, false).interpretation,
            resource_context_text(false, true).interpretation,
            resource_context_text(false, false).interpretation,
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "interpretations {i} and {j} collide");
            }
        }
    }
}
