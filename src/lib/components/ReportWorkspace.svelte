<script lang="ts">
  import { tick } from "svelte";
  import type { ReportClipboard } from "../reportClipboard";
  import { reportSaveMessage, reportSaveRole } from "../reportSave";
  import type { ReportSaveResult } from "../types";
  import ReportPreview from "./ReportPreview.svelte";

  let {
    preview,
    generation = "",
    saveReport = null,
    clipboard,
    clipboardState,
  }: {
    preview: string;
    generation?: string;
    saveReport?: ((generation: string) => Promise<ReportSaveResult>) | null;
    clipboard: ReportClipboard | null;
    clipboardState:
      "browser-unavailable" | "native-ready" | "native-init-failed";
  } = $props();
  let pending = $state(false);
  let outcome = $state<"idle" | "success" | "failure">("idle");
  let savePending = $state(false);
  let saveOutcome = $state<ReportSaveResult | null>(null);
  let saveButton = $state<HTMLButtonElement>();

  async function copyReport() {
    if (pending || preview.length === 0 || clipboard === null) return;

    pending = true;
    outcome = "idle";
    try {
      await clipboard.copyText(preview);
      outcome = "success";
    } catch {
      outcome = "failure";
    } finally {
      pending = false;
    }
  }

  async function saveReportFile() {
    if (
      savePending ||
      preview.length === 0 ||
      generation.length === 0 ||
      saveReport === null
    )
      return;

    savePending = true;
    saveOutcome = null;
    try {
      saveOutcome = await saveReport(generation);
    } catch {
      saveOutcome = "failed";
    } finally {
      savePending = false;
      await tick();
      saveButton?.focus();
    }
  }
</script>

<section class="workspace-content" aria-labelledby="report-heading">
  <h2 id="report-heading">Report</h2>
  <p class="workspace-intro">
    This workspace is intentionally narrower than the full app state.
  </p>
  <ReportPreview {preview} />
  <div class="report-clipboard">
    <p class="report-clipboard-note">
      Copy report places this report text on the system clipboard. Other
      applications on this computer may be able to read clipboard contents.
      Clipboard contents may remain after AI Engine Room closes until
      overwritten. AI Engine Room does not automatically send or upload the
      report.
    </p>
    {#if clipboardState === "browser-unavailable"}
      <p class="report-clipboard-unavailable">
        Copy report is available in the native app.
      </p>
    {:else if clipboardState === "native-init-failed"}
      <p class="report-clipboard-unavailable">
        Clipboard export could not be initialized in this native session.
      </p>
    {:else}
      <button
        class="copy-report"
        type="button"
        disabled={pending || preview.length === 0 || clipboard === null}
        aria-busy={pending}
        onclick={copyReport}
      >
        {pending ? "Copying report…" : "Copy report"}
      </button>
      {#if preview.length === 0}
        <p class="report-clipboard-unavailable">
          No report-safe preview is available to copy.
        </p>
      {:else if clipboard === null}
        <p class="report-clipboard-unavailable">
          Copy report is currently unavailable.
        </p>
      {/if}
      {#if outcome === "success"}
        <p class="report-clipboard-status" role="status">
          Report copied to the system clipboard.
        </p>
      {:else if outcome === "failure"}
        <p class="report-clipboard-status" role="alert">
          Could not copy the report to the system clipboard.
        </p>
      {/if}
    {/if}
  </div>
  <div class="report-save">
    {#if saveReport === null}
      <p class="report-save-unavailable">
        Save report is available in the native app.
      </p>
    {:else}
      <p class="report-save-note">
        Save report creates a plain-text file that remains in the location you
        choose until you remove it. Other software may read or synchronize that
        file. AI Engine Room does not upload or remember it.
      </p>
      <button
        bind:this={saveButton}
        class="save-report"
        type="button"
        disabled={savePending ||
          preview.length === 0 ||
          generation.length === 0}
        aria-busy={savePending}
        onclick={saveReportFile}
      >
        {savePending ? "Saving report…" : "Save report…"}
      </button>
      {#if saveOutcome}
        <p class="report-save-status" role={reportSaveRole(saveOutcome)}>
          {reportSaveMessage(saveOutcome)}
        </p>
      {/if}
    {/if}
  </div>
</section>
