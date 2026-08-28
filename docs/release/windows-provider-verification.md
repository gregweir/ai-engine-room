# Windows provider verification

This document records bounded owner verification of AI Engine Room's existing Ollama and LM Studio integrations on Windows. It is evidence for the tested configuration, not a broad Windows or provider compatibility claim.

## Verification baseline

- Date: 2026-08-20
- Operating system: Windows 11 25H2, build 26200.7462, x86-64 / AMD64
- Repository lineage: `182b4b7cf01fd5bfb25c4c0cdf92e36721dbe5b9` (the committed Milestone 1N verification plan), with application and package implementation unchanged from accepted Milestone 1M commit `22917627dc3d4551b93e46d5adeeae1bc9be1804`
- Scope: passive provider detection, inventory/state presentation, coexistence, and bounded UI checks only

All inference-enabling environment variables for the existing Ollama and LM Studio ignored live tests were confirmed unset. No ignored live test or inference request ran.

## Ollama evidence

- Ollama version: `0.32.15`
- Fixed loopback API: `127.0.0.1:11434`
- Passive `GET /api/version` was reachable and reported `0.32.15`.
- Passive `GET /api/tags` reported a catalogue count of 1.
- Passive `GET /api/ps` reported a loaded-model count of 0.
- AI Engine Room's displayed catalogue and loaded state matched this bounded ground truth.
- No `/api/generate` request or model-management action occurred.

Ollama's existing execution-location qualification remains unchanged. Loopback reachability and passive provider operation do not establish compute placement.

## LM Studio evidence

- The LM Studio CLI was present.
- The LM Studio server was initially stopped. The owner manually started it through the normal LM Studio CLI; AI Engine Room did not start it.
- Fixed loopback native REST v1 API: `127.0.0.1:1234`
- After the manual start, passive `GET /api/v1/models` verification passed.
- The planned bounded catalogue, model-type eligibility, and loaded/unloaded-state checks passed. Exact model identifiers and private inventory details were not retained.
- AI Engine Room did not call load, unload, download, delete, configuration, or other model-management APIs.

For LM Studio, `api_scope = same_machine_loopback` and `compute_location = not_independently_verified`. This verification did not run inference or establish CPU, GPU, or other compute placement.

## Coexistence and passive behavior

- AI Engine Room displayed Ollama and LM Studio simultaneously where planned.
- Neither provider suppressed or took precedence over the other; provider/model grouping remained distinct.
- Launch, navigation, model selection, and one explicit **Refresh** remained passive.
- No inference observation or provider-generated output appeared.
- AI Engine Room did not start or stop a provider service and did not download, load, unload, or delete a model.
- LM Studio loaded-instance state and Ollama loaded state remained stable across AI Engine Room's passive actions.
- The provider UI remained readable and usable within the limited Milestone 1N graphical/accessibility scope.

## Preserved boundaries

- Windows Available memory remained controlled unavailable; Resource Context did not invent Windows memory data.
- Report contained no provider, model, or loaded-state data.
- **Copy report** was not used and the clipboard remained untouched.
- Browser/mock behavior, permissions, endpoints, and application runtime behavior were unchanged.

## Limitations and items not tested

- No inference ran, so Windows inference and exact compute placement remain unverified.
- No claim is made for all Ollama or LM Studio models, provider authentication, custom ports, LAN/remote access, or provider configurations beyond the fixed tested loopback APIs.
- No claim is made for other Windows versions, builds, architectures, or machines.
- Windows RAM, CPU, and GPU telemetry remains unavailable.
- This verification does not establish production readiness, release-candidate status, or broad Windows support.
