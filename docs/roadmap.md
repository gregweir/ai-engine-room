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
   No upload, hosting, publication, distribution, or release is authorized.
   SignPath is a plausible no-cost route but is not application-ready, and its
   Foundation publisher identity remains undecided.
6. Exact unsigned-preview candidates were built and inspected from pushed
   `main` revision `a5482e9d51657a0cfb4471215a91750c5ba7db95` after passing
   deterministic CI. Their identities and the proposed release-page text are
   recorded locally on a separate documentation branch. Physical exact-artifact
   install/UI/removal acceptance, installed-Windows-executable verification,
   final independent review, and every publication approval remain pending.
7. Next, complete those physical checks when the developer is present at both
   machines, then request separate decisions on the exact artifacts, final
   release-page text, and publication channel. Signing/publisher identity,
   broader provider/platform coverage, and macOS remain later decisions.

No roadmap entry authorizes implementation by itself. The current milestone
state and developer gate control execution.
