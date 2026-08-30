# AI Engine Room 0.1.0 unsigned preview — release-notes draft

> **DRAFT — DO NOT PUBLISH OR DISTRIBUTE**
>
> Physical exact-artifact acceptance and independent final review have passed.
> All publication approvals remain pending.

AI Engine Room 0.1.0 is a deliberately unsigned preview for people exploring
bounded local-AI runtime and resource context. It is pre-release software, not
a stable, production-ready, security-certified, or broadly compatible release.

## Highlights

- Observe Ollama and LM Studio availability, catalogues, loaded state, and
  bounded provider-reported model metadata through fixed same-machine APIs.
- Passively inspect the supported traditional single-model llama.cpp contract
  without inference or model/server management.
- Present available memory, total memory, and native CPU architecture while
  keeping missing evidence unavailable rather than inventing values.
- Distinguish model weights, configured context, KV cache, runtime overhead,
  VRAM, and compute placement; unknown quantities remain unknown and are not
  combined into model-fit or headroom claims.
- Compare bounded session observations through deterministic **Diagnose**
  findings without polling, persistence, automated repair, or provider action.
- Preview an allow-listed plain-text report and copy it only after an explicit
  native **Copy report** action.
- Run one fixed synthetic Ollama or LM Studio observation only after a per-run
  disclosure and explicit authorization. No inference runs automatically.

## Candidate platforms

- Ubuntu 24.04 LTS x86-64: direct-download `.deb`
- Windows 11 25H2 build 26200.7462 x64: unsigned NSIS installer

Other Linux distributions and architectures, other Windows environments, and
macOS are not currently claimed. AppImage, MSI, Store distribution, automatic
updates, and package repositories are not included.

## Important Windows trust notice

The Windows installer and application are unsigned. The publisher will appear
unknown, and SmartScreen, Smart App Control, antivirus software, or
organizational policy may warn about or block installation. Do not disable or
weaken a security control, install a trust certificate, or bypass policy. If
Windows provides no normal allowed continuation, do not install the preview on
that device.

Always obtain files from the official project release page and verify the exact
filename, byte size, and SHA-256 before installation. A matching checksum
detects a file change relative to the release page; it is not a publisher
signature.

## Privacy and behavior boundary

AI Engine Room has no account, telemetry, or application persistence. Bounded
observations remain in the current application process and reset when the
application restarts. Runtime and machine observations are not uploaded by the
application. Clipboard text is written only after an explicit **Copy report**
action, after which other applications may read the operating-system clipboard.

The application does not manage providers or models, guarantee model fit,
benchmark hardware, or prove compute placement. LM Studio may JIT-load a model
after an explicitly authorized observation and may later auto-unload it under
its own configuration.

## Candidate verification

- Source commit: [`a5482e9d51657a0cfb4471215a91750c5ba7db95`](https://github.com/gregweir/ai-engine-room/tree/a5482e9d51657a0cfb4471215a91750c5ba7db95)
- Deterministic CI: [all five jobs passed](https://github.com/gregweir/ai-engine-room/actions/runs/33309714136)
- Exact build and inspection record:
  [unsigned-preview candidate evidence](unsigned-preview-candidate-evidence.md)
- Exact physical acceptance: **passed**
- Independent final review: **passed**
- Publication approval: **not granted**

The completed operator sequence and bounded Linux process-close deviation are
recorded in the [candidate evidence](unsigned-preview-candidate-evidence.md).
The later, separately authorized publication sequence is described in the
[publication and rollback runbook](unsigned-preview-publication-runbook.md).

The proposed filenames, sizes, hashes, verification commands, trust warning,
licence links, and known limitations are in the
[release-page draft](unsigned-preview-release-page-draft.md). The draft checksum
manifest is [`unsigned-preview-SHA256SUMS.draft.txt`](unsigned-preview-SHA256SUMS.draft.txt).

## Documentation

- [Support and limitations](../../SUPPORT.md)
- [Privacy boundary](../../README.md#privacy-and-data-boundaries)
- [Security reporting](../../SECURITY.md)
- [Apache License 2.0](../../LICENSE)
- [Application NOTICE](../../NOTICE)
- [Third-party licences](../../THIRD-PARTY-LICENSES.txt)
- [Third-party source locations](../../THIRD-PARTY-SOURCES.txt)
