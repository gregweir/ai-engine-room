<script lang="ts">
  import type { AvailableMemoryHistoryItem } from "../availableMemoryHistory";
  import { unavailableReasonLabel } from "../availableMemoryHistory";
  import { formatBytesFriendly, formatExactBytes } from "../formatBytes";
  import EvidenceLabel from "./EvidenceLabel.svelte";

  let { history }: { history: AvailableMemoryHistoryItem[] } = $props();

  let numericItems = $derived(
    history.filter(
      (
        item,
      ): item is AvailableMemoryHistoryItem & {
        event: { kind: "value"; bytes: number };
      } => item.event.kind === "value",
    ),
  );
  let numericValues = $derived(numericItems.map((item) => item.event.bytes));
  let minimum = $derived(
    numericValues.length > 0 ? Math.min(...numericValues) : null,
  );
  let maximum = $derived(
    numericValues.length > 0 ? Math.max(...numericValues) : null,
  );
  let current = $derived(history.at(-1) ?? null);
  let valuesAreIdentical = $derived(
    numericValues.length > 1 && minimum === maximum,
  );

  function xPosition(index: number): number {
    if (history.length <= 1) return 300;
    return 48 + (index / (history.length - 1)) * 504;
  }

  function yPosition(bytes: number): number {
    if (minimum === null || maximum === null || minimum === maximum) return 39;
    return 64 - ((bytes - minimum) / (maximum - minimum)) * 50;
  }
</script>

<figure
  class="available-memory-history"
  aria-labelledby="available-memory-history-heading"
>
  <figcaption class="available-memory-history-heading">
    <div>
      <EvidenceLabel label="OS observation" />
      <h3 id="available-memory-history-heading">
        Available memory — this session
      </h3>
    </div>
    <p class="available-memory-count">
      {history.length}
      {history.length === 1 ? "observation" : "observations"};
      {numericItems.length} numeric
      {numericItems.length === 1 ? "reading" : "readings"}
    </p>
  </figcaption>

  <p class="available-memory-sequence-note">
    Startup and explicit Refresh observations only. Spacing shows observation
    sequence, not time.
  </p>

  {#if current?.event.kind === "value"}
    <dl class="available-memory-current">
      <div>
        <dt>Current</dt>
        <dd>{formatBytesFriendly(current.event.bytes)}</dd>
      </div>
      <div>
        <dt>Exact value</dt>
        <dd>{formatExactBytes(current.event.bytes)}</dd>
      </div>
      <div>
        <dt>Source</dt>
        <dd>Operating system</dd>
      </div>
    </dl>
  {:else if current}
    <dl class="available-memory-current">
      <div>
        <dt>Current</dt>
        <dd>
          {current.event.approximate_bytes !== undefined
            ? `≈${formatBytesFriendly(current.event.approximate_bytes)}`
            : "No numeric value"}
        </dd>
      </div>
      <div>
        <dt>
          {current.event.approximate_bytes !== undefined
            ? "Exact value"
            : "State"}
        </dt>
        <dd>
          {current.event.approximate_bytes !== undefined
            ? "Not representable exactly in this presentation"
            : unavailableReasonLabel(current.event.reason)}
        </dd>
      </div>
      {#if current.event.approximate_bytes !== undefined}
        <div>
          <dt>Source</dt>
          <dd>Operating system</dd>
        </div>
      {/if}
    </dl>
  {/if}

  {#if history.length > 0}
    <div class="available-memory-plot" aria-hidden="true">
      <svg viewBox="0 0 600 100" focusable="false">
        {#each history as item, index (item.observation_id)}
          {#if item.event.kind === "value"}
            <circle
              class="available-memory-point"
              cx={xPosition(index)}
              cy={yPosition(item.event.bytes)}
              r="7"
              vector-effect="non-scaling-stroke"
            />
          {:else}
            <rect
              class="available-memory-gap"
              x={xPosition(index) - 7}
              y="77"
              width="14"
              height="14"
              vector-effect="non-scaling-stroke"
            />
          {/if}
        {/each}
      </svg>
    </div>
    <div class="available-memory-legend" aria-label="Observation marker legend">
      <span
        ><span class="history-marker history-marker-value"></span> Numeric observation</span
      >
      <span
        ><span class="history-marker history-marker-gap"></span> No numeric value</span
      >
    </div>
    {#if numericValues.length === 1}
      <p class="available-memory-scale-note">
        One numeric value is shown; no range is implied.
      </p>
    {:else if valuesAreIdentical}
      <p class="available-memory-scale-note">
        Recorded numeric values were identical.
      </p>
    {:else if minimum !== null && maximum !== null}
      <p class="available-memory-scale-note">
        Scale maximum shown: {formatBytesFriendly(maximum)}. Scale minimum
        shown:
        {formatBytesFriendly(minimum)}.
      </p>
    {/if}

    <details class="available-memory-values">
      <summary>Observation values</summary>
      <ol>
        {#each history as item (item.observation_id)}
          <li>
            <span>Observation {item.observation_id}:</span>
            {#if item.event.kind === "value"}
              <span>
                {formatBytesFriendly(item.event.bytes)} —
                {formatExactBytes(item.event.bytes)}
              </span>
            {:else}
              {#if item.event.approximate_bytes !== undefined}
                <span>
                  ≈{formatBytesFriendly(item.event.approximate_bytes)} — Exact value
                  not representable in this presentation
                </span>
              {:else}
                <span
                  >No numeric value — {unavailableReasonLabel(
                    item.event.reason,
                  )}</span
                >
              {/if}
            {/if}
          </li>
        {/each}
      </ol>
    </details>
  {:else}
    <p class="available-memory-empty">No observations recorded this session.</p>
  {/if}
</figure>
