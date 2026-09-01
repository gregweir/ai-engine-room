# Milestone 1Y-D Snap distribution disposition contract

## Status

Approved direction recorded for review on 2026-08-31. This is a
documentation-only closure proposal. It authorizes no Snapcraft invocation,
dependency retrieval, build, installation, account action, name registration,
upload, store contact, publication, or release.

## Purpose

Milestones 1Y-A through 1Y-C established that Snap distribution is feasible in
principle, produced a static strict-confinement proposal, and consumed exactly
one disposable expansion gate. That gate stopped before compilation because
Snapcraft's GNOME extension omitted the reviewed platform mapping and added
generated inputs, including an external Git source without an immutable
identity in the expanded definition.

This slice records the developer's decision not to spend further work on that
source boundary without evidence of user demand. It does not reinterpret the
stopped expansion as a packaging failure, predict Snap Store acceptance, or
claim that the generated source is unsafe.

## Authorized change if merged

Merging this contract would authorize only:

- this contract;
- one source-linked Snap disposition record;
- reconciliation of the consumed expansion record;
- a roadmap update that marks the current Snap channel deferred; and
- deterministic readiness assertions for the recorded decision.

The existing `snap/snapcraft.yaml` and desktop launcher remain historical,
non-release proposal material. Their presence does not authorize or imply a
build, supported package, store candidate, or future maintenance commitment.

## Decision

Defer Snap distribution for the current preview. The project will continue to
support only the already published direct `.deb` and NSIS preview downloads on
their exact verified baselines. It will not retry the consumed expansion,
broaden its allowlist, replace the GNOME extension, or prepare a Snap Store
submission under the current sequence.

This is a prioritization decision, not a permanent rejection of Snap. A future
reconsideration requires all of the following:

1. evidence of meaningful user or distribution demand;
2. a fresh developer-approved contract;
3. an explicit, reviewable source boundary for every generated or external
   build input;
4. a deliberate choice between pinned extension inputs and explicit
   least-privilege packaging;
5. renewed expansion, build, install, UI, provider, removal, licence-payload,
   and independent-review gates; and
6. a separate account, registration, upload, and publication decision.

No authority from Milestones 1Y-A through 1Y-C carries forward to those future
actions.

## Subsequent planning

With the current package-manager investigation closed, the next recommended
planning slice is the already identified documentation-led bounded
network-observability feasibility study. That study remains separately gated
and must not begin product implementation, packet inspection, firewall changes,
persistent monitoring, or elevated privileges in the main application.

## Explicit exclusions

This slice does not authorize:

- another Snapcraft expansion or build;
- fetching or pinning the generated `canonical/gpu-snap.git` source;
- modifying the static Snapcraft proposal or packaging payload;
- installing, launching, or removing a Snap;
- creating a Snap account, accepting terms, registering a name, contacting the
  store, uploading an artifact, or making an availability claim;
- changing the published `.deb`, NSIS installer, preview release, or website;
- provider access, inference, or product network-observability implementation;
  or
- PPA, official Debian/Ubuntu, Flatpak, or another distribution-channel work.

## Acceptance criteria

This proposal is ready for merge review only if:

- the change remains documentation and deterministic-policy only;
- the disposition cites the exact consumed expansion evidence;
- deferral is stated without predicting store acceptance or source safety;
- the existing Snap proposal is clearly historical and non-release;
- no later study or execution is described as approved; and
- deterministic repository checks pass.

## References

- [Milestone 1Y-C preparation contract](milestone-1y-c-snapcraft-one-build-preparation-contract.md)
- [Snapcraft expansion verification record](../distribution/snapcraft-expansion-verification-record.md)
- [Snap disposition](../distribution/snap-disposition.md)
- [Bounded network-observability planning note](bounded-network-observability-planning-note.md)
