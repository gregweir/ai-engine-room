# Linux pre-release packaging build evidence

This records local, unsigned, unpublished verification artifacts. It is evidence for the Ubuntu `.deb` packaging build baseline only, not proof of broad Linux compatibility, a signature, or a release candidate. AppImage is a rejected/deferred 1K experiment, not a current accepted package artifact.

## Current-source packaging-readiness result

The separately approved current-source packaging-readiness builds ran on
2026-08-29, and the final cleanup and independent review passed on 2026-08-30
for the following exact source identity:

- Source commit: `2293b336eaa314f4fd285737a8470a5b9abd151a`
- Source tree: `6f5df75b612f8b159f172a05766ab5bd2bec491f`
- Origin: `https://github.com/gregweir/ai-engine-room.git`
- Build baseline: Ubuntu 24.04 LTS, x86-64
- Node: v22.22.2
- npm: 10.9.7
- Rust: rustc 1.98.0
- Cargo: cargo 1.98.0
- Tauri CLI: 2.11.4

The entry gate passed on `main`: `HEAD`, local `main`, cached `origin/main`, and
live origin `main` matched the exact source commit; the checkout used the exact
sole origin above, was `0/0` ahead/behind, and had no tracked, staged, or
untracked changes. Hostname, username, private checkout path, address,
credential, token, and environment dumps are intentionally not recorded here.

The following current-source commands passed natively using the locked project
dependencies and existing toolchain:

```sh
npm ci
npm run check
npm run lint
npm run test:run
npm run build
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri build -- --bundles deb
```

Svelte/TypeScript checking, ESLint, the frontend production build, Rust
formatting, workspace tests, and strict Clippy passed. All 17 frontend test
files and 286 tests passed. Environment-dependent live tests remained ignored.
No provider was started, stopped, probed, or reconfigured, and no inference ran.
Tauri produced exactly one authorized `.deb`; no AppImage or other package
format was built.

### Current-source Debian artifact

- Filename: `AI Engine Room_0.1.0_amd64.deb`
- Format: Debian binary package, format 2.0
- Size: 4,276,214 bytes
- SHA-256: `f32531f9f22ea6dfff2fdc8a8b2631544fdfc5c9d0017bc0f012c0c5e3083d17`
- Package/version/architecture: `ai-engine-room` / `0.1.0` / `amd64`
- Maintainer: `Greg Weir`
- Dependencies: `libwebkit2gtk-4.1-0, libgtk-3-0`
- Homepage: `https://tartanleaf.com`

Read-only inspection confirmed the configured pre-release description, desktop
entry, `/usr/bin/aiengineroom`, and the expected 32, 128, and 256 pixel hicolor
icon paths. The artifact remained ignored under `target/` and local, unsigned,
and unpublished. It was unexecuted during build inspection.

Independent review confirmed the exact source identity, bounded command scope,
artifact metadata and hash, exclusion of other formats, and final clean `0/0`
Git state with no blocking discrepancy. This result establishes packaging
readiness only and did not transfer the older package's acceptance evidence to
this artifact.

### Current-source developer package acceptance

On 2026-08-30, after the packaging result and artifact identity had passed
independent review, the developer separately authorized and performed bounded
acceptance of the exact `.deb` identified above on the tested Ubuntu 24.04 LTS
x86-64 baseline.

- Installation through the package manager passed. `apt` selected the local
  `ai-engine-room` package at version `0.1.0`; its `_apt` local-file access
  warning did not prevent unpacking or configuration.
- Native launch passed. The installed `/usr/bin/aiengineroom` process opened one
  native window identified as `AI Engine Room`.
- The developer directly confirmed the approved icon and dashboard, absence of
  a mock or fixture banner, acceptable layout, navigation, focus, and
  accessibility presentation, and passive startup without automatic inference
  or clipboard write.
- Copy was not exercised. No provider was started, stopped, probed, or
  reconfigured, and no inference ran.
- The application closed successfully. Normal package-manager removal passed,
  and `dpkg-query` then reported that `ai-engine-room` was not installed.
- Independent read-only final verification found no installed package or
  application process, preserved the exact artifact size and SHA-256 above, and
  found the source checkout clean.

This is acceptance only for this exact local unsigned artifact and tested
baseline. It does not establish signing, publication, distribution,
release-candidate, public-release, broad-compatibility, privacy, security,
performance, provider, inference, accessibility-conformance, model-fit, or
headroom readiness.

## Current accepted packaging candidate

The sole current Milestone 1K packaging-verification format is `.deb`.

### Source and build host

- Source commit tested: `842884343c73484326e311ccadfef54983454cd3`
- OS: Ubuntu 24.04.4 LTS (Noble Numbat)
- Architecture: x86_64
- Node: v26.5.0
- npm: 11.17.0
- Rust: rustc 1.97.1 (8bab26f4f 2026-07-14)
- Cargo: cargo 1.97.1 (c980f4866 2026-06-30)
- Tauri CLI: 2.11.4
- Debian package inspection: dpkg-deb 1.22.6 (amd64)
- File identification: file 5.45

No username, hostname, home path, network address, endpoint, credential, token, or environment dump is recorded.

### Commands and deterministic results

The following gates completed successfully against the deb-only scope correction before it was committed, without enabling live tests or inference:

```sh
npm run check
npm run lint
npm run test:run
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Results: repository-readiness assertions passed; Svelte/TypeScript checks reported zero errors and warnings; ESLint passed; 13 frontend test files and 191 tests passed; the frontend production build passed; Rust formatting and clippy passed; all 254 deterministic application tests and doc tests passed. Six environment-dependent live tests remained ignored. No inference ran.

The developer also ran `cargo fmt --all --check` on the current tree after rebuilding the accepted package; it passed.

The developer rebuilt the package from the committed source above with:

```sh
npm run tauri build -- --bundles deb
```

Tauri completed exactly one bundle. No AppImage was built. Existing tooling was used; no project dependency or system package was installed.

### Debian package

- Filename: `AI Engine Room_0.1.0_amd64.deb`
- Format: Debian binary package, format 2.0
- Size: 4,165,422 bytes
- SHA-256: `629c189eedbc10ea784dab7f2f28ad39c3d58941bb182862044bddd24411aa71`
- Package: `ai-engine-room`
- Version: `0.1.0`
- Architecture: `amd64`
- Maintainer/developer metadata: `Greg Weir`
- Description: `Inspect local Ollama runtime and resource context.` followed by the configured pre-release long description
- Dependencies: `libwebkit2gtk-4.1-0, libgtk-3-0`
- Priority: `optional`
- Installed size: `14892`
- Homepage: `https://tartanleaf.com`

Read-only archive inspection found `/usr/bin/aiengineroom`, `usr/share/applications/AI Engine Room.desktop`, and 32, 128, and 256 pixel hicolor icon entries. The desktop entry contains `Name=AI Engine Room`, `Categories=Utility;`, `Icon=aiengineroom`, `Exec=aiengineroom`, `Terminal=false`, and the approved short description.

The archived production icon hashes matched the committed C2R1 Tauri inputs exactly:

| Archived icon                             | Committed input                  | SHA-256                                                            |
| ----------------------------------------- | -------------------------------- | ------------------------------------------------------------------ |
| `hicolor/32x32/apps/aiengineroom.png`     | `src-tauri/icons/32x32.png`      | `d9c0e8f5b882aef3f5e4646f31be9669f78843beea1f22ab40c5be89a5b727f6` |
| `hicolor/128x128/apps/aiengineroom.png`   | `src-tauri/icons/128x128.png`    | `52665f7621197093fb8f809999e87cf0437fcf912ae78f509df47ed377fb5448` |
| `hicolor/256x256@2/apps/aiengineroom.png` | `src-tauri/icons/128x128@2x.png` | `7e1e8269877c41bc67446aa863c7a88bd681cda82f4cacd3f3ac1ecca89bb76a` |

Developer verification of this exact artifact on the approved Ubuntu 24.04 LTS x86_64 baseline is complete:

- Install: passed using `sudo apt install "./target/release/bundle/deb/AI Engine Room_0.1.0_amd64.deb"`.
- Native launch: passed.
- Graphical/native behavior: the developer reports that it looks and runs properly.
- Removal: passed using `sudo apt remove ai-engine-room`.
- Post-removal state: `dpkg -s ai-engine-room` reported that package `ai-engine-room` is not installed and no information is available.

This is acceptance of the current local unsigned pre-release `.deb` on the narrow verified baseline. It is not evidence of production readiness, broad Linux compatibility, WCAG conformance, performance validation, or inference validation.

## Prior developer-tested Debian package

The preceding package was built from source commit `cba20e8448b8d7f6b8536f63bccee3ba1a1501ae`:

- Filename: `AI Engine Room_0.1.0_amd64.deb`
- Size: 4,165,416 bytes
- SHA-256: `d7d4ef99b1a89ce09ae861118ec25a747eb624c81b291b2c428fe6e87dd09521`
- Package/version/architecture: `ai-engine-room` / `0.1.0` / `amd64`
- Maintainer: `Greg Weir`
- Dependencies: `libwebkit2gtk-4.1-0, libgtk-3-0`

Developer verification results:

- Install: passed.
- Native launch and graphical behavior: passed; the application rendered the expected dashboard without a mock banner, presented the approved C2R1 branding acceptably, and otherwise looked and worked as expected.
- Removal: passed.

While installed, `dpkg -s ai-engine-room` reported status `install ok installed`, installed size `14892`, the expected description, and homepage `https://tartanleaf.com`. This evidence is useful but is not automatically transferred to the newly rebuilt package.

## Deferred/rejected 1K AppImage experiment

The preceding source commit also produced this historical diagnostic artifact:

- Filename: `AI Engine Room_0.1.0_amd64.AppImage`
- Size: 79,641,080 bytes
- SHA-256: `4b3002ccb0eb3308950321fe05834a6916a1162aeedd0889d885ce5bdc0d1efd`

The developer launched it and WebKitGTK reported `GStreamer element appsink not found. Please install it.` Host GStreamer 1.24.2 nevertheless successfully discovered the installed `appsink` element from `/usr/lib/x86_64-linux-gnu/gstreamer-1.0/libgstapp.so`. Read-only diagnosis established that the AppImage bundled WebKitGTK and GStreamer core/link libraries but no GStreamer plugin directory, `libgstapp.so`, or plugin scanner.

The supported Tauri `bundleMediaFramework` path would broadly bundle GStreamer plugins and helpers and materially expand the binary and licence/NOTICE review surface. The developer deferred AppImage rather than broadening Milestone 1K. Its filename and hash are retained only as historical diagnostic evidence; it is not a current accepted package artifact. No claim is made that AppImage is permanently unsupported or generally broken.

## Exclusions and pending gates

The current `.deb` and historical AppImage remain ignored/untracked under `target/release/bundle/`. The developer installed, launched, and removed the current `.deb` as recorded above. It was not signed, uploaded, published, or externally shared. This evidence-only update did not change package-manager or application state. No GitHub Release, hosted CI workflow, updater, Windows package, LM Studio integration, inference, clipboard operation, Save, or Share action was performed.

Milestone 1K performed only a proportionate dependency/NOTICE review and identified no required NOTICE content. AppImage media-framework bundling was not adopted, so no hypothetical GStreamer bundle review was added. Before any future public binary distribution or public release, bundled dependency licences and notices require a fuller publication review. This is not a comprehensive licence audit or legal-compliance certification.

The current `.deb` developer install, native launch, graphical/native behavior, removal, and post-removal absence gates are complete. The package remains local, unsigned, and unpublished. The developer retains the final push decision.
