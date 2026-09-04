# Changelog

Notable project changes are recorded here. The first stable release is 0.1.1;
0.1.0 was published only as the historical `v0.1.0-preview.1` prerelease.

## Unreleased

## 0.1.1 - 2026-09-03

### Added

- Native **Save report…** for explicitly saving the exact visible report-safe
  preview as a UTF-8 plain-text file. The backend-owned flow does not replace
  an existing destination and does not expose general frontend filesystem
  permissions.

### Validation

- Accepted exactly one bounded physical run of the exact 0.1.1 `.deb` and NSIS
  candidates on the supported Ubuntu and Windows baselines. Native launch,
  core navigation, Refresh, Report, exact clipboard copy, new-file saving, and
  existing-file no-clobber behavior passed on both. Each application was closed
  normally; disposable report and clipboard artifacts were cleaned. The
  operator retained both installations after normal removal had passed in
  earlier candidate testing. Native stale-preview behavior remains supported
  only by deterministic coverage.

### Changed

- Established 0.1.1 as the first stable release identity without adding new
  provider, inference, network, persistence, or updater behavior.
- Replaced current-facing preview wording with stable-release support and
  unsigned-install guidance. Historical preview evidence remains unchanged.

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
