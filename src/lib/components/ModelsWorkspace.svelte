<script lang="ts">
  import type {
    LoadedModelSetView,
    ModelInventoryView,
    LmStudioSnapshotView,
    LlamaCppSnapshotView,
  } from "../types";
  import { formatBytes } from "../formatBytes";
  import LoadedModelsCard from "./LoadedModelsCard.svelte";
  import ModelInventoryCard from "./ModelInventoryCard.svelte";
  let {
    loadedModels,
    modelInventory,
    lmStudio,
    llamaCpp,
  }: {
    loadedModels: LoadedModelSetView | null;
    modelInventory: ModelInventoryView | null;
    lmStudio: LmStudioSnapshotView | null;
    llamaCpp: LlamaCppSnapshotView | null;
  } = $props();
</script>

<section class="workspace-content" aria-labelledby="models-heading">
  <h2 id="models-heading">Models</h2>
  <section aria-labelledby="loaded-models-heading">
    <h3 id="loaded-models-heading">Currently loaded</h3>
    <p class="workspace-intro">
      Ollama-reported loaded state is not proof of local execution.
    </p>
    {#if loadedModels}<LoadedModelsCard loaded={loadedModels} />{:else}<p>
        Loaded-model state is unavailable.
      </p>{/if}
  </section>
  <section aria-labelledby="lm-studio-models-heading">
    <h3 id="lm-studio-models-heading">LM Studio</h3>
    {#if lmStudio}
      <p class="workspace-intro">{lmStudio.interpretation}</p>
      <div class="explanation resource-qualification">
        <p>{lmStudio.resource_interpretation}</p>
        <p>{lmStudio.resource_qualification}</p>
      </div>
      {#if lmStudio.state === "available"}
        <ul class="model-list">
          {#each lmStudio.models as model (model.model_id)}
            <li class="model-entry">
              <details>
                <summary class="model-name">
                  {model.display_name} — {model.model_type}
                  {model.loaded_instances.length > 0
                    ? `— ${model.loaded_instances.length} loaded instance${model.loaded_instances.length === 1 ? "" : "s"}`
                    : "— not currently loaded"}
                </summary>
                <dl class="model-details">
                  {#if model.publisher}<dt>Publisher</dt>
                    <dd>{model.publisher}</dd>{/if}
                  {#if model.architecture}<dt>Architecture</dt>
                    <dd>{model.architecture}</dd>{/if}
                  {#if model.quantization}<dt>Quantization</dt>
                    <dd>{model.quantization}</dd>{/if}
                  {#if model.params_string}<dt>Parameters</dt>
                    <dd>{model.params_string}</dd>{/if}
                  <dt>Catalogue size reported by LM Studio</dt>
                  <dd>{model.size_bytes === null
                      ? "Not reported by LM Studio"
                      : formatBytes(model.size_bytes)}</dd>
                  <dt>Maximum context metadata reported by LM Studio</dt>
                  <dd>{model.max_context_length === null
                      ? "Not reported by LM Studio"
                      : model.max_context_length.toLocaleString()}</dd>
                  {#each model.loaded_instances as instance (instance.instance_id)}
                    <dt>Loaded instance</dt>
                    <dd>
                      {instance.instance_id} — configured context
                      {instance.context_length === null
                        ? "not reported by LM Studio"
                        : instance.context_length.toLocaleString()}
                    </dd>
                  {/each}
                </dl>
              </details>
            </li>
          {/each}
        </ul>
      {/if}
    {:else}<p>LM Studio state is unavailable.</p>{/if}
  </section>
  <section aria-labelledby="llama-cpp-model-heading">
    <h3 id="llama-cpp-model-heading">llama.cpp served model</h3>
    {#if llamaCpp}
      <p class="workspace-intro">{llamaCpp.interpretation}</p>
      {#if llamaCpp.state === "ready" && llamaCpp.served_model}
        <dl class="model-details">
          <dt>Served-model ID</dt>
          <dd>{llamaCpp.served_model.model_id}</dd>
          <dt>API scope</dt>
          <dd>Same-machine loopback</dd>
          <dt>Compute location</dt>
          <dd>Not independently verified</dd>
        </dl>
        <p>{llamaCpp.why_it_matters}</p>
      {:else}
        <p>{llamaCpp.state_label}</p>
        <p>{llamaCpp.why_it_matters}</p>
      {/if}
    {:else}<p>llama.cpp state is unavailable.</p>{/if}
  </section>
  <section aria-labelledby="catalogue-heading">
    <h3 id="catalogue-heading">Available catalogue</h3>
    <p class="workspace-intro">
      Catalogue presence is distinct from currently loaded state.
    </p>
    {#if modelInventory}<ModelInventoryCard
        inventory={modelInventory}
      />{:else}<p>Catalogue state is unavailable.</p>{/if}
  </section>
</section>
