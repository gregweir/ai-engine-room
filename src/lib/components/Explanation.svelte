<script lang="ts">
  import type { SnapshotEntryView } from "../types";

  let { entry }: { entry: SnapshotEntryView } = $props();
</script>

<div class="explanation">
  <p class="interpretation">{entry.interpretation}</p>
  <p class="why-it-matters">{entry.why_it_matters}</p>
  {#if entry.deeper}
    <p class="deeper">{entry.deeper}</p>
  {/if}
</div>
