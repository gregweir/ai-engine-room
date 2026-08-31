# WinGet disposable-runner verification record

Status: **STOPPED AFTER THREE SEPARATELY AUTHORIZED RUNS — no complete
silent-install or silent-removal feasibility conclusion** on 2026-08-31.

## First authorization and exact run

The developer authorized one bounded execution of the
[Milestone 1X-F contract](../design/milestone-1x-f-disposable-windows-silent-install-contract.md).
It ran once in a fresh GitHub-hosted Windows virtual machine:

| Field | Recorded value |
| --- | --- |
| Workflow run | [`33393023752`](https://github.com/gregweir/ai-engine-room/actions/runs/33393023752) |
| Job | [`99490900882`](https://github.com/gregweir/ai-engine-room/actions/runs/33393023752/job/99490900882) |
| Procedure commit | `36f8bcfb10155015f613f2643227e67cba0d10bf` |
| Runner environment | `github-hosted` |
| Runner image OS | `win25-vs2026` |
| Runner image version | `20260824.214.3` |
| Windows caption | `Microsoft Windows Server 2025 Datacenter` |
| Windows version and build | `10.0.26100` / `26100` |
| Architecture | `X64` |

The workflow completed with a failed result after 24 seconds. GitHub
automatically decommissioned the job's virtual machine afterward. The one-time
workflow was then removed so a later branch update cannot repeat the installer
execution.

## Candidate identity

All required candidate checks passed before execution:

| Field | Observed value |
| --- | --- |
| Filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Bytes | `2,651,735` |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Authenticode | `NotSigned` |

## Observed sequence and stop

The exact installer was invoked with the case-sensitive NSIS `/S` switch. The
cleanup boundary subsequently found one registered application entry, showing
that installation had reached registered state. The primary procedure stopped
before application launch with:

> The installed application executable does not exist.

The procedure expected `AI Engine Room.exe` beneath the registered install
location. The log did not preserve an inventory of the installed directory or
the registered `DisplayIcon`, so it does not establish whether the actual
executable had a different name or location. This stop therefore must not be
reported as an installer failure.

No application launch, provider access, inference, manifest creation,
submission, signing, release change, or physical-host execution occurred.

## Cleanup and network evidence

The `finally` boundary invoked the registered silent uninstaller and recorded
`registered silent uninstall completed`. The monitored process names were the
installer, `Un`, and `uninstall`. No non-loopback TCP connection was observed
for that monitored installer/uninstaller process tree.

That cleanup result demonstrates bounded containment after the stop. It is not
the planned silent-removal pass because the primary procedure stopped before
the full registry, executable, uninstaller, and process-absence assertions.

## Conclusion and future gate

Milestone 1X-F did not meet its acceptance criteria. The run provides no basis
for a WinGet silent-install claim, manifest, submission, package availability
claim, or change to the unsigned-preview policy.

There will be no automatic retry. A future run would require a separately
reviewed procedure that first records and resolves installed executable
identity from evidence such as the registered `DisplayIcon`, installed-file
inventory, and installer metadata. The developer must then explicitly approve
one new exact run and its candidate, environment, stop conditions, and cleanup
boundary.

The later
[Milestone 1X-G preparation contract](../design/milestone-1x-g-executable-discovery-preparation-contract.md)
defines that static procedure correction. It creates no workflow and grants no
execution authority; the stopped result above remains unchanged.

## Milestone 1X-H authorized rerun

The developer separately approved the
[Milestone 1X-H contract](../design/milestone-1x-h-disposable-windows-rerun-contract.md),
the execution change, and exactly one run using workflow commit
`6ab843aaff84f97668acec9d61135f0603bf5c36`. The workflow ran once:

| Field | Recorded value |
| --- | --- |
| Workflow run | [`33409278455`](https://github.com/gregweir/ai-engine-room/actions/runs/33409278455) |
| Job | [`99544500003`](https://github.com/gregweir/ai-engine-room/actions/runs/33409278455/job/99544500003) |
| Workflow commit | `6ab843aaff84f97668acec9d61135f0603bf5c36` |
| Workflow parent | `0eb0787e4d7dde1192af6deb25f580ba197bea98` |
| Procedure Git blob | `3e6142baf154656106b74c75a9fbeeefa570076b` |
| Runner environment | `github-hosted` |
| Runner image OS | `win25-vs2026` |
| Runner image version | `20260824.214.3` |
| Windows caption | `Microsoft Windows Server 2025 Datacenter` |
| Windows version and build | `10.0.26100` / `26100` |
| Architecture | `X64` |

The repository, branch, head, single changed workflow path, runner identity,
and procedure blob checks all passed before download. The workflow concluded
once with a failed result after 28 seconds. It was removed in commit
`9a6b879936f78a064d657e958ad18ef91292fa57` immediately after evidence capture.
The branch update scheduled no second disposable job; no retry is authorized.

### Candidate, installation, and executable evidence

The unchanged candidate filename, byte size, SHA-256, and `NotSigned`
Authenticode state all matched the first run. Its silent installer returned
exit code `0`. The procedure then observed exactly one registered application:

| Field | Observed value |
| --- | --- |
| Display name | `AI Engine Room` |
| Publisher | `Tartanleaf.com Inc.` |
| Display version | `0.1.0` |
| Install scope | Current user |
| Quiet uninstall registered | No |
| DisplayIcon registered and usable | Yes |
| DisplayIcon resolved within install directory | Yes |
| DisplayIcon resolved outside install directory | No |
| Executable resolution | `registered_display_icon` |
| Application candidate count | `1` |

The sanitized inventory contained `aiengineroom.exe` (10,898,944 bytes), with
product name `AI Engine Room`, company `Tartanleaf.com Inc.`, description
`AI Engine Room`, and file version `0.1.0`. It also contained the separately
classified registered uninstaller `uninstall.exe` (79,241 bytes). These facts
correct the first run's executable-path uncertainty, but do not by themselves
complete the full feasibility contract.

### Network stop and cleanup

During the bounded application launch, the monitored process tree opened one
established non-loopback TCP connection:

| Field | Observed value |
| --- | --- |
| Phase | `bounded application launch` |
| Process ID | `2936` |
| Remote endpoint | `150.171.22.17:443` |
| State | `Established` |

The recorded process-name set was the installer, `aiengineroom`,
`msedgewebview2`, `Un`, and `uninstall`. The evidence does not identify which
named process held PID `2936`, inspect packet content, establish the endpoint's
purpose, or show that prompts, responses, model data, telemetry, or any other
particular content was transmitted. It therefore supports only the narrow fact
that a monitored launch process made an external connection during this run.

The procedure stopped immediately and recorded `registered silent uninstall
completed`. That is successful cleanup after a bounded stop, not the planned
primary silent-removal pass. No provider or inference task was intentionally
started, and no manifest, submission, signing, release change, or physical-host
execution occurred.

### Rerun conclusion

Milestone 1X-H safely answered the executable-discovery question and exposed a
separate network-observation stop. Because launch did not complete its bounded
checks and the primary removal assertions did not run, the result establishes
no complete silent-install or silent-removal feasibility claim and grants no
WinGet manifest or submission authority. The one-run gate is consumed. Any
future work must begin with a new static interpretation and design decision;
this record grants no retry or weaker network boundary.

The later
[Milestone 1X-I preparation contract](../design/milestone-1x-i-network-attribution-preparation-contract.md)
provides that static interpretation. It preserves the connection stop and adds
bounded owner-PID process and parent attribution to the inactive procedure. It
creates no workflow and grants no execution, retry, manifest, or submission
authority.

## Milestone 1X-J authorized attribution run

The developer separately approved the
[Milestone 1X-J contract](../design/milestone-1x-j-network-attribution-run-contract.md),
the exact execution change, and exactly one run using workflow commit
`48ddf00beee4d6cfd9ecdb61b026eb1382b20682`. The workflow ran once:

| Field | Recorded value |
| --- | --- |
| Workflow run | [`33425808598`](https://github.com/gregweir/ai-engine-room/actions/runs/33425808598) |
| Job | [`99598959867`](https://github.com/gregweir/ai-engine-room/actions/runs/33425808598/job/99598959867) |
| Workflow commit | `48ddf00beee4d6cfd9ecdb61b026eb1382b20682` |
| Procedure commit | `c34644adf8b5e2d0be0f25b189d627b03e98881d` |
| Procedure Git blob | `80071fab2377cbafa6276f16e8fb402f8bb01e14` |
| Runner environment | `github-hosted` |
| Runner image OS | `win25-vs2026` |
| Runner image version | `20260824.214.3` |
| Windows caption | `Microsoft Windows Server 2025 Datacenter` |
| Windows version and build | `10.0.26100` / `26100` |
| Architecture | `X64` |

The repository, branch, pull-request base, execution history, changed paths,
runner identity, procedure blob, and candidate constants all passed before
execution. The job concluded once with a failed result after 31 seconds. The
one-time workflow was removed in commit
`fe023f0799ccd7865263833de1b39241be51211e` before this evidence update was
pushed. That branch update did not schedule another disposable job; no retry is
authorized.

### Candidate, installation, and attributed stop

The unchanged candidate filename, byte size, SHA-256, and `NotSigned`
Authenticode state matched the earlier runs. The silent installer returned
exit code `0`. The registered application and resolved executable evidence
again reported `AI Engine Room`, version `0.1.0`, publisher
`Tartanleaf.com Inc.`, current-user scope, and `aiengineroom.exe` resolved from
the registered DisplayIcon inside the install directory.

During bounded application launch, the monitored process tree opened one
established non-loopback TCP connection:

| Field | Observed value |
| --- | --- |
| Phase | `bounded application launch` |
| Owner process | `msedgewebview2.exe` |
| Owner PID | `7188` |
| Attribution method | `owner_pid_lookup` |
| Process-tree role | `descendant` |
| Immediate parent | `msedgewebview2.exe` (PID `2852`) |
| Remote endpoint | `150.171.22.17:443` |
| State | `Established` |

The procedure stopped immediately and recorded `registered silent uninstall
completed`. That is successful cleanup after the attributed stop, not the
planned primary silent-removal pass. The snapshot associates the connection
with a WebView2 descendant during this run; it does not establish destination
ownership or purpose, inspect packet content, or show that prompts, responses,
model data, telemetry, or any other particular content was transmitted.

### Final run conclusion

Milestone 1X-J produced the bounded owner and immediate-parent attribution it
was designed to collect, so its result class is **attributed stop**. It did not
complete bounded launch or the primary silent-removal assertions and therefore
establishes no complete silent-install or silent-removal feasibility claim.
The final one-run gate is consumed. It grants no retry, address or process
allow-list, weaker network boundary, WinGet manifest or submission authority,
release change, or product network-observability implementation.
