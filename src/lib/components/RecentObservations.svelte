<script lang="ts">
  import {
    COMPARISON_LIMITATION,
    GENERATED_TOKEN_LIMITATION,
    LOAD_DURATION_LIMITATION,
    PROFILE_MISMATCH,
    PROVIDER_MISMATCH,
    comparisonSelection,
    type ObservationHistoryItem,
  } from "../observationHistory";

  let {
    history,
    selectedObservationIds = $bindable(),
    retainedAnnouncement = "",
    onClear,
  }: {
    history: ObservationHistoryItem[];
    selectedObservationIds: number[];
    retainedAnnouncement?: string;
    onClear: () => void;
  } = $props();

  let comparison = $derived(
    comparisonSelection(selectedObservationIds, history),
  );
  let announcement = $state("");

  function toggleSelection(id: number) {
    if (selectedObservationIds.includes(id)) {
      selectedObservationIds = selectedObservationIds.filter(
        (selectedId) => selectedId !== id,
      );
      announcement = `Observation ${id} removed from comparison.`;
      return;
    }
    if (selectedObservationIds.length >= 2) return;
    const nextSelectedIds = [...selectedObservationIds, id];
    selectedObservationIds = nextSelectedIds;
    const nextComparison = comparisonSelection(nextSelectedIds, history);
    if (nextComparison.kind === "eligible") {
      announcement = "Two observations selected. Comparison is available.";
    } else if (nextComparison.kind === "profile_mismatch") {
      announcement =
        "Two observations selected. Direct comparison is unavailable because their diagnostic profiles differ or are unknown.";
    } else if (nextComparison.kind === "provider_mismatch") {
      announcement =
        "Two observations selected. Direct comparison is unavailable across runtime providers.";
    } else {
      announcement = `Observation ${id} selected for comparison.`;
    }
  }

  function clearHistory() {
    onClear();
    announcement = "Recent observations cleared.";
  }

  function seconds(value: number | null): string | null {
    return value === null ? null : (value / 1_000_000_000).toFixed(3) + " s";
  }

  function reportedSeconds(value: number | null | undefined): string | null {
    return value == null ? null : value.toFixed(3) + " s";
  }

  function rate(value: number | null): string | null {
    return value === null ? null : value.toFixed(2) + " tokens/s";
  }

  function count(value: number | null): string | null {
    return value === null ? null : value.toLocaleString("en-US");
  }

  function executionLocation(boundary: string): string {
    return boundary === "undetermined"
      ? "Execution location not determined"
      : `Execution location: ${boundary}`;
  }
</script>

<section
  class="recent-observations"
  aria-labelledby="recent-observations-title"
>
  <div class="recent-observations-heading">
    <h3 id="recent-observations-title">Recent observations</h3>
    {#if history.length > 0}
      <button type="button" class="clear-observations" onclick={clearHistory}>
        Clear recent observations
      </button>
    {/if}
  </div>

  <p class="comparison-limitation">{COMPARISON_LIMITATION}</p>
  <p class="observation-announcement" aria-live="polite">
    {retainedAnnouncement}
  </p>
  <p class="observation-announcement" aria-live="polite">{announcement}</p>

  {#if history.length === 0}
    <p class="recent-observations-empty">
      Completed observations from this app session will appear here.
    </p>
  {:else}
    <ul class="observation-history-list">
      {#each history as item (item.observation_id)}
        <li class="observation-history-item">
          <label class="observation-selection">
            <input
              type="checkbox"
              checked={selectedObservationIds.includes(item.observation_id)}
              disabled={selectedObservationIds.length >= 2 &&
                !selectedObservationIds.includes(item.observation_id)}
              onchange={() => toggleSelection(item.observation_id)}
              aria-label={`Select Observation ${item.observation_id} (${item.observation.model}) for comparison`}
              aria-describedby="comparison-selection-instruction"
            />
            <span class="observation-sequence">
              Observation {item.observation_id}
            </span>
          </label>
          <dl class="observation-history-summary">
            <div>
              <dt>Provider</dt>
              <dd>
                {item.observation.provider === "lm_studio"
                  ? "LM Studio"
                  : "Ollama"}
              </dd>
            </div>
            <div>
              <dt>Model</dt>
              <dd>{item.observation.model}</dd>
            </div>
            <div>
              <dt>Diagnostic profile</dt>
              <dd>{item.observation.diagnostic_profile}</dd>
            </div>
            <div>
              <dt>
                {item.observation.provider === "lm_studio"
                  ? "LM Studio-reported total output tokens"
                  : "Generated tokens"}
              </dt>
              <dd>
                {#if count(item.observation.provider === "lm_studio" ? (item.observation.total_output_tokens ?? null) : item.observation.eval_count) !== null}
                  {count(
                    item.observation.provider === "lm_studio"
                      ? (item.observation.total_output_tokens ?? null)
                      : item.observation.eval_count,
                  )}
                {:else}
                  <span aria-label="Not reported">—</span>
                {/if}
              </dd>
            </div>
            <div>
              <dt>
                {item.observation.provider === "lm_studio"
                  ? "LM Studio-reported time to first token"
                  : "Ollama-reported generation duration"}
              </dt>
              <dd>
                {#if item.observation.provider === "lm_studio" ? reportedSeconds(item.observation.time_to_first_token_seconds) !== null : seconds(item.observation.eval_duration_ns) !== null}
                  {item.observation.provider === "lm_studio"
                    ? reportedSeconds(
                        item.observation.time_to_first_token_seconds,
                      )
                    : seconds(item.observation.eval_duration_ns)}
                {:else}
                  <span aria-label="Not reported">—</span>
                {/if}
              </dd>
            </div>
            <div>
              <dt>
                {item.observation.provider === "lm_studio"
                  ? "LM Studio-reported generation rate"
                  : "Observed generation rate (Engine Room calculated)"}
              </dt>
              <dd>
                {#if rate(item.observation.provider === "lm_studio" ? (item.observation.provider_reported_tokens_per_second ?? null) : item.observation.generation_tokens_per_second) !== null}
                  {rate(
                    item.observation.provider === "lm_studio"
                      ? (item.observation.provider_reported_tokens_per_second ??
                          null)
                      : item.observation.generation_tokens_per_second,
                  )}
                {:else}
                  <span aria-label="Not reported">—</span>
                {/if}
              </dd>
            </div>
            <div>
              <dt>Execution location</dt>
              <dd>{executionLocation(item.observation.execution_boundary)}</dd>
            </div>
          </dl>
        </li>
      {/each}
    </ul>
    <p id="comparison-selection-instruction" class="comparison-selection-note">
      {#if selectedObservationIds.length === 2}
        Two observations are selected. Deselect one before selecting another.
      {:else}
        Select two observations with the same diagnostic profile to compare
        their reported values.
      {/if}
    </p>
  {/if}

  {#if comparison.kind === "profile_mismatch"}
    <section class="observation-comparison" aria-labelledby="comparison-title">
      <h3 id="comparison-title">Compare observations</h3>
      <p class="profile-mismatch" role="status">{PROFILE_MISMATCH}</p>
    </section>
  {:else if comparison.kind === "provider_mismatch"}
    <section class="observation-comparison" aria-labelledby="comparison-title">
      <h3 id="comparison-title">Compare observations</h3>
      <p class="profile-mismatch" role="status">{PROVIDER_MISMATCH}</p>
    </section>
  {:else if comparison.kind === "eligible"}
    <section class="observation-comparison" aria-labelledby="comparison-title">
      <h3 id="comparison-title">Compare observations</h3>
      <p class="comparison-limitation">{COMPARISON_LIMITATION}</p>
      <div
        class="comparison-table-scroll"
        role="region"
        aria-label="Observation comparison table"
      >
        <table class="comparison-table">
          <thead>
            <tr>
              <th scope="col">Reported field</th>
              <th scope="col">Observation {comparison.first.observation_id}</th>
              <th scope="col">Observation {comparison.second.observation_id}</th
              >
            </tr>
          </thead>
          <tbody>
            <tr>
              <th scope="row">Observation ID</th>
              <td>{comparison.first.observation_id}</td>
              <td>{comparison.second.observation_id}</td>
            </tr>
            <tr>
              <th scope="row">Selected model</th>
              <td>{comparison.first.observation.model}</td>
              <td>{comparison.second.observation.model}</td>
            </tr>
            <tr>
              <th scope="row">Diagnostic profile</th>
              <td>{comparison.first.observation.diagnostic_profile}</td>
              <td>{comparison.second.observation.diagnostic_profile}</td>
            </tr>
            <tr>
              <th scope="row">Execution location</th>
              <td
                >{executionLocation(
                  comparison.first.observation.execution_boundary,
                )}</td
              >
              <td
                >{executionLocation(
                  comparison.second.observation.execution_boundary,
                )}</td
              >
            </tr>
            <tr>
              <th scope="row">Ollama-reported prompt tokens</th>
              <td>
                {#if count(comparison.first.observation.prompt_eval_count) !== null}
                  {count(comparison.first.observation.prompt_eval_count)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if count(comparison.second.observation.prompt_eval_count) !== null}
                  {count(comparison.second.observation.prompt_eval_count)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row">Ollama-reported generated tokens</th>
              <td>
                {#if count(comparison.first.observation.eval_count) !== null}
                  {count(comparison.first.observation.eval_count)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if count(comparison.second.observation.eval_count) !== null}
                  {count(comparison.second.observation.eval_count)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row">Ollama-reported load duration</th>
              <td>
                {#if seconds(comparison.first.observation.load_duration_ns) !== null}
                  {seconds(comparison.first.observation.load_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if seconds(comparison.second.observation.load_duration_ns) !== null}
                  {seconds(comparison.second.observation.load_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row">Ollama-reported prompt-evaluation duration</th>
              <td>
                {#if seconds(comparison.first.observation.prompt_eval_duration_ns) !== null}
                  {seconds(
                    comparison.first.observation.prompt_eval_duration_ns,
                  )}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if seconds(comparison.second.observation.prompt_eval_duration_ns) !== null}
                  {seconds(
                    comparison.second.observation.prompt_eval_duration_ns,
                  )}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row">Ollama-reported generation duration</th>
              <td>
                {#if seconds(comparison.first.observation.eval_duration_ns) !== null}
                  {seconds(comparison.first.observation.eval_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if seconds(comparison.second.observation.eval_duration_ns) !== null}
                  {seconds(comparison.second.observation.eval_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row">Ollama-reported total duration</th>
              <td>
                {#if seconds(comparison.first.observation.total_duration_ns) !== null}
                  {seconds(comparison.first.observation.total_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if seconds(comparison.second.observation.total_duration_ns) !== null}
                  {seconds(comparison.second.observation.total_duration_ns)}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
            <tr>
              <th scope="row"
                >Observed generation rate (Engine Room calculated)</th
              >
              <td>
                {#if rate(comparison.first.observation.generation_tokens_per_second) !== null}
                  {rate(
                    comparison.first.observation.generation_tokens_per_second,
                  )}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
              <td>
                {#if rate(comparison.second.observation.generation_tokens_per_second) !== null}
                  {rate(
                    comparison.second.observation.generation_tokens_per_second,
                  )}
                {:else}<span aria-label="Not reported">—</span>{/if}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="comparison-caveat">{LOAD_DURATION_LIMITATION}</p>
      <p class="comparison-caveat">{GENERATED_TOKEN_LIMITATION}</p>
    </section>
  {/if}
</section>
