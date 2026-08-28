<script lang="ts">
  // Minimal read-only model inventory presentation (Milestone 1D). Renders only
  // the controlled text the backend authors — `state_label`, `interpretation`,
  // `why_it_matters` — plus one row per reported model with its name, reported
  // size, and the optional secondary labels. The card does NOT infer an
  // inventory state from `source_availability`, status codes, or transport
  // kinds; it shows the backend's pre-computed wording verbatim. No charts,
  // gauges, animations, or model actions (no delete/pull/launch/benchmark).
  //
  // Locality is carried in the DTO for forward use but is NOT displayed per
  // model in 1D (every entry is "undetermined"; a repetitive badge adds no
  // value). No `digest`, endpoint, host, port, or `remote_host` is shown.
  import type { ModelInventoryView, RuntimeModelSummaryView } from "../types";
  import { formatBytes } from "../formatBytes";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let { inventory }: { inventory: ModelInventoryView } = $props();

  // Friendly runtime name for the heading. The controlled lowercase id
  // (`"ollama"`) is presentation only here; the backend owns all technical
  // wording.
  let runtimeName = $derived(
    inventory.runtime_type === "ollama" ? "Ollama" : inventory.runtime_type,
  );

  // The model list renders only for a populated available catalogue. A valid
  // empty catalogue is a normal state (not an error) and shows the state line
  // alone.
  let hasModels = $derived(
    inventory.state === "available" && inventory.models.length > 0,
  );
  let filter = $state("");
  let filteredModels = $derived(
    inventory.models.filter((model) =>
      model.display_name
        .toLocaleLowerCase()
        .includes(filter.toLocaleLowerCase()),
    ),
  );

  // Whether a model has any secondary detail to show (size, parameter size,
  // quantization, or family). Avoids rendering an empty `<dl>` for an entry
  // that reported only a name.
  function hasDetails(model: RuntimeModelSummaryView): boolean {
    return (
      model.size_bytes !== null ||
      model.parameter_size !== null ||
      model.quantization !== null ||
      model.families.length > 0
    );
  }
</script>

<section class="runtime-inventory-card">
  <EvidenceLabel label="Provider-reported state" />
  <h3 class="runtime-name">{runtimeName}</h3>
  <p class="runtime-inventory-label">{inventory.state_label}</p>
  <div class="explanation">
    <p class="interpretation">{inventory.interpretation}</p>
    <p class="why-it-matters">{inventory.why_it_matters}</p>
  </div>

  {#if hasModels}
    <div class="catalogue-filter">
      <label for="catalogue-filter">Filter catalogue by model name</label>
      <input id="catalogue-filter" type="search" bind:value={filter} />
      <p role="status">
        {filteredModels.length} of {inventory.models.length} catalogue models shown
      </p>
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex (the bounded results region must be keyboard-scrollable) -->
    <div
      class="catalogue-results"
      role="region"
      aria-label="Catalogue model results"
      tabindex="0"
    >
      <ul class="model-list">
        {#each filteredModels as model (model.display_name)}
          <li class="model-entry">
            <details>
              <summary class="model-name">{model.display_name}</summary>
              {#if hasDetails(model)}
                <dl class="model-details">
                  {#if model.size_bytes !== null}
                    <dt>Reported size</dt>
                    <dd>{formatBytes(model.size_bytes)}</dd>
                  {/if}
                  {#if model.parameter_size}
                    <dt>Parameter size</dt>
                    <dd>{model.parameter_size}</dd>
                  {/if}
                  {#if model.quantization}
                    <dt>Quantization</dt>
                    <dd>{model.quantization}</dd>
                  {/if}
                  {#if model.families.length > 0}
                    <dt>Family</dt>
                    <dd>{model.families.join(", ")}</dd>
                  {/if}
                </dl>
              {:else}<p class="model-no-details">
                  No additional catalogue details reported.
                </p>{/if}
            </details>
          </li>
        {/each}
      </ul>
      {#if filteredModels.length === 0}<p class="catalogue-no-match">
          No catalogue model names match this filter.
        </p>{/if}
    </div>
  {/if}
</section>
