# Current-source packaging-readiness proposed contract

Status: proposed on 2026-08-29. This document does not authorize execution.
Developer approval is required before any package build or artifact creation.

## Purpose

Reassess whether AI Engine Room's accepted Milestone 1V source can produce the
existing bounded Linux and Windows pre-release package formats without carrying
forward evidence from an older source revision or implying release readiness.

The accepted Milestone 1V application baseline that a packaging candidate must
contain is:

- commit `1f19ea0f249a3b07a55d6fb53a70d0c45cdb7e51`;
- tree `38898fe41808e596170d2bd0e2b0bed91286b853`; and
- sole origin `https://github.com/gregweir/ai-engine-room.git`.

The separate execution approval must name one exact packaging-candidate commit
and tree that contain this baseline and the approved contract record. Both build
hosts must use that same identity. Any later source change requires a new
identity decision before packaging.

## Authorized formats after approval

- Verified Ubuntu 24.04 x86-64 build host: exactly one local `.deb` bundle.
- Verified Windows 11 x86-64 build host: exactly one local NSIS installer.

AppImage, MSI, Microsoft Store, macOS, updater, archive, container, and other
package or distribution formats remain excluded.

## Entry gate

Before either build, record hostname, user, exact checkout path, branch, HEAD,
local `main`, cached `origin/main`, live origin `main`, HEAD tree, configured
remotes, ahead/behind counts, and tracked, staged, and untracked state.

Proceed only when every commit identity equals the exact packaging-candidate
commit named in the execution approval, the tree equals its recorded tree, the
origin is exact, the checkout is on `main`, ahead/behind is `0/0`, and tracked,
staged, and untracked state is clean. Refuse on dirt, divergence, identity
mismatch, additional remotes, or path mismatch.

## Bounded build scope after approval

Each host may:

1. run repository readiness and the existing deterministic frontend and Rust
   checks using already approved toolchains and locked dependencies;
2. use the repository's existing Tauri configuration and documented command for
   its single authorized format;
3. allow normal Tauri bundler-helper downloads only when required by the existing
   toolchain and separately authorized by the developer or an existing approval;
4. leave build outputs only in existing ignored build locations; and
5. record exact commands, tool versions, artifact path, filename, byte size, and
   SHA-256.

No project dependency, lockfile, source, configuration, workflow, or Git ref may
be changed to make a build pass. A failure is evidence to report, not permission
to fix or broaden scope.

## Read-only artifact inspection

Linux inspection is bounded to package format, control metadata, architecture,
maintainer/publisher, version, description, declared dependencies, desktop
entry, icon paths, and package contents using local package tools.

Windows inspection is bounded to PE and version metadata, architecture, product
identity, approved icon evidence available without execution, filename, size,
SHA-256, and Authenticode status. The expected status is unsigned; inspection
must not sign or establish publisher trust.

The Linux and Windows evidence must identify their own artifact and source.
Evidence for one artifact, host, or platform must not be transferred to the
other.

## Prohibited actions and claims

This contract does not authorize:

- executing or installing either package;
- launching the packaged application or exercising its UI;
- uninstalling or changing package-manager state;
- starting, stopping, probing, or reconfiguring Ollama, LM Studio, llama.cpp, or
  any other provider;
- inference, benchmarking, performance testing, or compute-placement claims;
- signing, notarization, trust-policy changes, security-warning bypasses, upload,
  artifact publication, distribution, GitHub Release creation, or public release;
- AppImage media-framework changes or broader dependency bundling;
- private-evidence access;
- production-ready, release-candidate, broad compatibility, security, privacy,
  accessibility-conformance, model-fit, or headroom claims; or
- source, documentation, dependency, configuration, CI, ref, or worktree changes
  during the build-and-inspection operation.

## Evidence and review requirements

- Record the Linux result in the existing Linux pre-release evidence structure
  without overwriting historical artifact identity.
- Record the Windows result in the existing Windows pre-release evidence
  structure without overwriting historical artifact identity.
- Preserve failures and limitations exactly; do not turn missing evidence into a
  pass.
- Perform an independent review of source identity, command scope, artifact
  metadata, hashes, unsigned state, exclusions, and final clean Git state.
- Keep artifacts local, ignored, unsigned, unpublished, and unexecuted.

## Acceptance criteria

- Both entry gates pass against the exact packaging-candidate commit and tree
  named in the execution approval.
- Deterministic checks pass natively on both build hosts.
- Exactly one authorized artifact is produced on each host.
- Each artifact passes the bounded read-only inspection for its platform.
- Source revision, artifact identity, commands, tool versions, hashes, results,
  exclusions, and limitations are recorded without superseding historical
  evidence.
- Independent review reports no blocking discrepancy.
- Both Git checkouts remain clean and `0/0` after the operation.

Passing these criteria establishes only that the explicitly authorized source
produced two local, unsigned, unpublished, unexecuted pre-release verification
artifacts on the tested hosts. It does not establish installation or runtime
acceptance, signing readiness, publication readiness, release-candidate status,
or public-release approval.

## Subsequent developer gates

The developer must make separate decisions for:

1. exact-artifact install, native launch, bounded UI/accessibility/passive
   behavior, uninstall, and post-removal acceptance on each platform;
2. fuller bundled-dependency licence and notice review before public binary
   distribution;
3. signing and publisher identity;
4. upload, hosting, publication, distribution, or public release; and
5. any additional format, platform, architecture, or provider coverage.
