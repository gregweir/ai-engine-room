# Windows pre-release packaging verification

This checklist covers a local, unsigned, unpublished NSIS pre-release for Windows 11 x86-64. It does not establish broad Windows support, signing, publication, or owner package acceptance.

Windows available-memory support has passed native compilation, current-source unsigned NSIS packaging, and owner package verification on the tested Windows baseline. That bounded result is recorded in [Windows available-memory verification](windows-available-memory-verification.md). The historical accepted installer identified below predates the implementation and remains separate evidence.

## Baseline and preparation

- Tested build environment: Windows 11 25H2, build 26200.7462, x86-64.
- Begin from a clean tracked tree and record the exact source commit.
- Use the committed npm and Cargo lockfiles. Do not update or add project dependencies.
- Record only the OS version/build, architecture, tool versions, source commit, commands, and artifact metadata. Do not record usernames, hostnames, private paths, addresses, secrets, or environment dumps.

## Deterministic gates

Run `npm run check`, `npm run lint`, `npm run test:run`, `npm run build`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --workspace`, and `cargo build --workspace --locked` from native PowerShell. Do not enable ignored/live tests, run inference, contact providers, or exercise the real clipboard.

## Package build and inspection

```powershell
npm run tauri build -- --bundles nsis
```

The Windows-specific Tauri configuration selects NSIS only; the base configuration remains `.deb` only for Linux. Keep the installer under `target/release/bundle/nsis/` local and untracked. Record its filename, byte size, SHA-256, inspectable PE/version metadata, approved icon evidence, and unsigned state. Do not execute, install, sign, upload, publish, or build MSI.

Normal Tauri NSIS bundling may download its bundler helper/tooling; that is not a project dependency.

## Owner-only package acceptance

The historical Milestone 1M acceptance applies only to `AI Engine Room_0.1.0_x64-setup.exe`, size 2,468,432 bytes, SHA-256 `49f1a34eb5ca9a3ca9a59cdaf78af9c5478893e312d96888931dcfa0c5396de4`, with Authenticode status `NotSigned`. A later artifact must have separately recorded evidence; the distinct current-source Milestone 1Q artifact is recorded in [Windows available-memory verification](windows-available-memory-verification.md). Do not treat either artifact identity as evidence for the other.

An unsigned installer may show **Unknown publisher**, SmartScreen, or organization/security-policy warnings. Do not disable SmartScreen, bypass policy, lower security settings, or use undocumented bypasses. If Windows or organizational policy blocks execution and there is no normal owner-approved path to continue, stop and record the policy block; do not characterize a warning as harmless or a false positive.

Record pass/fail for every required observation against the exact installer hash:

1. Recalculate and confirm the filename, byte size, SHA-256, and `NotSigned` Authenticode status, then record any Windows warning or security-policy outcome.
2. Install through the normal Windows installer flow and launch AI Engine Room natively. Verify the product identity and version wherever Windows exposes them.
3. Verify the approved C2R1 icon on the surfaces Windows actually exposes, such as the installer, Start menu, taskbar, Alt-Tab or application window, and Installed apps/uninstall entry. A surface that Windows does not expose is not a failure.
4. Confirm that no mock/fixture banner or fixture-only browser/mock presentation appears and that this is the native application.
5. For the historical artifact identified above, **Available memory** is expected to render the controlled unavailable state because that artifact predates the implementation. For a later artifact built from the current source, confirm **Available memory** renders the operating-system-reported byte value, including a legitimate zero if the API succeeds; an API failure must remain controlled and must not fabricate zero. Confirm Resource Context adds no total/free/headroom inference and does not claim numerical equivalence with Linux `MemAvailable`.
6. Confirm startup and navigation cause no automatic inference, provider/model action, or clipboard write. Ollama and LM Studio may be absent or unavailable; this alone is not a package-test failure and neither provider needs to be started.
7. Check keyboard navigation through all four workspaces, visible focus, disclosure controls, narrow-window layout, and enlarged text/zoom. Record any clipping, overlap, unreadable presentation, or unreachable control. These checks are not a WCAG conformance claim.
8. Close the application, uninstall it through normal Windows application removal, confirm the application/package entry is absent afterward, and record removal pass/fail. Do not invent filesystem-cleanup requirements.

Provider checks are optional. If the owner later chooses to run Ollama or LM Studio for passive detection, keep the check read-only/passive unless a separate explicit inference authorization is given. Provider availability is not an installer-acceptance prerequisite.

**Copy report** is optional. If tested, perform one explicit manual Copy action and compare the resulting plain text with the visible report-safe preview. No clipboard read permission is added, no automatic copy is permitted, and Copy is not required for package acceptance if the owner chooses not to use the clipboard.

Inference is **not required** for Milestone 1M package acceptance. Do not run Ollama or LM Studio inference for this procedure.

Record no private paths, username, hostname, IP address, raw environment dump, generated model output, or clipboard content beyond the controlled comparison if Copy is tested.

Windows available-memory support has current-source verification for the exact artifact and tested baseline recorded in [Windows available-memory verification](windows-available-memory-verification.md). This checklist remains the reusable procedure for future artifacts. Do not claim that the historical Milestone 1M package contains or verifies the metric, and do not carry the Milestone 1Q result forward to a different source revision or artifact without new evidence.
