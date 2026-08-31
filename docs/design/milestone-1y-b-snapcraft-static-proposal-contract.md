# Milestone 1Y-B static Snapcraft proposal contract

## Status

Developer-authorized on 2026-08-31 for one static packaging proposal. This
contract permits reviewable repository files only. It does not authorize a
Snapcraft installation, package build, Snap installation, provider access,
account action, name registration, upload, review request, or release.

## Purpose

Milestone 1Y-A found the Snap Store feasible in principle but left strict
confinement unproved. This slice turns that assessment into the smallest
inspectable packaging hypothesis so the project can review inputs, metadata,
interfaces, and stop conditions before spending a build.

## Authorized change

This slice may add only:

- this contract and one static-proposal record;
- one `grade: devel`, `confinement: strict`, amd64-only Snapcraft definition;
- a desktop launcher using existing approved product artwork;
- deterministic static assertions for the proposal;
- ignore rules for local Snapcraft output; and
- a roadmap update.

The proposal may refer to current official Snapcraft documentation and tracked
source. It must remain visibly unbuilt and unverified.

## Packaging hypothesis

The definition must:

1. build current source rather than wrap or silently republish the existing
   public Debian artifact;
2. use the tracked `0.1.0`, Apache-2.0, product URL, source URL, developer
   contact, reverse-DNS application ID, and existing icon;
3. use the core24 base, strict confinement, devel grade, and amd64 only;
4. use the official GNOME extension as a provisional WebKitGTK/GTK desktop
   runtime and add only the app-specific `network` client plug explicitly;
5. use the Rust plugin for `src-tauri` and Node 24 only at build time for the
   locked frontend build;
6. install the first-party licence, notice, third-party licence, and source
   inventory in the proposed payload; and
7. provide a desktop launcher that invokes the Snap app name and points to the
   packaged icon.

The GNOME extension's generated interface set is part of the hypothesis, not
an accepted least-privilege conclusion. Current official documentation says it
adds `desktop`, `desktop-legacy`, `gsettings`, `opengl`, `wayland`, `x11`,
`mount-observe`, and `calendar-service`, plus content interfaces. A later build
gate must expand and inspect the definition, with particular scrutiny of
interfaces unrelated to the implemented product behavior.

## Unproved assumptions

Static review cannot establish that:

- the declared Rust channel satisfies the complete locked dependency graph;
- the GNOME SDK and listed build packages compile the current Tauri source;
- the produced command path, WebKit subprocesses, graphics, fonts, portals,
  clipboard, or desktop integration work;
- the `network` interface permits all three fixed numeric loopback clients with
  the intended semantics;
- `/proc/meminfo` remains readable and retains the app's intended whole-system
  `MemAvailable` and `MemTotal` meaning;
- the generated payload contains only the expected runtime and licence files;
  or
- install, launch, close, removal, absence, refresh, or store review passes.

None of these assumptions becomes evidence when this proposal is merged.

## Required next gate

A later build requires a new exact developer approval naming the source commit
and the proposed build procedure. That gate must:

1. use one declared disposable or designated Ubuntu 24.04 LTS amd64
   environment;
2. inspect `snapcraft expand-extensions` before building and stop if the
   generated permissions or sources exceed the reviewed boundary;
3. record Snapcraft, snapd, core, GNOME SDK/platform, Node build-snap, Rust
   toolchain, and resolved package versions or revisions;
4. perform at most one clean build with no upload or account action;
5. hash and independently inspect the resulting local artifact and licence
   payload; and
6. stop before installation, provider access, inference, registration, or
   publication unless those actions receive their own later approval.

If the static definition is invalid or the build fails, the result is a
bounded packaging finding. It does not authorize an ad hoc retry, classic
confinement, broader interfaces, dependency installation on a physical host,
or a store action.

## Explicit exclusions

This slice does not authorize:

- installing or invoking Snapcraft, snapd, Multipass, LXD, or build tooling;
- expanding extensions, downloading build snaps, resolving package archives,
  compiling source, or producing a `.snap` artifact;
- installing, launching, refreshing, removing, or inspecting a built Snap;
- provider requests, inference, network capture, or product network
  observability;
- checking or registering the tentative name, creating or using an account,
  accepting terms, contacting Canonical, requesting review, or uploading;
- changing the Debian/NSIS packages, current public preview, product behavior,
  Tauri bundle targets, or deterministic CI workflow; or
- claiming compatibility, confinement success, store acceptance, signing,
  publication, or release readiness.

## Acceptance criteria

The proposal is ready for merge review only if:

- every changed path fits the authorized static scope;
- static assertions preserve strict confinement, devel grade, amd64 scope,
  fixed identity, fixed command, licence payload, and explicit exclusions;
- the launcher contains no shell, path, provider, or network action;
- documentation labels every runtime and build conclusion unproved;
- no generated Snapcraft output or artifact exists; and
- deterministic repository checks pass.

## References

- [Snap Store feasibility assessment](../distribution/snap-store-feasibility-assessment.md)
- [Snapcraft project-file schema](https://snapcraft.io/docs/snapcraft-yaml-schema/)
- [GNOME extension](https://documentation.ubuntu.com/snapcraft/latest/reference/extensions/gnome-extension/)
- [Rust plugin](https://documentation.ubuntu.com/snapcraft/stable/common/craft-parts/reference/plugins/rust_plugin/)
- [Snap desktop-file format](https://snapcraft.io/docs/reference/development/yaml-schemas/the-snap-format/)
