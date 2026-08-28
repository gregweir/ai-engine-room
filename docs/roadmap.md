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

1. Close Milestone 1U: total memory and native CPU architecture, including the Windows Clippy correction and the owner decision on 225% Text size evidence.
2. Owner roadmap decision before Milestone 1V.
3. Add memory-use and KV-cache capacity/context presentation only under a separately approved contract that distinguishes model weights, context/KV cache, runtime overhead, system memory, VRAM, and unknown placement without making unsupported fit claims.
4. Reassess current-source Linux and Windows packaging and release readiness.
5. Make separate owner decisions for signing, publication, distribution, broader provider/platform coverage, and macOS.

No roadmap entry authorizes implementation by itself. The current milestone
state and owner gate control execution.
