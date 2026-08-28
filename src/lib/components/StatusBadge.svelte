<script lang="ts">
  import type { SnapshotEntryView } from "../types";
  import { metricStatus } from "./status";

  let { entry }: { entry: SnapshotEntryView } = $props();
  let status = $derived(metricStatus(entry));
</script>

<span class="status-badge" data-tone={status.tone} data-marker={status.marker}>
  <span class="status-marker" aria-hidden="true"></span>
  <span class="status-label">{status.label}</span>
</span>
