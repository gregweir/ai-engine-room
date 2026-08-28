# Windows pre-release packaging build evidence

This records a local, unsigned, unpublished NSIS pre-release build and its bounded owner package acceptance. It does not establish broad Windows support.

## Source and build environment

- Source commit: `a539fd49c618685d1183b78b6ccac2a816c0487f`
- OS: Windows 11 25H2, build 26200.7462
- Architecture: x86-64 / AMD64
- WebView2 Runtime: 151.0.4129.93
- Visual Studio Community component: `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`
- Node: v24.19.0
- npm: 11.17.0
- Rust: rustc 1.97.1 (8bab26f4f 2026-07-14)
- Cargo: cargo 1.97.1 (c980f4866 2026-06-30)
- rustup: 1.29.0
- Rust toolchain/target: `stable-x86_64-pc-windows-msvc` / `x86_64-pc-windows-msvc`
- Git: 2.53.0.windows.1
- Tauri CLI: 2.11.4

No username, hostname, private path, address, credential, token, or environment dump is recorded.

## Deterministic verification

The following commands passed against the source commit above:

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

Repository readiness passed. Svelte/TypeScript reported zero errors and warnings. ESLint passed. All 13 frontend test files and 196 tests passed. The frontend production build passed. Rust formatting, clippy, tests, and both workspace builds passed. The Windows-specific production snapshot test confirmed that the non-Linux snapshot contains no fabricated metrics. Environment-dependent provider/inference tests remained ignored; no ignored test, inference, provider call, or real clipboard operation ran.

## NSIS build

Exact command:

```powershell
npm run tauri build -- --bundles nsis
```

Tauri built the x64 application and exactly one NSIS bundle. During the first packaging attempt, the sandbox blocked the normal helper download. With network access authorized, Tauri downloaded and hash-validated NSIS 3.11 and `nsis_tauri_utils` 0.5.3. These are bundler helpers, not project dependencies. The final rebuild reused the helper tooling. No MSI, signing, updater artifact, CI/CD, install, upload, publication, or project dependency change occurred.

## Artifact and read-only inspection

- Filename: `AI Engine Room_0.1.0_x64-setup.exe`
- Format/target: Windows PE NSIS installer, x64
- Size: 2,468,432 bytes
- SHA-256: `49f1a34eb5ca9a3ca9a59cdaf78af9c5478893e312d96888931dcfa0c5396de4`
- Installer file description/product: `AI Engine Room`
- Installer product/file version: `0.1.0`
- Authenticode status: `NotSigned`; signer: none
- Packaged application product/file description: `AI Engine Room`
- Packaged application product version: `0.1.0`
- Packaged application company: `Tartanleaf.com Inc.`
- Packaged application Authenticode status: `NotSigned`

The committed C2R1 `icon.ico` was not modified. Its SHA-256 is `9edc4eefd796a55d08f945c5a911461f0c5fcf61ded9ab7665bd4f51f9be3a7f`; deterministic readiness inspection confirms the ordered 32, 16, 24, 48, 64, and 256 pixel layers. The installer was not executed during build inspection; owner graphical/icon acceptance is recorded below.

## Owner package acceptance

The owner tested the exact reviewed candidate on Windows 11 25H2, build 26200.7462, x86-64 / AMD64:

- Filename: `AI Engine Room_0.1.0_x64-setup.exe`
- Size: 2,468,432 bytes
- SHA-256: `49f1a34eb5ca9a3ca9a59cdaf78af9c5478893e312d96888931dcfa0c5396de4`
- Authenticode status: `NotSigned`

The owner reported that the installer produced no Windows warning during this installation, installation succeeded, and the native application launched successfully. Visible identity/version passed where Windows exposed it, and the approved C2R1 icon passed on the relevant Windows surfaces actually exposed during the check. No mock/fixture presentation appeared.

No Ollama or LM Studio runtime was available during the check. Those controlled unavailable provider states are acceptable under the package contract and are not installation failures; provider availability is not an acceptance prerequisite. This package acceptance does not establish Ollama or LM Studio provider functionality on Windows.

The owner reported that **Available memory** displayed “Unavailable — no value reported,” Resource Context remained present without inventing a Windows memory value, and **Current observed inference state** displayed “No observation run this session.” Passive startup and navigation passed: no automatic inference, provider/model action, or clipboard write occurred. No inference was run.

The supplied native Overview screenshot supports only these visible observations: AI Engine Room native presentation; the Overview workspace; Ollama could not be reached; LM Studio was Unreachable/could not be reached; Available memory showed the controlled unavailable state; catalogue and currently loaded models were unavailable; no observation had run; Resource Context was present; no mock/fixture banner was visible; and the supplied normal-size view had no obvious clipping or overlap. The screenshot is not committed.

The owner separately confirmed that keyboard navigation through all four workspaces, visible focus, disclosure-control keyboard behavior, narrow-window layout, and enlarged-text/zoom layout all passed against the same exact artifact/hash. These observations are owner-supplied evidence and are not inferred from the screenshot. They are bounded graphical/accessibility acceptance observations, not a WCAG conformance claim.

The owner reported that uninstall passed and explicitly confirmed the application/package entry was absent afterward.

All required observations in the durable Milestone 1M owner checklist are complete for this exact unsigned artifact on Windows 11 25H2, build 26200.7462, x86-64 / AMD64. **Copy report** was not tested and remains optional. Passive detection with a running Ollama or LM Studio was not tested and remains optional. Inference is not required and was not performed.

AI Engine Room's local unsigned NSIS pre-release package has passed owner install, native launch, graphical/accessibility, passive-behavior, and removal verification on the tested Windows 11 25H2 build 26200.7462 x64 baseline. This is not evidence for other Windows versions, architectures, machines, provider configurations, broad Windows compatibility, provider functionality on Windows, signed/trusted publisher status, Store readiness, compute placement, production readiness, or release-candidate status. The artifact remains local, unsigned, and untracked.
