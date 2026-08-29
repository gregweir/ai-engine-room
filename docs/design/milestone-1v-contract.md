# Milestone 1V contract

Status: closed on 2026-08-29. The developer-approved implementation and
verification remained bounded by this contract.

## Purpose

Milestone 1V would make already-acquired model-size and configured-context
evidence easier to understand without turning AI Engine Room into a memory
accounting, capacity-planning, or model-fit tool.

The current provider views do not expose measured KV-cache bytes, runtime
overhead, total physical VRAM, or authoritative compute placement. Milestone 1V
therefore must present those concepts as unavailable or unknown rather than
calculate or imply them.

## Evidence that may be presented

| Provider | Existing field | Permitted meaning |
| --- | --- | --- |
| Ollama | `reported_size_bytes` | Loaded size reported by Ollama; not exact RAM use, disk use, or model-weight allocation. |
| Ollama | `reported_vram_bytes` | VRAM size reported by Ollama; not an independent measurement of physical VRAM use or capacity. |
| Ollama | `reported_context_length` | Configured context length reported for the running model; not a theoretical maximum or performance result. |
| LM Studio | `loaded_instances[].context_length` | Configured context length reported for that loaded instance. |
| LM Studio | `size_bytes` | Catalogue/model size reported by LM Studio; not loaded memory use. |
| LM Studio | `max_context_length` | Provider-reported model metadata; distinct from a loaded instance's configured context. |
| Operating system | existing total and available memory observations | Whole-system observations with the existing platform qualifications; not per-model attribution. |

Missing, invalid, or unavailable provider values remain explicitly unavailable.
Ollama and LM Studio values retain their provider identity and are not normalized
into a shared memory model.

## Required presentation

- Use a text-first, read-only presentation within the existing Observe resource
  context and provider surfaces.
- Label every displayed size or context value with its source and qualification.
- Distinguish model or loaded size, configured context, KV cache, runtime
  overhead, system memory, and VRAM.
- State that KV-cache bytes and runtime overhead are not separately reported by
  the current evidence.
- State that compute placement remains unknown except for the narrow fact that
  Ollama supplied its `reported_vram_bytes` value.
- Keep the existing qualitative statement that a larger configured context can
  require more memory, without converting context length to bytes.
- Preserve semantic HTML, keyboard access, enlarged-text behavior, visible
  unavailable states, and non-colour evidence labels.
- Author interpretations and qualifications in Rust-controlled view data; the
  frontend renders them without inventing relationships.

## Prohibited claims and calculations

Milestone 1V must not:

- calculate or estimate KV-cache bytes, runtime overhead, model RAM use, memory
  pressure, or physical VRAM capacity;
- sum provider-reported model sizes or VRAM sizes;
- subtract any model value from total or available memory;
- derive ratios, percentages, remaining memory, free memory, fit, headroom,
  health, safety, performance, or recommendations;
- treat catalogue size, loaded size, model weights, RAM use, VRAM use, and disk
  use as interchangeable;
- infer CPU, GPU, split, or offloaded placement;
- place system-memory and provider-reported values on a common graph or scale;
- add automatic refresh, monitoring, history, inference, provider control, or a
  new acquisition endpoint;
- add this evidence to Diagnose or the report-safe projection without a separate
  contract and developer decision.

## Acquisition and authority boundary

Implementation may compose only from the views already acquired by the current
manual Refresh flow. It must not make new provider, operating-system, network,
filesystem, process, or inference calls. No provider may be started, stopped, or
reconfigured.

This milestone does not authorize dependency installation, packaging, signing,
publication, distribution, release work, broader provider coverage, or macOS
work.

## Implementation gate

Implementation may begin only after the developer explicitly approves this
contract. The implementation plan must remain bounded to:

1. Rust view composition and controlled explanatory text;
2. matching TypeScript DTOs and text-first Svelte presentation;
3. deterministic artificial fixtures and tests; and
4. documentation needed to keep the evidence boundary explicit.

No live provider or inference run is required for implementation verification.

## Acceptance criteria

- Deterministic Rust tests cover every available, missing, invalid, and
  unavailable presentation state introduced by the milestone.
- Frontend tests prove provider/source labels, qualifications, semantic
  structure, accessible unavailable states, and enlarged-text-safe layout.
- Tests reject the prohibited arithmetic and fit/headroom vocabulary in all
  controlled presentation variants.
- Existing report, Diagnose, acquisition, and manual-refresh boundaries remain
  unchanged.
- Repository readiness, frontend type checks/tests/build, Rust formatting,
  Clippy, and Rust tests pass on the implementation branch using available
  toolchains.
- After local verification, a bounded native Linux verification and an
  independent Windows review are separately authorized and recorded before any
  packaging decision.

## Closure evidence

- Approved contract commit: `0e5824fdfa6ab211895fa65311e9e446e410a601`.
- Bounded implementation commit: `f8632ec47dc0613ae01ef2d70e373b80de41753b`.
- Deterministic-check correction commit:
  `90d853b6a6072d64379ef267516959505db50ec7`.
- Cross-platform repository-readiness correction and accepted source revision:
  `1f19ea0f249a3b07a55d6fb53a70d0c45cdb7e51`, tree
  `38898fe41808e596170d2bd0e2b0bed91286b853`.
- Deterministic GitHub checks passed for the accepted source revision on the
  frontend contract job, Ubuntu Rust job, and Windows Rust job.
- Bounded native verification passed on the verified Ubuntu host: repository
  readiness, frontend checks, lint, 286 deterministic frontend tests, frontend
  build, Rust format, workspace tests, and strict Clippy.
- Independent read-only review on the verified Windows host passed with no
  findings. The prior CRLF `NOTICE` readiness defect was corrected without
  changing the approved notice content.

No provider, inference, packaging, signing, upload, publication, distribution,
release, or private-evidence operation was part of closure.
