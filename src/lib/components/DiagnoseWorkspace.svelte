<script lang="ts">
  import {
    sourceCoverage,
    type DiagnosticDeltaResult,
    type DiagnosticObservationBundle,
  } from "../diagnosticObservation";
  import type { DiagnosticFindingView } from "../types";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let {
    history,
    changes,
    findings,
    diagnosisState,
  }: {
    history: DiagnosticObservationBundle[];
    changes: DiagnosticDeltaResult;
    findings: DiagnosticFindingView[];
    diagnosisState: "available" | "invalid_input" | "unavailable";
  } = $props();

  let latest = $derived(history.at(-1) ?? null);
  let latestCoverage = $derived(latest ? sourceCoverage(latest) : []);

  function readable(value: string): string {
    return value.replaceAll("_", " ");
  }

  function sourceLabel(source: DiagnosticFindingView["source"]): string {
    switch (source) {
      case "ollama":
        return "Ollama";
      case "lm_studio":
        return "LM Studio";
      case "llama_cpp":
        return "llama.cpp";
      case "observation":
        return "Explicit observation";
      case "cross_provider":
        return "Provider qualification";
    }
  }
</script>

<section
  class="workspace-content diagnose-workspace"
  aria-labelledby="diagnose-heading"
>
  <h2 id="diagnose-heading">Diagnose</h2>
  <p class="workspace-intro">
    Factual changes and controlled next checks from startup and explicit Refresh
    observations. This is session-only observation history, not continuous
    monitoring.
  </p>

  <section class="diagnose-scope" aria-labelledby="diagnose-scope-heading">
    <EvidenceLabel label="Qualification" />
    <h3 id="diagnose-scope-heading">Observation scope</h3>
    {#if latest}
      <p class="diagnose-latest">Latest observation: {latest.observation_id}</p>
      <p>
        {history.length} retained {history.length === 1
          ? "observation"
          : "observations"}
        this session. Startup counts once; each completed explicit Refresh attempt
        counts once.
      </p>
    {:else}
      <p>No diagnostic observation has completed this session.</p>
    {/if}
    <p class="diagnose-qualification">
      Sequence numbers show acquisition order, not elapsed time. Sources are
      acquired independently, so one observation bundle is not an atomic machine
      snapshot.
    </p>
  </section>

  <section aria-labelledby="diagnose-coverage-heading">
    <h3 id="diagnose-coverage-heading">Source coverage</h3>
    {#if latestCoverage.length > 0}
      <dl class="diagnose-coverage">
        {#each latestCoverage as coverage (coverage.source)}
          <div class:diagnose-source-gap={coverage.gap}>
            <dt>{coverage.source}</dt>
            <dd>{readable(coverage.state)}</dd>
          </div>
        {/each}
      </dl>
      <p class="diagnose-qualification">
        A source gap is unknown, not an empty model set or numeric zero. Raw
        acquisition errors are not retained here.
      </p>
    {:else}
      <p>Source coverage is not available yet.</p>
    {/if}
  </section>

  <section aria-labelledby="diagnose-changes-heading">
    <h3 id="diagnose-changes-heading">Changes since previous observation</h3>
    {#if changes.state === "no_previous"}
      <p>Observation 1 has no previous explicit observation to compare.</p>
    {:else if changes.state === "no_supported_change"}
      <p>No supported change was observed.</p>
    {:else}
      <p>
        {changes.changes.length} supported factual
        {changes.changes.length === 1 ? "change" : "changes"} observed.
      </p>
      <ol class="diagnose-change-list">
        {#each changes.changes as change (`${change.code}:${change.summary}`)}
          <li>
            <p>{change.summary}</p>
            {#if change.details.length > 0}
              <ul>
                {#each change.details as detail (detail)}
                  <li>{detail}</li>
                {/each}
              </ul>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}
    <p class="diagnose-qualification">
      These statements describe what differed. They do not establish why it
      changed and are not a recommendation or execution-location claim.
    </p>
  </section>

  <section aria-labelledby="diagnose-findings-heading">
    <h3 id="diagnose-findings-heading">Deterministic findings</h3>
    {#if diagnosisState === "invalid_input"}
      <p>
        Findings are unavailable because the bounded diagnostic input could not
        be validated.
      </p>
    {:else if diagnosisState === "unavailable"}
      <p>
        Controlled diagnostic findings are unavailable for this observation.
      </p>
    {:else if findings.length === 0}
      <p>No deterministic finding applies to the latest observation.</p>
    {:else}
      <div class="diagnostic-findings">
        {#each findings as finding (`${finding.code}:${finding.observation}`)}
          <article class="diagnostic-finding">
            <EvidenceLabel label={sourceLabel(finding.source)} />
            <h4>{readable(finding.code)}</h4>
            <dl>
              <div>
                <dt>Observation</dt>
                <dd>{finding.observation}</dd>
              </div>
              <div>
                <dt>Meaning</dt>
                <dd>{finding.meaning}</dd>
              </div>
              <div>
                <dt>Safe next check</dt>
                <dd>{finding.safe_next_check}</dd>
              </div>
            </dl>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  <section aria-labelledby="diagnose-history-heading">
    <h3 id="diagnose-history-heading">Observation history</h3>
    <p class="diagnose-qualification">
      Newest 12 at most, ordered by observation sequence. No timestamps,
      persistence, background sampling, or time axis.
    </p>
    {#if history.length > 0}
      <ol class="diagnose-history-list">
        {#each history as observation (observation.observation_id)}
          <li>
            <details>
              <summary
                >Diagnostic observation {observation.observation_id}</summary
              >
              <dl class="diagnose-history-coverage">
                {#each sourceCoverage(observation) as coverage (coverage.source)}
                  <div class:diagnose-source-gap={coverage.gap}>
                    <dt>{coverage.source}</dt>
                    <dd>{readable(coverage.state)}</dd>
                  </div>
                {/each}
              </dl>
            </details>
          </li>
        {/each}
      </ol>
    {:else}
      <p>No observations retained this session.</p>
    {/if}
  </section>
</section>
