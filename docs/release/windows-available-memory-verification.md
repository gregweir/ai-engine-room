# Windows available-memory verification

This document records bounded owner-supplied evidence for Milestone 1Q. It is
evidence for one accepted source revision, one current-source unsigned NSIS
artifact, and one tested Windows baseline—not a broad Windows, provider, or
release-readiness claim.

## Evidence authority and baseline

- Evidence date: 2026-08-23.
- Accepted source commit: `7e85354ea23f363eb1781c4fa342fa1e950da3be`.
- Operating system: Windows 11 25H2, build 26200.7462.
- Architecture: x86-64 / AMD64.
- Scope: native deterministic verification, current-source NSIS packaging,
  real `os.ram.available` observation, and bounded owner package acceptance.

No username, hostname, private path, private address, environment dump, raw
Win32 error, provider inventory, generated output, or clipboard content is
recorded.

## Native deterministic verification

The following commands passed natively on the Windows baseline above against
the accepted source commit:

```powershell
npm run check
npm run lint
npm run test:run
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo build --workspace --locked
```

`npm run check` passed on the CRLF Windows checkout. The frontend suite passed
with 13 test files and 216 tests. `windows-sys` 0.61.2 compiled natively; the
`MEMORYSTATUSEX` binding compiled, `GlobalMemoryStatusEx` linked, and the
`cfg(windows)` production path built. The NSIS x64 release build also passed.
No ignored/live test, provider call, inference, or clipboard action was part of
this verification.

## Current-source NSIS artifact

- Filename: `AI Engine Room_0.1.0_x64-setup.exe`.
- Format/target: local Windows NSIS installer, x64.
- Size: 2,487,355 bytes.
- SHA-256: `B0B2F34A78483820A4281D7CB16F93D2092DD31C691FD0F761E9D991FFEF6A44`.
- Authenticode status: `NotSigned`.
- Visible product identity/version: AI Engine Room 0.1.0 passed where Windows
  exposed it.
- Approved icon: passed on the Windows surfaces actually observed.

This artifact was built from the accepted Milestone 1Q source. It is distinct
from the historical Milestone 1M artifact recorded in
`windows-pre-release-build-evidence.md`; neither artifact identity is evidence
for the other. The current artifact remains local, unsigned, and unpublished.

## Real available-memory observations

Windows `os.ram.available` reads only the operating system's reported
`MEMORYSTATUSEX.ullAvailPhys` value. The following readings were taken at
different times and are intentionally recorded as separate volatile
observations:

| Observation point | Observed value |
| --- | ---: |
| Native build verification | 13,175,230,464 bytes (13.18 GB) |
| Owner package acceptance | 12,962,058,240 bytes (12.96 GB) |
| Report-safe projection | `Available memory: available (12961325056 bytes)` |

The differing values are expected for a volatile operating-system observation
and are not a discrepancy. A successful zero remains a legitimate observation;
an API failure remains controlled and must not fabricate zero.

AI Engine Room's Windows available-memory observation has passed native
compilation, current-source unsigned NSIS packaging, and owner package
verification on Windows 11 25H2 build 26200.7462 x64. The value is the
operating system's reported available physical memory and is not claimed to be
numerically equivalent to Linux `MemAvailable`.

## Resource Context and Report

Resource Context passed with the observed value through the existing cautious
composition. It added no total/free-memory inference, threshold, pressure,
headroom, model-fit recommendation, causal claim, provider interpretation, or
compute-placement claim.

The Report displayed the report-safe observation above through the existing
allow-listed `os.ram.available` projection and sanitizer. No new report field,
platform metadata, provider/model state, raw API error, or private-machine data
was added. No explicit clipboard Copy acceptance is claimed; absence of an
automatic clipboard write was observed.

## Owner package acceptance

Against the exact artifact identity above, the owner reported that:

- normal per-user installation and native launch passed;
- visible identity/version and the approved icon passed on the surfaces
  actually observed;
- production/no-mock presentation passed;
- real Windows **Available memory** and bounded Resource Context passed;
- the Report boundary passed;
- startup, navigation, and one explicit **Refresh** remained passive, with no
  inference or provider/model-management action;
- no automatic clipboard write was observed;
- keyboard navigation, visible focus, and disclosure controls passed;
- narrow-window presentation passed;
- enlarged-text presentation passed using **Windows Settings > Accessibility >
  Text size** at 225%;
- normal uninstall passed, and the application/package entry was absent after
  removal.

These are bounded owner acceptance observations, not a WCAG conformance claim.

## Optional checks not performed

- No explicit clipboard Copy acceptance is claimed.
- No live provider or inference verification was performed for Milestone 1Q.
- No live provider-functionality check was performed, and no provider/model
  management action was performed.
- No signing, trust-chain, updater, upload, publication, or public-distribution
  check was performed.
- No other Windows version, build, architecture, or machine was tested by this
  evidence record.

## Limitations and non-claims

- Other Windows versions, builds, architectures, and machines are not
  established.
- Milestone 1Q does not establish Windows provider functionality or Windows
  inference.
- Linux `MemAvailable` and Windows available physical memory are
  platform-native observations and are not claimed to be numerically
  equivalent.
- No memory-pressure, capacity, model-fit, or headroom recommendation is made.
- No CPU, GPU, NPU, process-memory, swap, total-memory, or broader Windows
  telemetry is established.
- No compute placement is verified.
- The artifact is unsigned.
- This evidence does not make AI Engine Room production-ready, a release
  candidate, publicly released, or publicly available.
