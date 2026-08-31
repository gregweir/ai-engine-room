# Milestone 1Y-A Snap Store feasibility contract

## Status

Developer-authorized on 2026-08-31 for one documentation-only feasibility
study. This contract authorizes no Snapcraft project file, account action,
name registration, build, installation, upload, review request, release, or
external communication.

## Purpose

The current public preview is available as a directly downloaded Debian
package, while the exact Windows candidate has been deferred from WinGet. This
slice asks whether the Snap Store could provide a proportionate future Linux
package-manager route without requiring paid code signing or weakening AI
Engine Room's evidence boundaries.

The answer must distinguish what current official documentation establishes
from what only a strict-confinement prototype can establish.

## Authorized change

This slice permits only:

- this contract;
- one source-linked Snap Store feasibility assessment; and
- a roadmap update that records the decision and the next separately gated
  step.

Public Snapcraft documentation and tracked source may be examined read-only.
No external state may be changed.

## Required questions

The assessment must address:

1. the documented account, name-registration, review, channel, and update
   model;
2. whether the normal publication flow identifies a certificate purchase or
   submission fee, while avoiding an unsupported universal cost guarantee;
3. the difference between store-signed assertions and a developer-purchased
   platform code-signing certificate;
4. the separate packaging and maintenance work implied by the absence of a
   native Snap target in the current Tauri bundle configuration;
5. strict confinement as the required starting point, with no classic request
   unless a later evidence record proves strict confinement insufficient;
6. the interfaces and evidence needed for the desktop UI, graphics, clipboard,
   numeric loopback provider clients, and Linux memory observation;
7. automatic Snap refresh behavior and its user-facing difference from the
   application's current lack of an application-managed updater;
8. architecture, publisher, licence, provenance, install, removal, residue,
   and independent-review boundaries; and
9. the exact approval gates required before account, build, test, or publication
   activity.

## Decision boundary

The permitted conclusion is a feasibility disposition, not package readiness
or store acceptance. A positive disposition may recommend one later static
packaging prototype and one later bounded strict-confinement test sequence,
but neither is authorized here.

The assessment must not check or claim availability of a proposed Snap name.
It must not imply that a free account makes ongoing packaging, maintenance,
support, or build work cost-free.

## Explicit exclusions

This slice does not authorize:

- creating or modifying `snapcraft.yaml`, AppStream metadata, desktop launchers,
  package targets, build scripts, or product code;
- installing Snapcraft, Multipass, LXD, snapd, a Snap, or any dependency;
- building, signing, hashing, uploading, registering, publishing, refreshing,
  or removing a Snap;
- creating or using a developer account, accepting terms, checking or reserving
  a name, requesting review, contacting Canonical, or changing store settings;
- provider access, inference, packet inspection, network-observability work, or
  a physical/disposable-machine run;
- changing the current Debian package or public preview; or
- claiming Snap Store availability, acceptance, privacy, safety, or release
  readiness.

## Acceptance criteria

The study is ready for merge review only if:

- the diff is documentation-only and limited to this contract, its assessment,
  and the roadmap;
- current official sources support statements about publishing, assertions,
  confinement, interfaces, and updates;
- tracked source supports statements about the current package target and
  runtime evidence paths;
- observed facts, design inferences, and untested assumptions remain distinct;
- later implementation and external actions remain separately gated; and
- deterministic repository checks pass.

## References

- [Snap publishing process](https://documentation.ubuntu.com/snapcraft/stable/explanation/snap-publishing-process/)
- [Register a snap](https://snapcraft.io/docs/registering-your-app-name/)
- [Assertions](https://snapcraft.io/docs/explanation/security/assertions/)
- [Snap confinement](https://snapcraft.io/docs/explanation/security/snap-confinement/)
- [Network interface](https://snapcraft.io/docs/reference/interfaces/network-interface/)
- [Manage updates](https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/)
- [WinGet disposition](../distribution/winget-disposition.md)
