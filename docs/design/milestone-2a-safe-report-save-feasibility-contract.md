# Milestone 2A safe report-save feasibility contract

## Status

Prepared locally on 2026-09-01 under developer authorization from exact public
baseline `54cc29f6afaee714606a0586c672d060a3e583c8` (tree
`2cce41f1d6a49a0d5f238f6a5bee69e5562225cc`). This preparation authorizes only
this documentation file and one local commit for separate review. It does not
authorize the feasibility study, product implementation, a product file write,
push, packaging, publication, or release.

## Purpose

AI Engine Room currently shows one sanitized, allow-listed plain-text report
preview and can copy exactly that preview to the system clipboard after an
explicit user action. It does not save, upload, send, persist, or share the
report.

A user-initiated plain-text save could make the existing **Report** step more
useful to non-specialists while avoiding the clipboard. It would also create a
new durable-data and filesystem boundary. Before implementation is considered,
a documentation-only feasibility study must determine whether the feature can
preserve the existing report-safe projection, passive posture, and
plain-language privacy model without adding disproportionate platform,
dependency, permission, security, or support cost.

## Current authorization

The current authorization permits only preparation and local review of this
contract. It does not authorize the proposed study described below. No other
file may change under this preparation slice.

## Proposed documentation-only study

If separately approved, the feasibility study may add only:

- one source-linked feasibility assessment;
- narrowly necessary architecture and user-guide reconciliations;
- a roadmap disposition that records adoption, further preparation, or
  deferral; and
- deterministic readiness assertions for those documents.

The study may inspect tracked source and current primary Tauri and operating-
system documentation. It must not add or update a dependency, invoke a native
file dialog, write or delete a file, run a provider or inference request,
package the application, or alter external state.

## Required questions

The study must answer, without selecting an implementation prematurely:

1. whether saving the existing preview addresses a concrete non-specialist
   need that clipboard copy does not meet;
2. whether the saved payload can remain exactly the already displayed,
   sanitized report text, with no new report field, format, metadata, timestamp,
   machine detail, or internal state;
3. which encoding, byte-order-mark, line-ending, `.txt` extension, media-type,
   and default-filename rules are required for a stable cross-platform file;
4. whether the existing Tauri capability boundary can support an explicit
   native save dialog and one bounded write without broad filesystem access;
5. what additional dependency, capability, permission, audit, licence, and
   package-payload consequences each credible implementation option would have;
6. how cancellation, an unavailable dialog, invalid or inaccessible paths,
   existing destinations, overwrite decisions, partial writes, disk-full
   conditions, and other failures would be represented without exposing raw
   paths or operating-system errors;
7. whether a temporary-file and atomic-replace strategy is feasible and how it
   would avoid leaving an unintended partial report or silently replacing an
   existing file;
8. how the interface would explain persistence before saving, remain fully
   keyboard-operable, retain visible focus, and provide non-colour success,
   cancellation, and failure feedback at normal and enlarged text sizes;
9. which deterministic frontend, Rust, capability, privacy, and package-
   licence tests could establish the proposed boundary without performing a
   real file write in ordinary CI; and
10. which later physical Windows and Ubuntu checks would be necessary for an
    exact implementation candidate, each behind a separate authorization.

## Required behavior boundary

Any later proposal must preserve all of these conditions:

- saving is initiated by one explicit user action from the visible **Report**
  workspace;
- the application shows the complete report-safe preview before that action;
- the saved payload is derived only from the existing allow-listed report
  projection and defence-in-depth sanitizer;
- choosing **Cancel** changes no file and is reported as cancellation rather
  than failure;
- an existing destination is never replaced without the platform's explicit
  user-facing overwrite decision;
- no report or chosen path is retained in application history, settings,
  diagnostics, logs, or another report;
- no automatic save, default-directory scan, background write, retry, upload,
  send, share, cloud integration, or application persistence is introduced;
  and
- clipboard copy remains a separate explicit action and is not silently
  combined with saving.

## Evidence and privacy boundaries

The study and any later proposal must state plainly that:

- `report-safe` means the payload is deliberately minimized; it does not mean
  anonymous, confidential, encrypted, authenticated, or safe to publish;
- a selected path or filename can itself contain identifying information and
  therefore must not enter retained application evidence;
- a locally selected destination may be synchronized, backed up, indexed, or
  read by other software outside AI Engine Room's control;
- a successful write establishes only that the operating system accepted the
  bounded operation, not who can later access the file;
- saving the visible report does not expand or strengthen the evidence in that
  report; and
- plain text provides neither publisher authentication nor tamper detection.

## Explicit exclusions

This contract does not authorize:

- product, UI, Rust, TypeScript, capability, configuration, dependency, test,
  package, workflow, or readiness-script changes;
- opening a native file picker or creating, overwriting, reading back, moving,
  deleting, or inspecting a report file;
- Markdown, HTML, JSON, PDF, archive, image, structured, encrypted, signed, or
  multi-file export;
- automatic filenames containing a username, hostname, provider, model,
  machine identifier, precise observation value, or hidden timestamp;
- a recent-files list, remembered destination, session persistence, report
  history, telemetry, analytics, upload, email, cloud storage, or Share action;
- provider access, inference, network observation, network access, elevation,
  helper installation, or broader filesystem permission;
- physical-machine validation or a claim of Windows, Ubuntu, packaging,
  accessibility, privacy, security, or release readiness; or
- push, pull request, merge, package, publication, release, signing, or
  distribution work.

## Adopt, prepare, or defer threshold

The documentation study may recommend a later implementation-preparation
contract only if it identifies a narrow cross-platform design that saves the
already visible plain text, obtains a user-selected destination, adds no broad
filesystem authority, fails without leaking paths or leaving partial files,
and has proportionate dependency, maintenance, privacy, and support costs.

It must recommend deferral if the useful design requires broad filesystem
access, expands the report schema or format, silently overwrites or persists
data, cannot provide bounded failure handling, or lacks a concrete advantage
over the existing clipboard action. An inconclusive result creates no
implementation authority.

## Acceptance criteria for this preparation

This contract is ready for local review only if:

- the exact baseline identity is recorded;
- the current authorization and every later gate are unambiguous;
- the existing report-safe and clipboard boundaries remain unchanged;
- durable-data, path, overwrite, partial-write, and external-sync risks are
  explicit;
- no implementation option is represented as selected or validated;
- the diff contains only this contract; and
- documentation formatting and diff checks pass without invoking product
  checks, providers, inference, packaging, or native file operations.

## Subsequent gates

After separate approval and push of this contract, execution of the proposed
documentation-only study requires a new exact developer authorization. Any
product implementation would then require its own source-bound contract and
local-commit review. Native Windows and Ubuntu file-save validation, package
inspection, release work, and publication each remain later independent gates.

## References

- [Architecture: report privacy](architecture.md#8-report-privacy-architecture)
- [User guide: reports and the clipboard](../user-guide.md#reports-and-the-clipboard)
- [Roadmap](../roadmap.md)
