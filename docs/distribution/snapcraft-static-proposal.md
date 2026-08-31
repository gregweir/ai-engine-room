# Static Snapcraft packaging proposal

Status: **STATIC PROPOSAL ONLY; UNBUILT; NO ACCOUNT OR STORE ACTION
AUTHORIZED** on 2026-08-31.

## Outcome

The repository now contains a narrow, reviewable hypothesis for packaging AI
Engine Room as an amd64 Snap. It deliberately uses strict confinement and a
development grade. It does not change the current Debian package, Windows
installer, public preview, application code, provider behavior, or Tauri bundle
targets.

The proposal is not evidence that Snapcraft accepts the file, that the source
builds, that the app runs under confinement, or that the Snap Store would
accept it.

## Proposed identity and metadata

| Field | Static proposal |
| --- | --- |
| Tentative Snap name | `ai-engine-room` (not checked or registered) |
| Title | `AI Engine Room` |
| Version | `0.1.0` |
| Application ID | `com.tartanleaf.aiengineroom` |
| Developer | Greg Weir |
| Intended publisher | Tartanleaf.com Inc. |
| Licence | Apache-2.0 |
| Base | `core24` |
| Architecture | amd64 only |
| Grade | `devel` |
| Confinement | `strict` |
| Store state | none |

The summary and description use plain language suitable for non-technical
readers and state that the definition is an unbuilt proposal. The website,
source, issue, and developer-contact links point to existing public project
routes. The launcher uses the Snap app name directly and reuses the approved
product icon already tracked in the repository.

The tentative name remains an internal proposal value. Its availability has
not been checked, and merging it does not reserve it.

## Source-build proposal

The definition proposes a source build rather than wrapping the already
published Debian file:

- the Rust plugin targets the real `src-tauri` package rather than the virtual
  workspace root;
- Node 24 is a build-only Snap input, matching the deterministic CI major
  version, and `npm ci` consumes the committed lockfile;
- the frontend build runs before the Rust plugin builds the Tauri binary;
- the tracked `rust-version` value, `1.77.2`, is the provisional Rust channel;
  and
- the proposed payload explicitly installs `LICENSE`, `NOTICE`,
  `THIRD-PARTY-LICENSES.txt`, and `THIRD-PARTY-SOURCES.txt`.

The Node channel and Rust toolchain are inputs to verify, not reproducibility
claims. A later build record must capture their exact resolved revisions and
versions. If the locked dependencies require a newer Rust toolchain, the build
must stop and record that fact rather than silently broadening the proposal.

## Desktop and interface proposal

The app is a GTK/WebKitGTK desktop application, so the definition uses the
official core24-compatible GNOME extension. Official documentation says that
the extension supplies the GNOME platform and GPU content snaps, WebKitGTK
layouts, desktop runtime variables, and common desktop interfaces.

The only app-specific plug added by this proposal is `network`, because all
implemented provider adapters are outbound clients to fixed numeric loopback
endpoints. The definition does not add `network-bind`, `home`, removable media,
system observation, process control, firewall, or privileged hardware
interfaces.

However, the GNOME extension itself currently expands to include interfaces
such as `mount-observe` and `calendar-service` that AI Engine Room has not
justified. A later build gate must inspect the exact expansion and decide
whether the extension remains proportionate. This static proposal does not
treat extension-generated access as automatically acceptable and does not
authorize a classic-confinement fallback.

## Static safeguards

Repository readiness now checks that the proposal:

- remains `base: core24`, `grade: devel`, `confinement: strict`, and amd64-only;
- retains the tentative name, application ID, command, GNOME extension, and
  explicit `network` plug;
- does not declare classic/devmode confinement or broad app-specific plugs;
- retains the required licence and notice payload commands; and
- keeps the launcher non-terminal and directly bound to `ai-engine-room`.

The ignore rules exclude ordinary local Snapcraft work directories and
`*.snap` output from accidental source control. No such output exists in this
slice.

## Build-gate recommendation

The next recommended step is one separately approved **expansion and local
build gate**, not installation or publication. It should name the exact source
commit and perform these bounded stages:

1. verify a clean Ubuntu 24.04 LTS amd64 environment and record tool versions;
2. expand the GNOME extension and review every generated build snap, content
   snap, interface, environment rule, and layout;
3. stop if the expansion exceeds the approved access or source boundary;
4. perform one clean build without a Snap account, upload, or registration;
5. record the artifact filename, bytes, SHA-256, internal metadata, file list,
   dynamic-library resolution, and licence payload; and
6. stop before installing or launching the artifact.

Strict-confinement installation and runtime acceptance would remain a later,
independently approved gate. Provider access and inference remain later still.

## References

- [`snap/snapcraft.yaml`](../../snap/snapcraft.yaml)
- [`snap/gui/ai-engine-room.desktop`](../../snap/gui/ai-engine-room.desktop)
- [Milestone 1Y-B contract](../design/milestone-1y-b-snapcraft-static-proposal-contract.md)
- [Milestone 1Y-A feasibility assessment](snap-store-feasibility-assessment.md)
- [GNOME extension](https://documentation.ubuntu.com/snapcraft/latest/reference/extensions/gnome-extension/)
- [Rust plugin](https://documentation.ubuntu.com/snapcraft/stable/common/craft-parts/reference/plugins/rust_plugin/)
- [Snap desktop-file format](https://snapcraft.io/docs/reference/development/yaml-schemas/the-snap-format/)
