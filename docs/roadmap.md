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
2. The developer approved the bounded Milestone 1V contract in
   [`design/milestone-1v-contract.md`](design/milestone-1v-contract.md).
3. Complete and verify provider-reported model-size and configured-context
   presentation under that contract. KV-cache bytes, runtime overhead, physical
   VRAM capacity, and compute placement remain unavailable or unknown; do not
   calculate them or make fit/headroom claims.
4. Reassess current-source Linux and Windows packaging and release readiness.
5. Make separate developer decisions for signing, publication, distribution, broader provider/platform coverage, and macOS.

No roadmap entry authorizes implementation by itself. The current milestone
state and developer gate control execution.
