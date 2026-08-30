# Milestone 1W-C workflow-visual contract

## Status

Developer-approved on 2026-08-30 for one bounded visual-documentation
implementation.

## Purpose

Milestone 1W-C gives a first-time reader one accurate visual explanation of the
host-led workflow. The visual must make the passive `Refresh` path visibly
distinct from the optional, per-run-authorized observed-inference path without
presenting a development fixture as native-product evidence.

## Authorized work

This milestone authorizes only:

- this contract;
- one accessible, source-derived SVG workflow diagram under `docs/assets/`;
- embedding that same diagram in the README and user guide; and
- updating the documentation sequence in `docs/roadmap.md`.

## Visual provenance and accuracy boundaries

- The diagram is a conceptual workflow derived from the accepted documentation
  and implemented behavior at merged `main` revision
  `87403a079e6afbe8edd4beb83b8bb2f1105d741c`. It is not a native application
  screenshot and must be labelled accordingly.
- No current native executable was installed or available in the synchronized
  Windows development checkout when the visual work began. The accepted public
  preview had previously been removed after physical verification. This
  contract does not authorize reinstalling or rebuilding it solely to
  manufacture screenshot evidence.
- The visual contains no provider response, model identity, machine metric,
  account detail, or user data. It must not invent dashboard values or imply a
  particular provider state.
- The passive lane must show that a provider is started by the user, `Refresh`
  reacquires evidence without inference, and report copying is a separate
  explicit action.
- The observed-inference lane must show disclosure and authorization for one
  run before one fixed synthetic prompt and bounded attempt.
- The diagram represents an action and evidence flow, not the exact position,
  size, colour, or state of controls in a native application window.
- Missing evidence remains unavailable or unknown. The visual must not imply
  model fit, performance, root cause, repair, continuous monitoring, exact
  compute placement, provider management, persistence, or upload.
- Text alternatives and adjacent prose must preserve the meaning if SVG
  rendering is unavailable.

## Explicit exclusions

Milestone 1W-C does not authorize:

- a native or fixture screenshot, screen recording, product mockup, marketing
  image, logo, icon, or brand redesign;
- product, UI, provider, inference, source-code, dependency, workflow, package,
  signing, Store, release, or release-asset changes;
- installing dependencies or the public preview, building a new executable, or
  running live providers or inference;
- a new compatibility, security, privacy, performance, readiness, model-fit,
  root-cause, repair, or compute-placement claim; or
- non-specialist user validation, which remains the separately gated 1W-D
  decision.

## Acceptance criteria

The milestone is complete only when:

- one visual clearly separates passive observation from optional authorized
  inference;
- every action and boundary in the visual matches accepted product behavior;
- the SVG has a meaningful title, description, role, and readable text without
  depending on colour alone;
- light and dark colour-scheme presentation remains legible;
- the README and user guide identify it as a conceptual workflow rather than a
  screenshot;
- repository-readiness and SVG-structure checks pass; and
- the implementation diff is limited to this contract, one SVG asset, the
  README, the user guide, and the roadmap.

## Subsequent gate

Non-specialist validation and any resulting content, product, or readiness
change remain Milestone 1W-D and require separate developer authorization. This
contract does not authorize that work or any later signing or release action.
