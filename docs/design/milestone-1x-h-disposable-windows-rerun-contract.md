# Milestone 1X-H disposable-Windows rerun contract

## Status

Completed and consumed on 2026-08-31. The developer separately approved this
contract, workflow commit `6ab843aaff84f97668acec9d61135f0603bf5c36`,
and exactly one run. Run `33409278455`, job `99544500003`, stopped on a bounded
non-loopback connection after installation and executable discovery passed;
registered cleanup completed. The workflow was removed in
`9a6b879936f78a064d657e958ad18ef91292fa57`. No retry or execution authority
remains. See the
[verification record](../distribution/winget-disposable-runner-verification-record.md).

## Decision requested after preparation

After this contract is merged, the developer may approve or defer exactly one
new run of the corrected silent-install procedure. Approval must cite this gate
and the later workflow commit. A generic continuation, merge approval, or prior
Milestone 1X-F authorization is not reusable execution authority.

## Exact reviewed procedure

| Field | Required value |
| --- | --- |
| Repository | `gregweir/ai-engine-room` |
| Procedure merge commit | `4ce1449fce60301256385680f471e89c6dfd9ea8` |
| Procedure path | `scripts/verify-winget-silent-install.ps1` |
| Procedure Git blob | `3e6142baf154656106b74c75a9fbeeefa570076b` |
| Proposed execution branch | `codex/winget-executable-discovery-verification` |

The future execution branch must start from a `main` commit containing that
exact procedure blob. The one-time workflow may add only its own workflow file
before execution. Any procedure change, candidate change, or additional file
change stops the gate and requires review again.

## Exact candidate

The candidate remains unchanged from the stopped run:

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

Any identity difference stops before execution.

## Disposable runner policy

The proposed run uses the explicit GitHub-hosted `windows-2025` x64 label, not
the moving `windows-latest` alias. GitHub's runner-image catalogue currently
maps `windows-2025` to Windows Server 2025. Standard GitHub-hosted jobs use a
new virtual machine that GitHub decommissions after the job.

The standard label does not pin a continuously updated image version. The job
must record `RUNNER_ENVIRONMENT`, `RUNNER_OS`, `RUNNER_ARCH`, `ImageOS`,
`ImageVersion`, Windows caption, version, and build before download. It must
stop unless the environment is GitHub-hosted Windows x64 and the OS reports
Windows Server 2025 version `10.0.26100`. The recorded image version qualifies
the result; it does not support a claim about other image revisions or Windows
desktop editions.

## One-time workflow contract

Only a later explicitly authorized execution change may add the workflow. It
must:

1. use `pull_request` only, with no schedule, push, or `workflow_dispatch`;
2. require the exact repository and execution branch above;
3. use `windows-2025`, `contents: read`, and a ten-minute job timeout;
4. use the repository's pinned `actions/checkout` commit;
5. verify the procedure Git blob before invoking it;
6. set the script's explicit approval environment variable only for that job;
7. invoke the procedure once, with no matrix, retry, rerun, fallback runner,
   alternate candidate, or continue-on-error behavior; and
8. be deleted from the branch immediately after the one conclusion is captured,
   before the evidence-record commit is pushed.

The workflow run identifier and job identifier become part of the permanent
record. Removal prevents a later branch synchronization from executing the
installer again.

## Required evidence and stop behavior

The corrected procedure must preserve the existing Milestone 1X-F evidence and
stop boundaries plus:

- registered `DisplayIcon` availability and in-directory resolution state;
- sanitized relative executable inventory, sizes, version fields, and
  uninstaller classification;
- executable-resolution method and candidate count before uniqueness is
  asserted;
- separately validated application product and company metadata;
- cleanup after every post-execution stop path; and
- the absence of an automatic second cleanup after completed primary removal.

One passing job would establish feasibility only for the exact candidate,
procedure, and recorded runner image. Any stop, cancellation, infrastructure
failure, or incomplete evidence establishes no conclusion and grants no retry.

## Explicit exclusions

This gate does not authorize:

- execution before a later explicit developer approval;
- execution on either physical host;
- a second job, manual rerun, alternate runner, bypass, trust change, or
  security-control change;
- provider access, inference, packet-content inspection, or continuous
  monitoring;
- candidate rebuilding, signing, republishing, or release changes;
- a WinGet manifest, identifier reservation, external fork, submission, or
  availability claim; or
- certificate application, purchase, MSIX/Store work, or another package
  channel.

## Acceptance criteria for this preparation

This preparation passes only if the contract identifies the exact procedure,
candidate, runner policy, proposed workflow behavior, evidence, cleanup,
single-run limit, and exclusions; ordinary deterministic CI passes; and review
finds no workflow or execution-capable change in the branch.

## Sources

- GitHub Docs,
  [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- GitHub,
  [Actions runner images](https://github.com/actions/runner-images)
- [Milestone 1X-G preparation](milestone-1x-g-executable-discovery-preparation-contract.md)
- [Stopped Milestone 1X-F record](../distribution/winget-disposable-runner-verification-record.md)
