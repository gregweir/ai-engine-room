# Contributing to AI Engine Room

Thank you for helping improve AI Engine Room. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Setup and checks

Use an Ubuntu 24.04 LTS x86_64 development environment for the verified baseline, the Tauri 2 Linux prerequisites, Node.js/npm, and Rust/Cargo. Install JavaScript dependencies from the committed lockfile with `npm ci`.

Before proposing a change, run the frontend gates (`npm run check`, `npm run lint`, `npm run test:run`, and `npm run build`) and the Rust gates (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo build --workspace`). Use Prettier for affected frontend, JSON, and Markdown files.

Keep deterministic tests independent of running Ollama, LM Studio, or llama.cpp instances and inference. Use artificial llama.cpp responses with synthetic served-model IDs only; never place real model paths or raw provider inventories in fixtures. Tests named `*_live.rs` are ignored by default; run live tests or inference only with explicit owner/user authorization. Never set live-inference variables merely to make a deterministic gate pass. The ignored LM Studio test additionally requires an owner-selected model and all four `AER_1L_*` acknowledgement variables documented in its source; never auto-select a model. Milestone 1O adds no llama.cpp live or inference test.

## Safe contributions

- Use artificial fixtures. Do not put secrets, credentials, private model lists, generated output, private machine data, endpoints, or sensitive logs in issues, tests, or fixtures. Describe verification environments with generic platform labels; do not publish hostnames, usernames, local filesystem paths, device models, exact hardware inventories, or private verification reports.
- Preserve keyboard operation, visible focus, semantic labels, enlarged-text usability, and narrow-layout behavior.
- Preserve the report privacy boundary: report data is allow-listed and sanitized, Copy is explicit and native-only, and clipboard authority remains write-text-only.
- Preserve explicit per-run inference authorization. Keep Ollama's undetermined execution boundary distinct from LM Studio's same-machine API scope and not-independently-verified compute placement. Disclose possible LM Studio JIT load/auto-unload effects.
- Proposals adding network access, persistence, telemetry, filesystem/dialog/shell permissions, broader clipboard access, or other native authority require design and security review before implementation.
- Do not commit `dist/`, Cargo `target/`, packages, raw logs, secrets, or machine-specific files.

Stage explicit files and review the staged diff before committing. Keep changes focused and explain tests, accessibility impact, and privacy/permission impact in the pull request.
