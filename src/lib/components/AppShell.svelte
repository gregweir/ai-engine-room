<script lang="ts">
  import type { Snippet } from "svelte";
  import MockBanner from "./MockBanner.svelte";

  export type Workspace =
    "overview" | "models" | "observed" | "diagnose" | "report";

  let {
    activeWorkspace = $bindable(),
    isMock,
    onRefresh,
    refreshing,
    children,
  }: {
    activeWorkspace: Workspace;
    isMock: boolean;
    onRefresh: () => void;
    refreshing: boolean;
    children: Snippet;
  } = $props();

  const workspaces: { id: Workspace; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "models", label: "Models" },
    { id: "observed", label: "Observed inference" },
    { id: "diagnose", label: "Diagnose" },
    { id: "report", label: "Report" },
  ];
  let tabElements = $state<HTMLButtonElement[]>([]);

  function focusTab(index: number) {
    tabElements[index]?.focus();
  }

  function handleKeydown(event: KeyboardEvent, index: number) {
    let next: number | null = null;
    if (event.key === "ArrowRight") next = (index + 1) % workspaces.length;
    if (event.key === "ArrowLeft")
      next = (index - 1 + workspaces.length) % workspaces.length;
    if (event.key === "Home") next = 0;
    if (event.key === "End") next = workspaces.length - 1;
    if (next !== null) {
      event.preventDefault();
      focusTab(next);
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      activeWorkspace = workspaces[index]!.id;
    }
  }
</script>

<header class="app-header">
  <div class="app-heading-row">
    <div>
      <h1>AI Engine Room</h1>
      <p class="intro">
        A plain view of local AI system metrics and observations.
      </p>
    </div>
    <button
      type="button"
      class="refresh"
      onclick={onRefresh}
      disabled={refreshing}
      aria-busy={refreshing}>{refreshing ? "Refreshing…" : "Refresh"}</button
    >
  </div>
  {#if isMock}<MockBanner />{/if}
  <nav class="workspace-nav" aria-label="Primary workspaces">
    <div role="tablist">
      {#each workspaces as workspace, index (workspace.id)}
        <button
          bind:this={tabElements[index]}
          type="button"
          role="tab"
          id={`workspace-tab-${workspace.id}`}
          aria-controls={`workspace-panel-${workspace.id}`}
          aria-selected={activeWorkspace === workspace.id}
          tabindex={activeWorkspace === workspace.id ? 0 : -1}
          onclick={() => (activeWorkspace = workspace.id)}
          onkeydown={(event) => handleKeydown(event, index)}
          >{workspace.label}</button
        >
      {/each}
    </div>
  </nav>
</header>

<main>{@render children()}</main>
