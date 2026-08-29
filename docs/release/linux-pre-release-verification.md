# Linux pre-release packaging verification

This checklist is for local, unsigned, unpublished verification artifacts only. It is not a public-release, signing, broad Linux-compatibility, or release-candidate process.

## Baseline and preparation

- Verified development and `.deb` packaging build baseline: Ubuntu 24.04 LTS x86_64. Final developer verification of each rebuilt package remains a separate gate.
- Begin from a clean tracked tree and record the exact source commit.
- Use the committed npm and Cargo lockfiles. Do not update dependencies or install new project dependencies.
- Record only OS distribution/version, architecture, Node, npm, Rust, Cargo, and Tauri CLI versions. Do not record usernames, hostnames, home paths, addresses, endpoints, credentials, tokens, or environment dumps.

## Deterministic gates

Run the four npm gates and four Cargo gates listed in CONTRIBUTING.md. No inference or running Ollama instance is required. Do not enable ignored live tests.

## Package build and inspection

```sh
npm run tauri build -- --bundles deb
```

The expected output is under `target/release/bundle/deb/`. Inspect the `.deb` read-only with locally available package tools, including package name, version, architecture, maintainer/publisher, description, dependencies, desktop entry, and installed icon paths.

Record the artifact filename, format, byte size, and SHA-256. A hash is an integrity record, not a signature. Do not stage or commit the artifact, `dist/`, `target/`, or raw logs.

## Developer-only manual checks

After independent review, the developer may separately inspect and install the `.deb`; launch it and verify identity, approved icon, dashboard, accessibility, layout, passive startup, no mock banner, no automatic inference, and no automatic clipboard write; remove it through the package manager; and confirm removal. Native **Copy report** is optional and explicit. No inference is required.

Do not publish, upload, sign, or externally share the artifact.

## Deferred AppImage format

AppImage is not part of the current Milestone 1K packaging-verification contract. The tested package exposed bundled WebKitGTK/GStreamer core libraries without making the installed host `appsink` plugin discoverable, so it failed developer runtime acceptance. Tauri's supported media-framework option would broadly bundle GStreamer plugins and helpers; the developer chose to defer that expanded packaging and licence/NOTICE surface to a separately approved design/research task. Do not enable `bundleMediaFramework`, add custom AppRun paths, or post-process plugins as part of 1K.

Milestone 1K performed a proportionate dependency/NOTICE review and identified no required NOTICE content. Before any public binary distribution or public release, bundled dependency licences and notices require a fuller publication review. This is not a comprehensive licence audit or legal-compliance certification.

## Evidence fields

Record the source commit, host/tool baseline, exact commands, deterministic results, artifact metadata and hashes, read-only inspection results, exclusions, and developer checks still pending in `docs/release/linux-pre-release-build-evidence.md`.
