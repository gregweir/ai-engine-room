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
3. Review and separately approve the proposed
   [`current-source packaging-readiness contract`](design/current-source-packaging-readiness-contract.md).
4. Only after that approval, reassess local unsigned `.deb` and NSIS packaging
   from the accepted source revision through bounded build and read-only artifact
   inspection. This step does not authorize package execution or release.
5. Make separate developer decisions for install/launch/removal acceptance,
   signing, publication, distribution, broader provider/platform coverage, and
   macOS.

No roadmap entry authorizes implementation by itself. The current milestone
state and developer gate control execution.
