# WinGet silent-install feasibility preflight record

Status: **stopped before download or execution** on 2026-08-31.

## Authorization and boundary

The developer authorized the next previously identified WinGet gate: one
bounded, reversible Windows silent-install feasibility run against the exact
public preview installer. The authorization did not extend to manifest
creation, WinGet submission, release changes, signing, provider access,
inference, or network-observability implementation.

The approved assessment requires a fresh Windows Sandbox or equivalent
disposable Windows environment. It also requires the run to stop on an
environment mismatch, warning, policy block, unexpected network request,
unexpected child process, non-zero result, or need for a bypass.

## Exact candidate resolved before execution

Tracked publication evidence resolved the possible test candidate without
downloading it:

| Field | Resolved identity |
| --- | --- |
| Release | `v0.1.0-preview.1` prerelease |
| Source commit | `a5482e9d51657a0cfb4471215a91750c5ba7db95` |
| Hosted filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Format | NSIS executable, x86-64 |
| Bytes | `2,651,735` |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Installed version expected from existing evidence | `0.1.0` |
| Publisher metadata expected from existing evidence | `Tartanleaf.com Inc.` |
| Authenticode expected from existing evidence | `NotSigned` |

Tauri's current NSIS distribution guidance documents uppercase `/S` for
silent installation. The switch was not executed in this preflight, and the
exact uninstaller path and unattended-removal behavior remain unverified.

## Host preflight and stop

The available Windows host reported Windows Home, version 25H2, build
`26200.9278`, x64. `WindowsSandbox.exe` was absent. Microsoft documents Windows
Sandbox support for Pro, Enterprise, Pro Education/SE, and Education editions
and explicitly states that Windows Home is unsupported.

The host also no longer matches the exact previously verified Windows build
`26200.7462`. Running the unsigned installer directly on this physical Home
host would weaken both the disposable-environment requirement and the exact
baseline discipline.

The run therefore stopped before:

- downloading or executing the installer;
- changing Installed apps or application files;
- launching AI Engine Room;
- invoking an uninstaller;
- creating a manifest or package identifier; or
- changing any release, package-manager, signing, or publisher state.

No silent-install or silent-removal conclusion was established.

## Other read-only readiness gaps checked

### Official publisher-source discoverability

A focused public check of the Tartanleaf home page and Tools page on
2026-08-31 found no AI Engine Room listing or link to the exact release asset.
The GitHub release is public, but the assessment separately requires an
official publisher-controlled HTTPS page that identifies the application and
links the immutable version-specific installer. That evidence remains absent.

This is a point-in-time check of those public surfaces, not a claim that no
unindexed or unrelated page can exist.

### Release and installed-version alignment

The public release identity remains `v0.1.0-preview.1`, while the installer and
existing Installed apps evidence use `0.1.0`. The existing preview was not
designed as a WinGet version contract and must not be repurposed automatically.
At this preflight, the relationship had not yet been resolved.

A subsequent documentation review clarified that current WinGet guidance
recommends matching `PackageVersion` to the installed Apps & Features version.
The existing candidate would therefore use `0.1.0`; its prerelease tag is a
separate channel label that requires consistent explanation rather than an
automatic schema failure. See the
[identity and publisher-page plan](winget-identity-and-publisher-page-plan.md).

## Evidence sources

- [Milestone 1X-A feasibility contract](../design/milestone-1x-a-winget-feasibility-contract.md)
- [WinGet feasibility assessment](winget-feasibility-assessment.md)
- [Unsigned preview publication record](../release/unsigned-preview-publication-record.md)
- [Microsoft Windows Sandbox documentation](https://learn.microsoft.com/windows/security/application-security/application-isolation/windows-sandbox/)
- [Tauri NSIS silent-install guidance](https://v2.tauri.app/distribute/microsoft-store/#silent-install)
- [Tartanleaf home page](https://www.tartanleaf.com/)
- [Tartanleaf Tools page](https://www.tartanleaf.com/tools/)

## Recommended next gate

Do not purchase a Windows edition upgrade solely for this feasibility test and
do not relax the isolation requirement. Resume only when an existing licensed
Windows Pro, Enterprise, Education, or equivalent disposable Windows VM is
available. A renewed developer gate should approve:

1. the exact disposable environment and its clean-state evidence;
2. the exact immutable installer identity and hash;
3. the unattended install and removal commands;
4. the required metadata fields and residue checks; and
5. the observation and stop procedure for dialogs, warnings, processes, and
   network activity.

The publisher-page and release-identity clarity work can be addressed
separately, but neither should trigger release or website changes without its
own approval.
