<script lang="ts">
  // Observed-inference surface (Milestone 1G). Lets the user explicitly select a
  // model from the 1D catalogue and run ONE observed inference: a fixed
  // synthetic diagnostic prompt sent to that model. The run is NEVER automatic —
  // not on mount, not on Refresh, not on selection, not on a timer. It happens
  // only after the user selects a model, clicks "Run observation", and then
  // authorizes that one run in a disclosure dialog.
  //
  // Per-run authorization: the disclosure explains Engine Room cannot determine
  // whether the model will run on this computer or through Ollama Cloud, and the
  // user must authorize one run. There is NO "Don't ask again" — each run needs
  // its own authorization. The acknowledgement is request-scoped; nothing is
  // persisted.
  //
  // The component renders only controlled text the backend authors (mirrored
  // verbatim from the Rust `ollama_inference_text` mapping via the view DTO) and
  // the Ollama-reported counts/durations (as seconds) plus the
  // Engine-Room-calculated rate. It never displays the generated response, never
  // echoes the fixed prompt, never claims the model ran locally, and never
  // claims a benchmark or quality score. `execution_boundary` is always
  // `"undetermined"`. No raw errors, endpoints, host/ports, or status codes are
  // shown.
  import type { DataSource } from "../datasource";
  import type {
    InferenceObservationView,
    ModelInventoryView,
    LmStudioSnapshotView,
    RuntimeProvider,
  } from "../types";
  import {
    DISCLOSURE_BODY,
    DISCLOSURE_PRIMARY_ACTION,
    DISCLOSURE_SECONDARY_ACTION,
    DISCLOSURE_TITLE,
    GENERATION_RATE_EXPLANATION,
    GENERATION_RATE_LABEL,
  } from "../fixtures/states";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let {
    inventory,
    lmStudio = null,
    dataSource,
    isActive = true,
    onObservation = () => {},
    onCompletedObservation = () => {},
  }: {
    inventory: ModelInventoryView;
    lmStudio?: LmStudioSnapshotView | null;
    dataSource: DataSource;
    isActive?: boolean;
    onObservation?: (observation: InferenceObservationView) => void;
    onCompletedObservation?: (observation: InferenceObservationView) => void;
  } = $props();

  // The model selector is populated only from a populated available catalogue.
  // No auto-select, no auto-discover: the user explicitly picks a model.
  let selectableModels = $derived([
    ...(inventory.state === "available"
      ? inventory.models.map((model) => ({
          provider: "ollama" as RuntimeProvider,
          model_id: model.display_name,
          display_name: model.display_name,
          loaded: null as boolean | null,
        }))
      : []),
    ...(lmStudio?.state === "available"
      ? lmStudio.models
          .filter((model) => model.inference_eligible)
          .map((model) => ({
            provider: "lm_studio" as RuntimeProvider,
            model_id: model.model_id,
            display_name: model.display_name,
            loaded: model.loaded_instances.length > 0,
          }))
      : []),
  ]);

  let selectedModel = $state<string>("");
  let selectedProvider = $derived(
    (selectableModels.find(
      (model) => `${model.provider}:${model.model_id}` === selectedModel,
    )?.provider ?? "ollama") as RuntimeProvider,
  );
  let selectedModelId = $derived(
    selectableModels.find(
      (model) => `${model.provider}:${model.model_id}` === selectedModel,
    )?.model_id ?? "",
  );
  let observation = $state<InferenceObservationView | null>(null);
  let running = $state(false);
  let showDisclosure = $state(false);
  // Generic, controlled IPC-failure message (no raw error detail). The backend
  // command returns a controlled state for every outcome; this only covers a
  // transport-level IPC failure.
  let observationError = $state<string | null>(null);
  let runButton = $state<HTMLButtonElement>();
  let dialogElement = $state<HTMLDivElement>();
  let cancelButton = $state<HTMLButtonElement>();

  let canRun = $derived(
    selectableModels.length > 0 &&
      selectedModel !== "" &&
      !running &&
      !showDisclosure,
  );

  // If this workspace is hidden while the authorization disclosure is open,
  // close the disclosure immediately. Do not restore focus here — the active
  // workspace tab already owns focus after the user navigates away.
  $effect(() => {
    if (!isActive && showDisclosure) {
      showDisclosure = false;
    }
  });

  function openDisclosure() {
    if (!canRun) return;
    observationError = null;
    showDisclosure = true;
    requestAnimationFrame(() => cancelButton?.focus());
  }

  function cancelDisclosure() {
    showDisclosure = false;
    requestAnimationFrame(() => runButton?.focus());
  }

  function handleDialogKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      cancelDisclosure();
      return;
    }
    if (event.key !== "Tab") return;
    if (!dialogElement) return;
    const controls = Array.from(
      dialogElement.querySelectorAll<HTMLElement>("button:not([disabled])"),
    );
    const first = controls[0];
    const last = controls.at(-1);
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  }

  async function authorizeRun() {
    // The user acknowledged the undetermined execution boundary for this one
    // run. `true` is NOT locality proof and NOT cryptographic; it authorizes
    // one synthetic observation only.
    showDisclosure = false;
    running = true;
    observation = null;
    observationError = null;
    try {
      const result = await dataSource.runInferenceObservation({
        provider: selectedProvider,
        model: selectedModelId,
        execution_boundary_acknowledged: true,
        provider_behavior_acknowledged: selectedProvider === "lm_studio",
      });
      observation = result;
      onObservation(result);
      if (
        result.state === "completed" &&
        typeof result.model === "string" &&
        result.model.trim().length > 0 &&
        typeof result.diagnostic_profile === "string" &&
        result.diagnostic_profile.trim().length > 0
      ) {
        onCompletedObservation(result);
      }
    } catch {
      // Surface only a controlled, generic message — never raw internals.
      observationError = "The observation could not run right now.";
    } finally {
      running = false;
    }
  }

  // Display a nanosecond duration as seconds (the only unit transform 1G does;
  // no date/time dependency).
  function nsToSeconds(ns: number): string {
    return (ns / 1_000_000_000).toFixed(3) + " s";
  }

  function formatRate(rate: number): string {
    return rate.toFixed(2) + " tokens/s";
  }

  let showMetrics = $derived(
    observation !== null && observation.state === "completed",
  );
</script>

<section class="observed-inference" aria-label="Observed inference">
  <EvidenceLabel label="Observed inference" />
  <h2 class="section-heading">Observed inference</h2>
  <p class="observed-inference-intro">
    Run one fixed diagnostic observation against a model you select from the
    catalogue. Each run needs its own authorization.
  </p>
  <p class="observed-inference-intro">
    llama.cpp is passive-only in this phase and is not available for observed
    inference.
  </p>

  {#if selectableModels.length > 0}
    <div class="observed-inference-controls">
      <label class="model-select-label" for="observed-inference-model">
        Model
      </label>
      <select
        id="observed-inference-model"
        class="model-select"
        bind:value={selectedModel}
        disabled={running}
      >
        <option value="" disabled>Select a model</option>
        {#each selectableModels as model (`${model.provider}:${model.model_id}`)}
          <option value={`${model.provider}:${model.model_id}`}>
            {model.provider === "ollama" ? "Ollama" : "LM Studio"} — {model.display_name}{model.loaded ===
            false
              ? " (not currently loaded)"
              : ""}
          </option>
        {/each}
      </select>
      <button
        bind:this={runButton}
        type="button"
        class="run-observation"
        onclick={openDisclosure}
        disabled={!canRun}
      >
        {DISCLOSURE_PRIMARY_ACTION}
      </button>
    </div>
  {:else}
    <p class="observed-inference-unavailable">
      The model catalogue is not available right now, so no observation can be
      run.
    </p>
  {/if}

  {#if running}
    <p class="state-message" role="status">Running one observation…</p>
  {/if}

  {#if observationError}
    <p class="state-message" role="alert">{observationError}</p>
  {/if}

  {#if observation}
    <div class="observation-result" aria-live="polite">
      <p class="observation-interpretation">{observation.interpretation}</p>
      <p class="observation-why-it-matters">{observation.why_it_matters}</p>
      <p class="observation-limitations">{observation.limitations}</p>
      <p class="observation-execution-boundary">
        <span class="evidence-label evidence-label-inline">Qualification</span>
        <span class="observation-field-label">Execution boundary</span>
        <span class="observation-field-value">
          {observation.execution_boundary}
        </span>
      </p>
      <p class="observation-provider">
        <span class="observation-field-label">Provider</span>
        <span class="observation-field-value">
          {observation.provider === "lm_studio" ? "LM Studio" : "Ollama"}
        </span>
      </p>
      {#if showMetrics}
        <dl class="observation-metrics">
          {#if observation.prompt_eval_count !== null}
            <dt>Prompt tokens</dt>
            <dd>{observation.prompt_eval_count.toLocaleString("en-US")}</dd>
          {/if}
          {#if observation.eval_count !== null}
            <dt>Generated tokens</dt>
            <dd>{observation.eval_count.toLocaleString("en-US")}</dd>
          {/if}
          {#if observation.load_duration_ns !== null}
            <dt>Load time</dt>
            <dd>{nsToSeconds(observation.load_duration_ns)}</dd>
          {/if}
          {#if observation.prompt_eval_duration_ns !== null}
            <dt>Prompt evaluation time</dt>
            <dd>{nsToSeconds(observation.prompt_eval_duration_ns)}</dd>
          {/if}
          {#if observation.eval_duration_ns !== null}
            <dt>Generation time</dt>
            <dd>{nsToSeconds(observation.eval_duration_ns)}</dd>
          {/if}
          {#if observation.total_duration_ns !== null}
            <dt>Total time</dt>
            <dd>{nsToSeconds(observation.total_duration_ns)}</dd>
          {/if}
          {#if observation.generation_tokens_per_second !== null}
            <dt>{GENERATION_RATE_LABEL}</dt>
            <dd>{formatRate(observation.generation_tokens_per_second)}</dd>
          {/if}
          {#if observation.input_tokens != null}
            <dt>LM Studio-reported input tokens</dt>
            <dd>{observation.input_tokens.toLocaleString("en-US")}</dd>
          {/if}
          {#if observation.total_output_tokens != null}
            <dt>LM Studio-reported total output tokens</dt>
            <dd>{observation.total_output_tokens.toLocaleString("en-US")}</dd>
          {/if}
          {#if observation.reasoning_output_tokens != null}
            <dt>LM Studio-reported reasoning output tokens</dt>
            <dd>
              {observation.reasoning_output_tokens.toLocaleString("en-US")}
            </dd>
          {/if}
          {#if observation.provider_reported_tokens_per_second != null}
            <dt>LM Studio-reported generation rate</dt>
            <dd>
              {formatRate(observation.provider_reported_tokens_per_second)}
            </dd>
          {/if}
          {#if observation.time_to_first_token_seconds != null}
            <dt>LM Studio-reported time to first token</dt>
            <dd>{observation.time_to_first_token_seconds.toFixed(3)} s</dd>
          {/if}
          {#if observation.model_load_time_seconds != null}
            <dt>LM Studio-reported model load time</dt>
            <dd>{observation.model_load_time_seconds.toFixed(3)} s</dd>
          {/if}
        </dl>
        {#if observation.generation_tokens_per_second !== null}
          <p class="observation-rate-explanation">
            {GENERATION_RATE_EXPLANATION}
          </p>
        {/if}
      {/if}
    </div>
  {/if}
</section>

{#if showDisclosure}
  <div
    bind:this={dialogElement}
    class="disclosure-overlay"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="observed-inference-disclosure-title"
    onkeydown={handleDialogKeydown}
  >
    <div class="disclosure-dialog">
      <h3 id="observed-inference-disclosure-title" class="disclosure-title">
        {DISCLOSURE_TITLE}
      </h3>
      <div class="disclosure-body">
        {#each selectedProvider === "lm_studio" ? ["AI Engine Room will send one fixed synthetic observation to the selected LM Studio model.", "No user, repository, or machine content is included.", "LM Studio may JIT-load an unloaded model and may later auto-unload it according to its configuration. AI Engine Room does not call model-management APIs.", "The API is limited to same-machine loopback; exact compute placement is not independently verified.", "This is one observed run, not a benchmark. This authorization applies to this run only."] : DISCLOSURE_BODY as paragraph (paragraph)}
          <p class="disclosure-paragraph">{paragraph}</p>
        {/each}
      </div>
      <div class="disclosure-actions">
        <button
          bind:this={cancelButton}
          type="button"
          class="disclosure-cancel"
          onclick={cancelDisclosure}
        >
          {DISCLOSURE_SECONDARY_ACTION}
        </button>
        <button type="button" class="disclosure-confirm" onclick={authorizeRun}>
          {DISCLOSURE_PRIMARY_ACTION}
        </button>
      </div>
    </div>
  </div>
{/if}
