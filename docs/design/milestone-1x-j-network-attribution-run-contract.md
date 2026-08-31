# Milestone 1X-J network-attribution run contract

## Status

Gate consumed on 2026-08-31. After the separately approved execution change
and exactly one separately authorized run, the procedure recorded an
**attributed stop** during bounded application launch. The one-time workflow
was removed, no retry is authorized, and there is no complete silent-install
or silent-removal feasibility conclusion.

## Consumed execution record

| Field | Recorded value |
| --- | --- |
| Execution commit | `48ddf00beee4d6cfd9ecdb61b026eb1382b20682` |
| Procedure commit | `c34644adf8b5e2d0be0f25b189d627b03e98881d` |
| Procedure Git blob | `80071fab2377cbafa6276f16e8fb402f8bb01e14` |
| Workflow run | [`33425808598`](https://github.com/gregweir/ai-engine-room/actions/runs/33425808598) |
| Job | [`99598959867`](https://github.com/gregweir/ai-engine-room/actions/runs/33425808598/job/99598959867) |
| Result class | `Attributed stop` |
| Observed connection owner | `msedgewebview2.exe` (PID `7188`) |
| Immediate parent | `msedgewebview2.exe` (PID `2852`) |
| Observed endpoint | `150.171.22.17:443` (`Established`) |
| Cleanup | `registered silent uninstall completed` |
| Workflow removal commit | `fe023f0799ccd7865263833de1b39241be51211e` |

The process snapshot does not establish destination ownership, purpose, or
transmitted content. The full evidence and interpretation are in the
[disposable-runner verification record](../distribution/winget-disposable-runner-verification-record.md).

## Approval boundary

Merging this contract would authorize only later preparation of an exact
execution change. Before any installer job can be triggered, the developer
must separately approve:

1. the execution change containing the exact procedure and one-time workflow;
   and
2. exactly one run by citing that execution commit.

A continuation instruction, this contract's merge approval, either previous
run authorization, or approval of only one item above is insufficient.

## Reviewed baseline

| Field | Required value |
| --- | --- |
| Repository | `gregweir/ai-engine-room` |
| Baseline merge commit | `d64b194ff0f4fa1803ba01f44b54c20667640a90` |
| Procedure path | `scripts/verify-winget-silent-install.ps1` |
| Baseline procedure Git blob | `bdb1105263587dae7718cf16a013a41dc27e8cd3` |
| Proposed execution branch | `codex/winget-network-attribution-verification` |

The later execution change may modify the procedure only to replace its stale
approved-branch literal with the proposed branch above. It must add exactly one
one-time workflow file. No other path or procedure change is permitted in that
commit. The resulting commit and procedure blob must be reported for review
before the pull request is opened.

## Exact unchanged candidate

| Field | Required value |
| --- | --- |
| Release | `v0.1.0-preview.1` prerelease |
| Source commit | `a5482e9d51657a0cfb4471215a91750c5ba7db95` |
| Hosted filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Bytes | `2,651,735` |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Authenticode | `NotSigned` |
| Expected installed name | `AI Engine Room` |
| Expected installed version | `0.1.0` |
| Expected publisher | `Tartanleaf.com Inc.` |

Any difference stops before candidate execution.

## Disposable runner boundary

The proposed run uses one fresh GitHub-hosted `windows-2025` x64 virtual
machine. The job must record the runner environment, architecture, `ImageOS`,
`ImageVersion`, Windows caption, version, and build before downloading the
candidate. It must stop unless the runner is GitHub-hosted Windows x64 and the
operating system reports Windows Server 2025 version `10.0.26100`.

The rolling image version qualifies the result. The run would not support a
claim about other runner revisions, Windows desktop editions, either physical
verification host, or general Windows behavior.

## One-time trigger design

The execution workflow must:

1. trigger only on opening a pull request to `main` from the exact proposed
   branch in the exact repository;
2. use `windows-2025`, `contents: read`, and a ten-minute job timeout;
3. check out the immutable pull-request head with the repository's pinned
   checkout action;
4. assert the expected repository, base commit, execution commit, parent,
   changed-path set, runner identity, procedure blob, and candidate constants;
5. set `AER_WINGET_FEASIBILITY_APPROVED=true` only for the single job;
6. invoke the procedure exactly once, with no matrix, retry, fallback runner,
   alternate candidate, `continue-on-error`, or bypass; and
7. be removed immediately after the job reaches one conclusion, before any
   evidence-record update is pushed.

Opening the pull request is the one trigger. A cancelled, stopped, failed, or
incomplete job consumes the gate just as a passing job does.

## Required evidence

The run must preserve the earlier candidate, install, executable-discovery,
window, timeout, child-process, connection, and cleanup boundaries. For a
connection stop it must additionally record only:

- phase, owner PID, address, port, and TCP state;
- owner process name or explicit unavailable attribution;
- root-versus-descendant role; and
- immediate parent PID and process name when available.

The process lookup is a snapshot and may be stale or unavailable. It does not
establish destination ownership, purpose, trust, or transmitted content. The
procedure must not perform DNS, WHOIS, reputation, or other network lookups.

## Result interpretation

- **Pass:** the exact candidate completes silent installation, bounded launch
  without a selected external connection, primary silent removal, and all
  absence assertions on the recorded runner.
- **Attributed stop:** the procedure stops on a selected connection and
  captures the bounded owner/parent fields before registered cleanup.
- **Unattributed stop:** the connection is preserved but the immediate process
  lookup is unavailable before registered cleanup.
- **Other stop:** any identity, UI, timeout, process, metadata, removal,
  cleanup, infrastructure, or evidence mismatch.

Only the first case could support a narrow feasibility conclusion. Every stop
case supports only its observed facts, grants no retry, and must not be
described as proof that particular content was or was not transmitted.

## Explicit exclusions

This contract does not authorize:

- a workflow, pull-request trigger, installer execution, or retry;
- an address, process, publisher, service, or destination allowlist;
- weaker network, UI, process, timeout, cleanup, or evidence boundaries;
- packet inspection, traffic interception, firewall or routing changes,
  elevation, telemetry, persistence, provider access, or inference;
- product network-observability implementation;
- candidate rebuilding, signing, replacement, or release changes; or
- a WinGet manifest, identifier reservation, fork, submission, availability
  claim, or another distribution channel.

## Preparation acceptance

This contract is ready for review only if the branch contains documentation
changes alone, ordinary deterministic CI passes, the inactive procedure is
unchanged from the reviewed baseline blob, and no tracked workflow can invoke
it.

## References

- [Milestone 1X-I preparation](milestone-1x-i-network-attribution-preparation-contract.md)
- [Milestone 1X-H consumed run contract](milestone-1x-h-disposable-windows-rerun-contract.md)
- [Disposable-runner verification record](../distribution/winget-disposable-runner-verification-record.md)
- [WinGet validation FAQ](https://github.com/microsoft/winget-pkgs/blob/master/doc/FAQ.md)
- [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
