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
  import type { LoadedModelSetView } from "../types";
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
  <div class="explanation resource-qualification">
    <p>{loaded.resource_interpretation}</p>
    <p>{loaded.resource_qualification}</p>
  </div>

  {#if hasModels}
    <ul class="model-list">
      {#each loaded.models as model (model.display_name)}
        <li class="model-entry">
          <span class="model-name">{model.display_name}</span>
          <dl class="model-details">
            <dt>Loaded size reported by Ollama</dt>
            <dd>{model.reported_size_bytes === null
                ? "Not reported by Ollama"
                : formatBytes(model.reported_size_bytes)}</dd>
            <dt>VRAM size reported by Ollama</dt>
            <dd>{model.reported_vram_bytes === null
                ? "Not reported by Ollama"
                : formatBytes(model.reported_vram_bytes)}</dd>
            <dt>Configured context reported by Ollama</dt>
            <dd>{model.reported_context_length === null
                ? "Not reported by Ollama"
                : model.reported_context_length.toLocaleString()}</dd>
          </dl>
        </li>
      {/each}
    </ul>
  {/if}
</section>
