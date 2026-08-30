# Milestone 1W-A public-onboarding contract

Status: approved by the developer on 2026-08-30 for one bounded
documentation implementation.

## Purpose

Milestone 1W-A makes AI Engine Room's purpose, intended audience, and safety
posture understandable before a reader reaches the detailed compatibility and
verification record.

The current README accurately records a large amount of bounded evidence, but
its opening leads with a dense capability inventory. A non-specialist can miss
the product's value and mistake deliberate limitations for omissions. This
milestone changes the documentation hierarchy, not the product boundary.

## Authorized documentation change

Implementation is limited to:

1. adding this contract;
2. replacing the README's opening capability sentence with one concise purpose
   statement;
3. adding a short explanation of the questions the application helps answer;
4. presenting the current safety boundaries as product features; and
5. retaining the existing detailed capability, support, build, privacy, and
   project-document sections below the new front door.

The opening must continue to identify Greg Weir as the developer,
Tartanleaf.com Inc. as the publisher, the Apache-2.0 licence, the 2026
copyright, and the pre-release product state.

## Required accuracy boundaries

- **Refresh** must remain distinct from observed inference. Refresh does not
  authorize or run inference.
- Observed inference must be described as one explicitly authorized fixed
  synthetic prompt with existing timeout, concurrency, and no-retry bounds.
- The application may be described as using fixed same-machine loopback
  provider APIs. It must not be described as making no network calls.
- The inference path may be described as bounded. It must not be described as
  sandboxed.
- Ollama execution location and exact LM Studio compute placement remain
  unverified. The opening must not guarantee local compute placement.
- The application does not call model-management or service-control APIs.
  Documentation must preserve the existing qualification that an authorized LM
  Studio observation may cause LM Studio itself to JIT-load a model.
- Session observations remain explicitly acquired, bounded, in-memory, and
  non-temporal. They are not continuous monitoring or a timeline.
- Safety language must not imply certification, containment, or protection
  against a compromised local provider.

## Explicit exclusions

Milestone 1W-A does not authorize:

- application, UI, runtime, provider, inference, report, or data-model changes;
- quick-start, glossary, screenshot, diagram, comparison-table, or roadmap
  implementation, which remain later 1W chunks;
- a new provider, endpoint, port, acquisition path, export format, file-write
  capability, network destination, or dependency;
- signing, a signing application, account creation, certificate purchase,
  Microsoft Store or MSIX work, packaging, upload, asset replacement,
  publication, or another release;
- a stable, production-ready, broadly compatible, security-certified,
  privacy-certified, benchmark, model-fit, root-cause, or compute-placement
  claim; or
- alteration of historical build, acceptance, publication, or release evidence.

## Acceptance criteria

- A reader encounters the product purpose before the detailed capability list.
- The new opening answers what the application is for, what questions it helps
  answer, when inference can occur, and which actions remain outside the
  product boundary.
- The wording uses plain language while preserving the existing evidence and
  qualification model.
- Existing detailed README content and public-preview identity remain intact.
- No source, product behavior, release artifact, signing state, dependency, or
  workflow changes.
- Repository readiness and deterministic documentation-sensitive checks pass.
- The final diff is limited to this contract and the README front door.

## Subsequent gates

Separate developer approval remains required for:

1. 1W-B quick start, user guide, glossary, and public-roadmap restructuring;
2. 1W-C privacy-reviewed screenshots and a safety/data-flow diagram;
3. 1W-D non-specialist validation and additional readiness assertions;
4. any product or provider change prompted by later user evidence; and
5. any signing application, paid service, Store channel, packaging format, or
   release change.
