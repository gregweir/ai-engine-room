# Milestone 1W-B guides and public-roadmap contract

## Status

Developer-approved on 2026-08-30 for one bounded documentation implementation.

## Purpose

Milestone 1W-B gives a non-specialist a safe path from the public preview to a
first interpretation of the application. It also turns the roadmap from an
internal chronology-first document into a public statement of what is
available, what is next, what is only being considered, and what is outside the
current product. The completed delivery record remains available in the same
roadmap.

## Authorized work

This milestone authorizes only:

- this contract;
- a concise README quick start;
- `docs/user-guide.md`;
- `docs/glossary.md`;
- a public-facing restructuring of `docs/roadmap.md` that preserves its
  completed delivery record; and
- links to the new guide and glossary from the README.

## Accuracy and safety boundaries

The documentation must preserve all of these boundaries:

- The only accepted public-preview binaries are the three assets published at
  [`v0.1.0-preview.1`](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1):
  - `AI.Engine.Room_0.1.0_amd64.deb`, 4,722,942 bytes, SHA-256
    `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e`;
  - `AI.Engine.Room_0.1.0_x64-setup.exe`, 2,651,735 bytes, SHA-256
    `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399`;
  - `SHA256SUMS.txt`, 198 bytes, SHA-256
    `2b1c787237ad10d9262f552438d9093cea44aee8c3c3ae035c4b564da2caf2de`.
- The preview is unsigned. A checksum detects changed bytes but does not
  authenticate a publisher. Documentation must not tell a user to disable or
  weaken a security control, install a trust certificate, or bypass an
  organizational policy. A user should continue only through a platform's
  normal option after verifying the official download and deciding that the
  unsigned preview is acceptable.
- `Refresh` passively reacquires the supported observations. It does not run or
  authorize inference.
- Provider access is limited to the implemented fixed numeric same-machine
  loopback endpoints. That scope is not a claim that the application makes no
  network requests, and a loopback API does not prove where compute executes.
- Bounded behavior is deliberately constrained behavior, not a sandbox and not
  a guarantee of local compute.
- An authorized LM Studio observation may cause LM Studio to JIT-load an
  unloaded model. AI Engine Room does not call model-management APIs.
- Session history is a bounded sequence of startup and explicit-refresh
  observations. It is not a continuous monitor or a time-based timeline.

## Explicit exclusions

Milestone 1W-B does not authorize:

- product, UI, provider, inference, source-code, dependency, workflow, package,
  signing, Store, release, or release-asset changes;
- screenshots or diagrams, which remain a separately gated 1W-C decision;
- user testing or an expansion of readiness claims, which remain a separately
  gated 1W-D decision;
- a signing application, certificate purchase, publisher-identity decision, or
  MSIX/Store work;
- a new provider, endpoint, export format, file write, or external destination;
  or
- a compatibility, security, privacy, performance, model-fit, root-cause,
  repair, or compute-placement claim not already established by accepted
  evidence.

## Acceptance criteria

The milestone is complete only when:

- a first-time reader can safely find the accepted preview, verify it, install
  it, launch it, use `Refresh`, and understand the five workspaces;
- the guide contains the exact published filenames, byte sizes, SHA-256 values,
  and platform verification commands;
- unsigned-code guidance states both the decision boundary and the limitation
  of checksums without recommending weakened controls;
- glossary terms align with the implemented behavior and accepted evidence;
- the roadmap visibly distinguishes present capability, next documentation
  work, uncommitted possibilities, and current non-goals;
- the completed delivery record remains in the roadmap; and
- the implementation diff is limited to this contract, the README, the user
  guide, the glossary, and the roadmap.

## Subsequent gates

Screenshots and diagrams (1W-C), non-specialist validation and readiness review
(1W-D), product or provider changes, signing, and any later release each require
their own developer authorization. No roadmap entry or completion of this
contract authorizes them.
