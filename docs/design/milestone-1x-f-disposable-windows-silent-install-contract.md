# Milestone 1X-F disposable-Windows silent-install contract

## Status

Developer-approved and executed once on 2026-08-31 against the exact public
Windows preview in a fresh GitHub-hosted Windows virtual machine. The run
stopped before launch because the expected installed executable path did not
exist. It established no silent-install or silent-removal feasibility
conclusion, and this contract does not authorize a retry.

## Purpose

Milestone 1X-F resumes the stopped 1X-B preflight without weakening its
isolation requirement. GitHub documents that each standard hosted Windows job
runs in a newly provisioned virtual machine that is automatically
decommissioned after the job. Standard hosted runners are free for this public
repository.

This run asks only whether the exact current NSIS preview can install and
remove without interaction in that disposable environment while exposing the
expected installed metadata. It does not reproduce Microsoft's private WinGet
validation or security scans.

## Exact candidate

The run is limited to:

| Field | Required value |
| --- | --- |
| Release | `v0.1.0-preview.1` prerelease |
| Source commit | `a5482e9d51657a0cfb4471215a91750c5ba7db95` |
| Hosted filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Bytes | `2,651,735` |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Authenticode | `NotSigned` |
| Expected installed version | `0.1.0` |
| Expected publisher metadata | `Tartanleaf.com Inc.` |

Any identity difference stops the run before execution.

## Authorized procedure

The one-run procedure may:

1. execute only on a GitHub-hosted `windows-latest` runner created for the
   approved branch;
2. establish clean pre-install registry and process state;
3. download the exact immutable installer from the public GitHub release;
4. verify filename, bytes, SHA-256, and `NotSigned` status;
5. run the documented, case-sensitive NSIS `/S` installation switch with a
   bounded timeout;
6. fail on any visible installer window, non-zero result, policy or execution
   block, or non-loopback connection associated with the installer process
   tree;
7. inspect only the required uninstall-registry identity, installed executable
   identity, and install location;
8. launch the app once without providers or inference, require a bounded stable
   process, and fail on a non-loopback connection in its process tree;
9. close the app, run the registered quiet uninstaller without a bypass, and
   require successful registry, executable, uninstaller, and process absence;
   and
10. emit sanitized job-log evidence before GitHub decommissions the VM.

The script must attempt cleanup in a `finally` boundary if the primary path
stops after installation. VM decommissioning is the final containment boundary,
not evidence that removal passed.

## Stop conditions

Stop and establish no feasibility conclusion if:

- the job is not the approved GitHub-hosted branch run;
- the runner is not Windows x64 or is not initially clean;
- download identity, hash, bytes, or signing state differ;
- installation or removal shows a window, times out, returns non-zero, or is
  blocked;
- required installed name, publisher, version, location, executable, or quiet
  uninstall data are missing or different;
- the app does not remain running for the bounded launch observation;
- an installer, application, WebView, or uninstaller descendant opens a
  non-loopback TCP connection;
- cleanup cannot be attempted through the registered uninstaller; or
- any bypass, trust change, security-control change, retry with a different
  binary, provider access, or inference would be required.

## Explicit exclusions

This milestone does not authorize:

- execution on either physical host;
- provider access, inference, clipboard use, graphical or accessibility claims,
  packet-content inspection, or continuous monitoring;
- changing, rebuilding, signing, or republishing the installer;
- a WinGet manifest, package identifier approval, external fork, submission,
  contributor agreement, or catalog claim;
- certificate purchase or application; or
- any conclusion beyond this exact candidate and disposable runner image.

## Acceptance criteria

The milestone passes only if the one-run job completes every authorized step,
the recorded evidence is internally consistent, a repository record preserves
the result and limits, deterministic CI passes, and independent review finds no
scope or evidence discrepancy.

Those criteria were not met. The exact result and containment evidence are
preserved in the
[disposable-runner verification record](../distribution/winget-disposable-runner-verification-record.md).
Any revised procedure and second run require a separate developer gate.

## Sources

- GitHub Docs,
  [Using GitHub-hosted runners](https://docs.github.com/en/actions/how-tos/manage-runners/github-hosted-runners/use-github-hosted-runners)
- GitHub Docs,
  [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- NSIS documentation,
  [Command Line Usage](https://nsis.sourceforge.io/Docs/Chapter3.html)
- [WinGet feasibility assessment](../distribution/winget-feasibility-assessment.md)
- [Stopped 1X-B preflight](../distribution/winget-silent-install-preflight-record.md)
- [Unsigned-preview publication record](../release/unsigned-preview-publication-record.md)
