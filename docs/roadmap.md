# AI Engine Room roadmap

AI Engine Room is a privacy-conscious desktop utility that helps people
understand local-AI runtime and resource state through **Observe → Explain →
Diagnose → Report**. It reports bounded evidence and qualifications; it does
not manage providers, promise model fit, infer compute placement, or act as a
hardware benchmark.

## Current product goals

- Make local runtime, model, and machine context understandable to non-experts.
- Distinguish observed facts from calculations, estimates, and unavailable evidence.
- Keep collection passive by default and inference explicitly authorized.
- Preserve report-safe output that excludes sensitive or unnecessary machine details.
- Provide bounded native behavior on Linux and Windows before public release.

## Current sequence

1. Milestone 1U is closed: total memory and native CPU architecture include the
   Windows Clippy correction and the developer decision on 225% Text size evidence.
2. Milestone 1V is closed at accepted source revision
   `1f19ea0f249a3b07a55d6fb53a70d0c45cdb7e51`: provider-reported model-size
   and configured-context presentation passed deterministic CI, bounded native
   Linux verification, and independent Windows review under
   [`design/milestone-1v-contract.md`](design/milestone-1v-contract.md). KV-cache
   bytes, runtime overhead, physical VRAM capacity, and compute placement remain
   unavailable or unknown; do not calculate them or make fit/headroom claims.
3. The separately approved current-source packaging-readiness contract was
   completed at source revision `2293b336eaa314f4fd285737a8470a5b9abd151a`.
   Its bounded `.deb` and NSIS builds, independent review, and exact-artifact
   install/launch/removal acceptance are complete historical evidence. Those
   artifacts remain local, unsigned, and unpublished; their acceptance does
   not transfer to a rebuilt preview candidate.
4. The fuller
   [`binary-distribution licence and notice review`](release/binary-distribution-license-notice-review.md)
   and its remediation are complete. Rebuilt `.deb` and NSIS licence payloads
   passed deterministic CI and independent package inspection.
5. The developer approved an
   [`unsigned preview release contract`](release/unsigned-preview-release-contract.md)
   and a bounded
   [`SignPath Foundation eligibility assessment`](release/signpath-eligibility-assessment.md).
   The first public preview remains unsigned; no signing action was taken.
   SignPath is a plausible no-cost route but is not application-ready. Greg
   Weir is the developer and Tartanleaf.com Inc. is the product/package
   publisher; whether a future SignPath certificate may instead authenticate
   SignPath Foundation remains undecided.
6. Exact unsigned-preview candidates were built and inspected from pushed
   `main` revision `a5482e9d51657a0cfb4471215a91750c5ba7db95` after passing
   deterministic CI. Their identities and the approved release-page text are
   recorded locally on a separate documentation branch. Physical exact-artifact
   install/UI/removal acceptance, installed-Windows-executable verification,
   and independent final review passed. The bounded Linux process-close
   deviation and its completed correction are recorded in the candidate
   evidence. The developer approved the exact artifacts, final text, GitHub
   Releases channel, `v0.1.0-preview.1` prerelease identity, `SHA256SUMS.txt`,
   and creation and upload of a non-public three-asset draft. After GitHub
   normalized spaces in the uploaded binary names, the developer approved the
   resulting dotted hosted filenames; the binary hashes and accepted bytes are
   unchanged.
7. The independently verified draft received a later, separate public-transition
   approval and is now the public
   [`AI Engine Room 0.1.0 unsigned preview`](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1).
   Fresh public downloads, checksums, unsigned status, pinned documentation
   links, and an independent public review all passed. The exact outcome is in
   [`the publication record`](release/unsigned-preview-publication-record.md).
   No further promotion, additional channel, asset replacement, signing action,
   or follow-on release is authorized. Future signing certificate identity,
   broader provider/platform coverage, and macOS remain later decisions.

No roadmap entry authorizes implementation by itself. The current milestone
state and developer gate control execution.
