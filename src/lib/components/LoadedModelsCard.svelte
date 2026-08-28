<script lang="ts">
  // Minimal read-only loaded-models presentation (Milestone 1E). Renders only
  // the controlled text the backend authors — `state_label`, `interpretation`,
  // `why_it_matters` — plus one row per currently loaded model with its name and
  // the sizes/counts Ollama reports for the loaded instance. The card does NOT
  // infer loaded-model state from `source_availability`, status codes, or
  // transport kinds; it shows the backend's pre-computed wording verbatim. No
  // charts, gauges, animations, or model actions (no load/unload/pull/delete/
  // generate/benchmark).
  //
  // `expires_at` is carried in the DTO but is NOT displayed here (no countdown,
  // no date/time dependency). No `digest`, endpoint, host, port, `remote_host`,
  // or locality badge is shown. No aggregate total is computed. Distinct from
  // the 1D catalogue card: this answers "what is running now?".
  import type { LoadedModelSetView, LoadedModelSummaryView } from "../types";
  import { formatBytes } from "../formatBytes";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let { loaded }: { loaded: LoadedModelSetView } = $props();

  // Friendly runtime name for the heading. The controlled lowercase id
  // (`"ollama"`) is presentation only here; the backend owns all technical
  // wording.
  let runtimeName = $derived(
    loaded.runtime_type === "ollama" ? "Ollama" : loaded.runtime_type,
  );

  // The model list renders only for a populated available loaded-model set. A
  // valid empty list is a normal state (not an error) and shows the state line
  // alone.
  let hasModels = $derived(
    loaded.state === "available" && loaded.models.length > 0,
  );

  // Whether a loaded entry has any detail to show (size, VRAM, or context
  // length). Avoids rendering an empty `<dl>` for an entry that reported only a
  // name.
  function hasDetails(model: LoadedModelSummaryView): boolean {
    return (
      model.reported_size_bytes !== null ||
      model.reported_vram_bytes !== null ||
      model.reported_context_length !== null
    );
  }

  // `expires_at` is carried in the DTO but is intentionally never referenced
  // here — it is not displayed (no countdown, no date/time dependency).
</script>

<section class="runtime-inventory-card">
  <EvidenceLabel label="Provider-reported state" />
  <h3 class="runtime-name">{runtimeName}</h3>
  <p class="runtime-inventory-label">{loaded.state_label}</p>
  <div class="explanation">
    <p class="interpretation">{loaded.interpretation}</p>
    <p class="why-it-matters">{loaded.why_it_matters}</p>
  </div>

  {#if hasModels}
    <ul class="model-list">
      {#each loaded.models as model (model.display_name)}
        <li class="model-entry">
          <span class="model-name">{model.display_name}</span>
          {#if hasDetails(model)}
            <dl class="model-details">
              {#if model.reported_size_bytes !== null}
                <dt>Loaded size</dt>
                <dd>{formatBytes(model.reported_size_bytes)}</dd>
              {/if}
              {#if model.reported_vram_bytes !== null}
                <dt>VRAM size</dt>
                <dd>{formatBytes(model.reported_vram_bytes)}</dd>
              {/if}
              {#if model.reported_context_length !== null}
                <dt>Context length</dt>
                <dd>{model.reported_context_length.toLocaleString()}</dd>
              {/if}
            </dl>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
