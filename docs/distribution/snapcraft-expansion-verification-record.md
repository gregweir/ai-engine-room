# Snapcraft extension-expansion verification record

Status: **ONE-RUN GATE CONSUMED; EXPANSION STOP; NO BUILD** on 2026-08-31.

## Outcome

Exactly one separately authorized disposable-Ubuntu run expanded the static
Snapcraft proposal and stopped before compilation. The generated definition
exceeded the reviewed source and build-plan boundary, so the procedure returned
its intentional `expansion_stop` result with exit status `42`.

No build command ran. No `.snap` was produced, uploaded, installed, launched,
retained, registered, or submitted to the Snap Store. No account, name,
provider, inference, or product network-observability action occurred. The
one-time workflow was removed immediately, and no retry is authorized.

## Consumed gate identity

| Field | Recorded value |
| --- | --- |
| Baseline | `ac78d2613c5a5df5e3a9204345273c391c7890e5` |
| Procedure commit | `b0f1d8bb5260abf961454ec1f82ec50cc97726d9` |
| Procedure blob | `42d4085daab51730d1837b9d1ca18ea6638c58e9` |
| Execution commit | `e9ef30a6f21772cfc7e21607bb9ca9caf2107590` |
| Workflow blob | `57eb8da0de1ba109e2a411ac174e7e728cb03d22` |
| Pull request | [#24](https://github.com/gregweir/ai-engine-room/pull/24) |
| Workflow run | [`33450652377`](https://github.com/gregweir/ai-engine-room/actions/runs/33450652377) |
| Job | [`99679548099`](https://github.com/gregweir/ai-engine-room/actions/runs/33450652377/job/99679548099) |
| Workflow-removal commit | `437243e44a24cfe6eb3e77a48dfdc015179f0b6d` |
| Result class | `expansion_stop` |
| Result phase | `extension_expansion` |
| Exit status | `42` |

The GitHub job conclusion is `failure` because the fail-closed procedure used
nonzero status `42` for the expected stop. That conclusion is not a build
failure: the build stage was never entered.

## Disposable environment and tools

| Field | Recorded value |
| --- | --- |
| Runner environment | GitHub-hosted |
| Runner OS and architecture | Linux x64; Ubuntu 24.04.4 LTS; amd64 |
| Runner image | `ubuntu24`, version `20260823.283.1` |
| Kernel | `6.17.0-1022-azure` |
| Snapcraft channel | `latest/stable` |
| Snapcraft | `9.0.1`, revision `18514` |
| snap / snapd client | `2.76.3+ubuntu24.04` |
| installed snapd Snap | `2.76.2`, revision `27710` |
| core24 | `20260410`, revision `1643` |
| Host Node / npm | `v22.23.2` / `10.9.8` |
| Host Rust / Cargo | `1.98.0` / `1.98.0` |

The host Node and Rust values describe the disposable runner, not a completed
Snap build toolchain. The proposed Node 24 build snap, GNOME SDK, Rust `1.77.2`
toolchain, Ubuntu build packages, npm dependencies, and Cargo dependencies were
not resolved because expansion stopped first.

## Preserved identity and interfaces

The expanded definition preserved these reviewed values:

- name `ai-engine-room`, base `core24`, grade `devel`, and strict confinement;
- command `bin/aiengineroom` and common ID
  `com.tartanleaf.aiengineroom`;
- first-party source `.` and proposed Node build snap `node/24/stable`; and
- app plugs `desktop`, `desktop-legacy`, `gsettings`, `opengl`, `wayland`,
  `x11`, and the explicitly proposed `network`.

Contrary to the pre-run documentation expectation, this exact Snapcraft
revision did **not** add `mount-observe` or `calendar-service` to the app. That
is an observation about revision `18514`, not a promise about other Snapcraft
versions or future expansion behavior.

The generated snap-wide content plugs were `desktop`, `gtk-3-themes`,
`icon-themes`, `sound-themes`, `gnome-46-2404`, and `gpu-2404`. Their presence
was reported but did not itself cause this stop.

## Attributed stop

The complete expanded YAML had SHA-256
`e20a2da87d3d4d0368235f34611febe54f8067499407926a2b4899a4f7c3c649`.
The independent parser identified two reviewed-boundary mismatches:

1. **Build-plan representation:** the expanded document omitted the original
   `platforms` mapping instead of retaining either reviewed amd64 form. The
   runner itself was amd64, but runner architecture is not a substitute for an
   inspectable expanded build plan.
2. **Generated sources:** the extension added three sources beyond the
   first-party `.` source:
   - Snapcraft revision `18514`'s packaged desktop command-chain source;
   - the same revision's packaged GPU command-chain source; and
   - `https://github.com/canonical/gpu-snap.git` for generated GPU cleanup.

The generated `gpu-snap.git` entry shown in the expansion had no visible commit
or tag pin. This record does not claim that the fetched content would be unsafe
or that Snapcraft would necessarily select a moving revision, because no pull
or build occurred. It does establish that the static proposal did not disclose
or review that remote source and that its immutable identity was not present in
the expanded text. Proceeding would therefore have exceeded the approved
source boundary.

## Interpretation

The run establishes that the static file is accepted far enough by Snapcraft
9.0.1 to produce an expanded definition on the recorded Ubuntu runner. It does
not establish that the source compiles, that an artifact can be packaged, that
strict confinement works at runtime, or that the Snap Store would accept the
application.

The earlier interface concern should be updated, not erased: two previously
documented GNOME plugs were absent in this exact expansion, while new generated
GPU parts and sources became visible. Expansion must remain a gate because its
result depends on the installed Snapcraft revision.

## Subsequent disposition

The developer selected deferral in the separately reviewed
[Snap disposition](snap-disposition.md). The project will not retry this gate,
broaden its allowlist, replace the GNOME extension, or prepare a Snap Store
submission under the current release sequence. The tracked static definition
remains historical proposal evidence, not a supported package or execution
authority.

Future reconsideration requires evidence of meaningful demand, a fresh
developer-approved contract, and an immutable, reviewable boundary for every
generated and external source before any new expansion or build gate.

## References

- [Milestone 1Y-C preparation contract](../design/milestone-1y-c-snapcraft-one-build-preparation-contract.md)
- [Milestone 1Y-D disposition contract](../design/milestone-1y-d-snap-disposition-contract.md)
- [Snap disposition](snap-disposition.md)
- [Static Snapcraft packaging proposal](snapcraft-static-proposal.md)
- [Snapcraft GNOME extension](https://documentation.ubuntu.com/snapcraft/latest/reference/extensions/gnome-extension/)
- [Snapcraft build environment options](https://documentation.ubuntu.com/snapcraft/stable/reference/build-environment-options/)
