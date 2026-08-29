<script lang="ts">
  // Minimal read-only resource-context explanatory note (Milestone 1F). Renders
  // ONLY the controlled text the backend authors — `interpretation`,
  // `why_it_matters`, and nonnumeric concept states — verbatim, under an
  // accessible "Resource context" heading.
  // It juxtaposes OS-reported Available memory with Ollama-reported loaded-model
  // state as context only; it does NOT attribute exact RAM/VRAM consumption to a
  // model, and Engine Room does not treat the values as an exact account.
  //
  // The component performs no I/O and holds no state. It groups the retained,
  // already-acquired metric cards beneath the Rust-authored explanation without
  // choosing variants, combining values, or inferring relationships. Manual
  // Refresh (in App.svelte) remains the only acquisition trigger.
  import type { ResourceContextView, SnapshotEntryView } from "../types";
  import MetricCard from "./MetricCard.svelte";

  let {
    context,
    entries = [],
  }: { context: ResourceContextView; entries?: SnapshotEntryView[] } = $props();
</script>

<section class="resource-context" aria-label="Resource context">
  <h2 class="resource-context-heading">Resource context</h2>
  <p class="resource-warning">
    Values come from different sources. Available memory and Ollama-reported
    model values must not be added or subtracted to infer exact model memory
    use.
  </p>
  <details class="resource-interpretation">
    <summary>How to interpret memory and loaded-model values</summary>
    <div class="explanation">
      <p class="interpretation">{context.interpretation}</p>
      <p class="why-it-matters">{context.why_it_matters}</p>
    </div>
  </details>
  <section aria-labelledby="resource-evidence-heading">
    <h3 id="resource-evidence-heading">What the current evidence establishes</h3>
    <dl class="resource-concepts">
      {#each context.concepts as item (item.concept)}
        <div class="resource-concept">
          <dt>{item.concept}</dt>
          <dd>
            <strong>{item.state_label}</strong>
            <span>{item.interpretation}</span>
          </dd>
        </div>
      {/each}
    </dl>
  </section>
  <details class="metric-details">
    <summary>Metric details</summary>
    <section class="metrics" aria-label="Current metric states">
      <h3 class="visually-hidden">Current metric states</h3>
      {#each entries as entry (entry.display_name)}<MetricCard {entry} />{/each}
    </section>
  </details>
</section>
