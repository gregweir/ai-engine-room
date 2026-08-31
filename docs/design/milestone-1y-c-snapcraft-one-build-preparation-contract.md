# Milestone 1Y-C Snapcraft one-build preparation contract

## Status

Gate consumed on 2026-08-31. After the separately approved execution change
and exactly one separately authorized run, the procedure recorded an
**expansion stop** before compilation. The one-time workflow and procedure were
removed, no retry is authorized, and no Snap artifact exists. This consumed
gate does not authorize installing or invoking Snapcraft again, building or
installing a Snap, or taking any account or store action.

## Consumed execution record

| Field | Recorded value |
| --- | --- |
| Baseline | `ac78d2613c5a5df5e3a9204345273c391c7890e5` |
| Procedure commit | `b0f1d8bb5260abf961454ec1f82ec50cc97726d9` |
| Procedure blob | `42d4085daab51730d1837b9d1ca18ea6638c58e9` |
| Execution commit | `e9ef30a6f21772cfc7e21607bb9ca9caf2107590` |
| Workflow run | [`33450652377`](https://github.com/gregweir/ai-engine-room/actions/runs/33450652377) |
| Job | [`99679548099`](https://github.com/gregweir/ai-engine-room/actions/runs/33450652377/job/99679548099) |
| Result class | `expansion_stop` |
| Workflow removal | `437243e44a24cfe6eb3e77a48dfdc015179f0b6d` |

The complete evidence and bounded interpretation are in the
[Snapcraft expansion verification record](../distribution/snapcraft-expansion-verification-record.md).

## Purpose

Milestone 1Y-B merged a strict, development-grade packaging hypothesis without
executing it. This slice defines the smallest disposable-runner procedure that
could test that hypothesis while preserving a hard review boundary between
extension expansion and compilation.

## Approval boundary

Merging this preparation contract would authorize only a later exact execution
change. Before anything can install or invoke Snapcraft, the developer must
separately approve:

1. an execution change containing the complete procedure and one-time workflow;
   and
2. exactly one run by citing that execution commit.

A continuation instruction, this contract's merge approval, either prior Snap
approval, or approval of only one item above is insufficient. A cancelled,
stopped, failed, or incomplete run consumes the one-run gate.

## Reviewed baseline

| Field | Required value |
| --- | --- |
| Repository | `gregweir/ai-engine-room` |
| Source and baseline commit | `54d58e209399b830e929e4793e06dc778960689f` |
| Snapcraft definition | `snap/snapcraft.yaml` |
| Definition Git blob | `ff6d085085ae7dd693e2834541c5b1cb2c3c965c` |
| Desktop launcher Git blob | `bd5bc3af83ca13ae29d53e4331255296c1526611` |
| Proposed execution branch | `codex/snapcraft-one-build-execution` |

The later execution change may add only one inactive verification procedure
and one one-time workflow. It must not alter the definition, launcher,
application, dependency locks, deterministic workflow, or release files. The
execution commit, parent, changed paths, procedure blob, workflow blob, and
complete commands must be reported before its trigger pull request is opened.

## Disposable environment

The proposed run uses exactly one fresh GitHub-hosted `ubuntu-24.04` amd64
runner with `contents: read`, no repository or store secrets, no matrix, and a
45-minute job timeout. It must record the runner image labels, image version,
kernel, machine architecture, Ubuntu release, and package architecture before
installing build tooling.

The job may install the current `latest/stable` classic Snapcraft snap only on
that disposable runner. It must record the advertised channel and revision,
`snapcraft --version`, `snap version`, and the installed core revision. Use of
Multipass, LXD, Launchpad remote build, a physical host, a self-hosted runner,
or a second environment is excluded.

## One-time trigger design

The later workflow must:

1. trigger only when a pull request to `main` is opened from the exact execution
   branch in the exact repository;
2. check out the immutable pull-request head using the repository's pinned
   checkout action;
3. assert the repository, base, execution commit, parent, changed-path set,
   runner identity, definition and launcher blobs, and procedure blob;
4. set a procedure-specific approval variable only for the single job;
5. invoke the procedure exactly once, with no retry, fallback runner,
   `continue-on-error`, or artifact upload; and
6. be removed immediately after the job reaches one conclusion, before any
   verification-record update is pushed.

Opening the pull request is the one trigger. Ordinary deterministic CI may run
in parallel but cannot invoke the Snapcraft procedure.

## Expansion gate

After installing Snapcraft, the procedure must run
`snapcraft expand-extensions` from the immutable checkout and retain its full
text in the public job log. Before any build command, it must independently
parse and report:

- every app and snap-wide plug, including interface, target, and default
  provider fields;
- every build snap and content snap, with resolved installed revision when
  available;
- every package added to the part;
- every command chain, environment entry, layout, and source; and
- the resulting base, grade, confinement, platform, command, common ID, and
  desktop launcher.

The procedure must stop before compilation if the expansion adds an unknown
source, a non-amd64 build plan, classic or devmode confinement, a command or
identity change, a store or account operation, or any access not explicitly
justified by implemented product behavior. In particular,
`mount-observe` and `calendar-service` are not currently justified and must
cause a recorded expansion stop if generated. The gate cannot silently accept,
remove, or replace them during the run.

An expansion stop is a useful packaging finding and consumes the gate. It does
not authorize editing the definition, broadening the allowlist, or proceeding
to the build.

## Conditional one-build procedure

Only if the expansion passes every reviewed assertion may the same disposable
runner invoke Snapcraft once with `--destructive-mode` and an amd64-only build
plan. Destructive mode is permitted solely because the runner is disposable;
it is not approved for either physical verification machine or a shared host.
The procedure must not invoke a second lifecycle attempt, clean-and-retry,
remote build, or alternate toolchain.

The build stage must record:

- resolved Node build-snap, Rust toolchain, core, GNOME SDK/platform, GPU
  content, Ubuntu package, npm, and Cargo dependency versions or revisions;
- the single artifact filename, byte size, and SHA-256;
- unpacked `meta/snap.yaml`, desktop launcher, complete file list, and packaged
  command identity;
- dynamic-library resolution for the packaged executable; and
- presence and SHA-256 of `LICENSE`, `NOTICE`,
  `THIRD-PARTY-LICENSES.txt`, and `THIRD-PARTY-SOURCES.txt`.

Exactly one `.snap` must exist. Missing, additional, malformed, or unexpected
output is an attributed stop. The `.snap` must not be uploaded as a workflow
artifact, installed, launched, refreshed, published, or retained after the
disposable runner is destroyed.

## Result classes

- **Expansion stop:** the definition expands, but generated access, sources,
  identity, or build plan exceeds the reviewed boundary.
- **Build pass:** expansion passes, one build succeeds, and all local artifact
  and licence inspections pass.
- **Build stop:** expansion passes, but tooling, dependency resolution,
  compilation, packaging, or artifact inspection does not pass exactly once.
- **Infrastructure stop:** runner identity, checkout identity, tool
  installation, or evidence capture cannot be established.

Only a build pass could justify proposing a later, separately approved local
installation and strict-confinement runtime gate. None of these results proves
store acceptance, runtime behavior, provider compatibility, privacy, signing,
or release readiness.

## Explicit exclusions

This preparation and the proposed run do not authorize:

- a Snap account, login, name check or registration, upload, review request,
  channel, track, release, or store contact;
- installing, launching, refreshing, removing, or connecting interfaces for
  the built Snap;
- an artifact upload, release attachment, package publication, or retention;
- use of either physical verification machine, another physical host, or a
  self-hosted runner;
- provider access, inference, DNS/WHOIS/reputation lookup, packet inspection,
  network capture, firewall change, or product network observability;
- classic or devmode confinement, a broader interface set, an alternate base
  or architecture, dependency changes, or an ad hoc retry; or
- changes to the public Debian/NSIS preview, signing, application behavior, or
  existing release evidence.

## Preparation acceptance

This contract is ready for merge review only if the branch contains
documentation and deterministic-guardrail changes alone, no execution workflow
or procedure exists, no Snapcraft output exists, and ordinary deterministic CI
passes.

## References

- [Milestone 1Y-B static proposal](milestone-1y-b-snapcraft-static-proposal-contract.md)
- [Static Snapcraft packaging proposal](../distribution/snapcraft-static-proposal.md)
- [Expand extensions](https://documentation.ubuntu.com/snapcraft/stable/reference/commands/expand-extensions/)
- [Build environment options](https://documentation.ubuntu.com/snapcraft/stable/reference/build-environment-options/)
- [GNOME extension](https://documentation.ubuntu.com/snapcraft/latest/reference/extensions/gnome-extension/)
