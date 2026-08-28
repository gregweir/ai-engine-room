<script lang="ts">
  // Minimal runtime-status presentation (Milestone 1C). Renders only the
  // controlled text the backend authors — `status_label`, `interpretation`,
  // `why_it_matters` — plus the verbatim version when one was reported. The
  // card does NOT infer a detection state from `source_availability`, status
  // codes, or transport kinds; it shows the backend's pre-computed wording
  // verbatim. No charts, gauges, animations, controls, or model lists.
  import type { RuntimeStatusView } from "../types";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let { status }: { status: RuntimeStatusView } = $props();

  // Friendly runtime name for the heading. The controlled lowercase id
  // (`"ollama"`) is presentation only here; the backend owns all technical
  // wording.
  let runtimeName = $derived(
    status.runtime_type === "ollama" ? "Ollama" : status.runtime_type,
  );
</script>

<section class="runtime-status-card">
  <EvidenceLabel label="Provider-reported state" />
  <h3 class="runtime-name">{runtimeName}</h3>
  <p class="runtime-status-label">{status.status_label}</p>
  {#if status.version}
    <p class="runtime-version">
      Version {status.version}
      {#if status.version_reported_by === "runtime_reported"}
        <span class="runtime-version-source">(reported by the runtime)</span>
      {/if}
    </p>
  {/if}
  <div class="explanation">
    <p class="interpretation">{status.interpretation}</p>
    <p class="why-it-matters">{status.why_it_matters}</p>
  </div>
</section>
