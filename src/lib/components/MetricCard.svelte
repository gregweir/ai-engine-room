<script lang="ts">
  import type { SnapshotEntryView } from "../types";
  import StatusBadge from "./StatusBadge.svelte";
  import Explanation from "./Explanation.svelte";
  import { formatValue, provenanceLabel, unitLabel } from "./labels";

  let { entry }: { entry: SnapshotEntryView } = $props();
  let value = $derived(
    entry.outcome.kind === "ok"
      ? `${formatValue(entry.outcome.value)} ${unitLabel(entry.outcome.unit)}`
      : null,
  );
  let provenance = $derived(
    entry.outcome.kind === "ok"
      ? provenanceLabel(entry.outcome.provenance)
      : null,
  );
</script>

<section class="metric-card">
  <h2 class="metric-name">{entry.display_name}</h2>
  <StatusBadge {entry} />
  {#if value}
    <p class="metric-value">{value}</p>
  {/if}
  {#if provenance}
    <p class="provenance">{provenance}</p>
  {/if}
  <Explanation {entry} />
</section>
