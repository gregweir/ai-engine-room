# Milestone 1X-G executable-discovery preparation contract

## Status

Completed and merged as `4ce1449fce60301256385680f471e89c6dfd9ea8` on
2026-08-31. This contract authorized procedure correction, documentation,
deterministic checks, and review only. It did not authorize an installer
download or execution, a disposable-runner workflow, or a second feasibility
run.

## Purpose

The one authorized Milestone 1X-F run stopped because its procedure assumed
that the installed application would be named `AI Engine Room.exe`. Repository
packaging evidence identifies the Windows binary as `aiengineroom.exe`, but the
stopped runner did not preserve the installed directory inventory or registered
`DisplayIcon` needed to establish the actual installed path.

Milestone 1X-G prepares a revised evidence-first procedure without converting
repository knowledge into an unobserved claim about the stopped run.

## Authorized static work

This preparation may:

1. remove the hard-coded installed executable path;
2. parse a registered `DisplayIcon` without recording an absolute user path;
3. inventory executable files beneath the registered install directory using
   relative paths, byte sizes, version fields, and uninstaller classification;
4. resolve the application first from an in-directory registered `DisplayIcon`,
   then from one unique matching product name, then from one unique
   non-uninstaller executable;
5. require exactly one resolved candidate and separately validate its product
   name and company metadata;
6. retain the existing candidate identity, disposable-runner, timeout, window,
   process, network, launch, removal, and no-bypass boundaries;
7. attempt registered cleanup after any installer execution that stops, even if
   the installer exits before the primary path records installation success;
   and
8. run only static parsing, repository-readiness, diff, and deterministic CI
   checks against the revised source.

The proposed future execution branch is
`codex/winget-executable-discovery-verification`. No workflow for that branch is
created by this milestone.

## Evidence and stop behavior

A future approved run must preserve sanitized executable inventory before it
asserts candidate uniqueness. It must stop without a feasibility conclusion if
a usable registered icon resolves outside the install directory, candidate
resolution is zero or ambiguous, executable product or company metadata
differs, or any existing Milestone 1X-F stop condition occurs.

An absent or unusable `DisplayIcon` is not itself a bypass. The procedure may
use its documented unique-candidate fallbacks, but it must record which method
resolved the path. Absolute install paths must not enter the evidence output.

## Explicit exclusions

This milestone does not authorize:

- creating or enabling a GitHub Actions installer workflow;
- downloading, installing, launching, or removing the candidate on any host;
- a second disposable-Windows run or a retry of the stopped run;
- changing, rebuilding, signing, or republishing the candidate;
- provider access, inference, packet-content inspection, or physical-host
  execution;
- a WinGet manifest, identifier reservation, fork, submission, or availability
  claim; or
- certificate application, purchase, packaging-channel change, or release.

## Acceptance criteria

Preparation passes only if:

- the hard-coded application filename is absent from the active procedure;
- discovery is bounded to executable files within the registered install
  directory and excludes the registered uninstaller;
- evidence records the discovery method, candidate count, and sanitized
  executable inventory before a uniqueness assertion;
- the cleanup boundary covers all post-execution stop paths without running a
  second cleanup after a completed primary removal;
- PowerShell parsing, repository-readiness checks, diff checks, and ordinary
  deterministic CI pass; and
- review finds no installer workflow or expanded execution authority.

## Subsequent gate

A separate developer decision may approve or defer one new exact run. Approval
would need to identify the reviewed procedure
commit, exact immutable candidate, runner image policy, one-time workflow,
evidence requirements, and automatic workflow removal. It would authorize one
run only, not a retry loop or WinGet submission.

The later
[Milestone 1X-H contract](milestone-1x-h-disposable-windows-rerun-contract.md)
prepares those exact terms but still grants no execution authority.

## Related evidence

- [Stopped disposable-runner verification](../distribution/winget-disposable-runner-verification-record.md)
- [Milestone 1X-F contract](milestone-1x-f-disposable-windows-silent-install-contract.md)
- [WinGet feasibility assessment](../distribution/winget-feasibility-assessment.md)
