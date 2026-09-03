# Changelog

Notable project changes will be recorded here. AI Engine Room is currently work-in-progress pre-release software. The first public unsigned prerelease is `v0.1.0-preview.1`; no stable or general 0.1.0 release has been created.

## Unreleased

### Added

- Native **Save report…** for explicitly saving the exact visible report-safe
  preview as a UTF-8 plain-text file. The backend-owned flow does not replace
  an existing destination and does not expose general frontend filesystem
  permissions.

### Validation

- Accepted bounded Ubuntu and Windows native evidence for cancellation,
  clipboard copy, new-file saving, and no-clobber behavior on the exact tested
  candidates. The Windows new-file evidence is content-only because the
  operator selected the parent temporary location, and its launcher timer
  expired before normal closure. Cleanup passed; no additional native run is
  required. Native stale-preview behavior remains supported only by
  deterministic coverage.

These changes are in current source and are not included in
`v0.1.0-preview.1`.

## 0.1.0-preview.1 - 2026-08-30

### Added

- Ubuntu-focused native dashboard for Ollama status, catalogue, loaded models, and resource context.
- Explicitly authorized, bounded synthetic inference observations with session-only history and comparison.
- LM Studio native REST v1 detection, model/loaded-instance inventory, and explicitly authorized stateless observations; the first bounded developer-authorized live integration test passed on the verified Ubuntu development environment.
- Provider-labelled selection and history with cross-provider comparison blocked.
- Allow-listed and sanitized report preview with explicit native write-only clipboard Copy.
- Public-project documentation, contribution policies, and local unsigned `.deb` packaging-verification configuration.
- Local unsigned Windows NSIS packaging baseline; developer acceptance passed on the tested Windows 11 25H2 build 26200.7462 x64 environment without establishing broad Windows or provider compatibility.
- Bounded passive Ollama and LM Studio verification and coexistence passed on the tested Windows 11 25H2 x64 baseline without inference or provider/model management.
- Passive traditional single-model llama.cpp detection and validated served-model ID display at fixed numeric loopback; bounded developer verification passed on the tested Ubuntu 24.04 LTS x86_64 baseline, while llama.cpp inference remains deferred.
- Windows available-memory observation through the existing `os.ram.available` metric path; native compilation, current-source unsigned NSIS packaging, and bounded developer package verification passed on the tested Windows 11 25H2 build 26200.7462 x64 baseline.
- Session-only explicit change history and a fifth Diagnose workspace with deterministic Observation → Meaning → Safe next check findings over already-acquired controlled state.
- Bounded total-memory and native-CPU-architecture machine context for Ubuntu and Windows; both remain outside the copied Report, and bounded native compile, value, and presentation verification passed on the tested Windows 11 25H2 x64 environment, including developer-established 225% Text size.
- Deterministic Ubuntu and Windows CI with ephemeral `.deb` and NSIS licence-payload builds, but without live provider access, inference, signing, release, or artifact publication.

### Changed

- Added a compact session-only Available-memory observation sequence and made the allow-listed plain-text Report substantially more human-readable without adding monitoring, persistence, or new report fields.
- Hardened fixed-loopback provider transports against redirect and proxy escape.
- Deferred AppImage after the tested package failed WebKitGTK/GStreamer plugin discovery on the verified Ubuntu baseline; `.deb` is the sole current Milestone 1K packaging-verification format.
