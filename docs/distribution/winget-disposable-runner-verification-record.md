# WinGet disposable-runner verification record

Status: **STOPPED — no silent-install or silent-removal feasibility
conclusion** on 2026-08-31.

## Authorization and exact run

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
