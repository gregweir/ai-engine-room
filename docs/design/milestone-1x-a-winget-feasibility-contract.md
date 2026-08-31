# Milestone 1X-A WinGet-feasibility contract

## Status

Developer-approved on 2026-08-31 for one bounded, documentation-only
distribution-feasibility implementation.

## Purpose

Milestone 1X-A determines what evidence AI Engine Room would need before the
developer could decide whether to submit a Windows package to the Windows
Package Manager Community Repository. This first slice records current
evidence, gaps, stop conditions, and later gates. It does not submit or publish
a package.

## Authorized work

This implementation authorizes only:

- this contract;
- one source-linked WinGet feasibility assessment; and
- a roadmap update that records the deferred non-specialist round and the new
  distribution-feasibility sequence.

The assessment may rely on public Microsoft documentation, the public AI
Engine Room repository and release record, and existing tracked package
evidence. It may not execute an installer or change external state.

## Required questions

The assessment must distinguish established evidence from work that still
requires a later gate, including:

- whether the existing NSIS format is accepted by the community repository;
- whether the package identity, publisher, version, architecture, installer
  type, scope, URLs, hashes, licence, and Add/Remove Programs metadata can be
  represented accurately;
- whether installation and removal can complete without user interaction;
- whether a version-specific installer URL is public, immutable, and
  discoverable from an official publisher-controlled page;
- whether the unsigned installer can pass the repository's security scans,
  execution checks, and warning-sensitive validation;
- how a candidate would be tested in an isolated Windows environment;
- which manifest files and submission checks would be required; and
- what ongoing update and maintenance obligations publication would create.

Unknown or untested conditions must remain unknown or untested. The assessment
must not interpret the absence of a formal Authenticode requirement for NSIS as
evidence that an unsigned installer will be accepted.

## Explicit exclusions

This slice does not authorize:

- downloading, executing, installing, uninstalling, rebuilding, repackaging,
  signing, or replacing a release asset;
- creating or validating a WinGet manifest against an installer;
- forking or cloning `microsoft/winget-pkgs`, accepting an external contributor
  agreement, opening an issue or pull request, or contacting a moderator;
- editing the existing GitHub prerelease or its tag, assets, checksums, or
  public text;
- publishing through WinGet, Microsoft Store, Snap Store, a PPA, Debian,
  Ubuntu, Flathub, or a self-hosted repository;
- purchasing or applying for a certificate, establishing publisher trust, or
  changing the publisher identity;
- changing the product, provider, inference, updater, packaging, dependencies,
  CI, or release process; or
- claiming package-manager, broader Windows, signing, security, or production
  readiness.

## Acceptance criteria

This preparation slice is complete only when:

- every conclusion identifies its local or official public source;
- the assessment records the exact existing installer identity and separates
  historical evidence from new verification still required;
- a later silent-install test is bounded, reversible, isolated, and stopped by
  any warning, prompt, policy block, scan failure, unexpected process, or
  incomplete removal;
- the submission sequence includes manifest validation, isolated installation
  testing, security validation, moderator review, and ongoing version updates;
- signing is described as a risk and trust decision rather than a falsely
  settled prerequisite or guarantee;
- WinGet remains the first candidate and Snap remains a separately gated later
  feasibility study; and
- the diff is limited to this contract, the feasibility assessment, and the
  roadmap.

## Subsequent gates

Completion of this slice authorizes no external action. The next possible gate
is one local, bounded Windows feasibility run against a separately approved
exact installer candidate. Creating a manifest, publishing a new release,
submitting to `microsoft/winget-pkgs`, responding to external review, and
maintaining later versions each require appropriate developer authorization.
