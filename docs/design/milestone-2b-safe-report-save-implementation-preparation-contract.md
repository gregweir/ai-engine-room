# Milestone 2B safe report-save implementation-preparation contract

## Status

Prepared locally on 2026-09-01 under developer authorization from exact public
baseline `1d4817c0346092333073aec58e4b3f082c8ceadb` (tree
`4ff5ae0fe858fc5c18f3ffdd4c2b71b183c2ed2f`). This preparation authorizes only
this documentation file and one local commit for separate review. It does not
authorize product implementation, a dependency or capability change, a native
dialog, a file operation, a push, packaging, publication, or release.

## Decision carried forward

The merged [Milestone 2A feasibility study](../research/safe-report-save-feasibility.md)
records **PREPARE, NOT ADOPT**. It found a concrete advantage over clipboard-only
copy but left no-clobber staging, cleanup, exact dependencies, controlled IPC
results, and native validation unresolved.

This contract resolves those preparation questions narrowly enough for a later
implementation commit to be reviewed. It does not adopt the feature and creates
no implementation authority by itself. If the exact implementation cannot
preserve every fail-closed condition below, deferral is required.

## Reviewed implementation baseline

| Field                           | Required value                                       |
| ------------------------------- | ---------------------------------------------------- |
| Repository                      | `gregweir/ai-engine-room`                            |
| Contract parent                 | `1d4817c0346092333073aec58e4b3f082c8ceadb`           |
| Proposed implementation branch  | `codex/safe-report-save-implementation`              |
| Existing report value           | Backend-authored, sanitized plain-text preview       |
| Existing frontend capability    | `core:default`, `clipboard-manager:allow-write-text` |
| Proposed final filename         | `ai-engine-room-report.txt`                          |
| Proposed maximum payload        | 1,048,576 UTF-8 bytes                                |
| Proposed additional Rust plugin | `tauri-plugin-dialog` exactly `2.7.2`                |
| Existing platform bindings      | `libc` `0.2.189`; `windows-sys` `0.61.2`             |

Before a later implementation branch is pushed, its exact commit, parent,
changed paths, `Cargo.lock` diff, dependency and licence diff, and executed
commands must be reported for developer approval. Approval to merge this
contract is not approval of that implementation commit.

## Allowed later implementation paths

A separately authorized implementation commit may modify only:

- root `Cargo.lock`;
- `src-tauri/Cargo.toml`;
- `src-tauri/src/lib.rs` and `src-tauri/src/commands.rs`;
- new `src-tauri/src/report_save.rs` and narrowly split modules beneath
  `src-tauri/src/report_save/`;
- `src/App.svelte` and `src/app.css`;
- `src/lib/types.ts`;
- `src/lib/datasource/index.ts`, `src/lib/datasource/tauri.ts`,
  `src/lib/datasource/mock.ts`, `src/lib/datasource/datasource.test.ts`, and
  `src/lib/fixtures/states.ts` only as needed for the preview response change;
- new `src/lib/reportSave.ts` and its deterministic test;
- `src/lib/components/ReportWorkspace.svelte` and its existing test;
- `src/App.test.ts`;
- generated third-party licence and source materials required by the normal
  repository licence process;
- `scripts/verify-target-license-coverage.mjs` and
  `scripts/repository-readiness.mjs` for deterministic dependency and boundary
  assertions; and
- this contract, `docs/design/architecture.md`, `docs/user-guide.md`, and
  `docs/roadmap.md` for exact implementation status.

No capability file, `package.json`, npm lockfile, Tauri configuration, workflow,
provider, inference path, report field, renderer, package configuration, live
test, release record, or unrelated source may change. If another path proves
necessary, preparation must stop for a contract amendment.

## Authoritative preview and generation boundary

`report_preview` remains the only report producer. It must still acquire,
allow-list, sanitize, and render the report independently of Overview history.
Its response may change from a bare string only to this closed shape:

- `text`: exactly the existing rendered preview; and
- `generation`: an opaque fixed-width lowercase hexadecimal string derived
  from a checked process-local monotonic counter.

The backend may retain only the current `{ generation, text }` pair in memory.
A new preview atomically replaces the previous pair. Neither value is written,
logged, included in diagnostics, or retained after process exit. Counter
overflow clears the retained preview and fails closed; it must not wrap or
reuse a generation.

The frontend displays `text` unchanged and treats `generation` as opaque. A
dedicated `save_report` command accepts only that generation string. It accepts
no report text, path, filename, overwrite flag, format, or arbitrary options
from the WebView.

At command entry the backend validates the bounded token, rejects an empty or
oversized preview, marks one save operation active, and clones the retained
bytes. Immediately after the dialog returns and before any staging file is
created, it validates that the same generation is still current. A Refresh or
replacement during the dialog therefore returns `preview_changed` and performs
no write. A backend RAII lease must clear the one-operation guard on every
handled return and unwind path.

## Native dialog boundary

The later implementation may add `tauri-plugin-dialog = "=2.7.2"` to the Rust
application only and register its Rust plugin. It may use the blocking Rust save
dialog only inside `tauri::async_runtime::spawn_blocking`; it must not block the
main thread.

The dialog must:

- be opened only by explicit activation of **Save report…** in the native
  **Report** workspace;
- be parented to the main window;
- use title **Save AI Engine Room report**;
- suggest `ai-engine-room-report.txt` without choosing or remembering a
  directory;
- offer only a **Plain text** `txt` filter; and
- represent a closed dialog as `cancelled`, without a file operation.

The Rust `FilePath` must convert to a local filesystem path. A URI or conversion
failure returns `invalid_destination`. If the selected name has no extension,
the backend appends `.txt`; an ASCII-case-insensitive `.txt` extension is
accepted and any other extension is rejected. The selected or normalized path
never crosses IPC, enters a retained state, or enters a log.

No `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs`, `tauri-plugin-fs`,
frontend path API, persisted scope, `dialog:*` permission, or `fs:*` permission
may be added. The capability must remain byte-for-byte unchanged. The dialog
plugin's frontend commands therefore remain unavailable to the WebView; only
the bounded application command can initiate this operation.

## Exact bytes and staging boundary

The final file contains exactly `text.as_bytes()`:

- valid UTF-8;
- no byte-order mark;
- no newline conversion;
- no added or removed trailing newline;
- no save time, path, filename, identifier, hash, signature, or metadata; and
- no read-back after saving.

The backend must never open the selected destination for writing. It creates a
new staging file in the selected destination's parent directory with
`OpenOptions::create_new(true)`. Candidate staging names use a fixed
`.ai-engine-room-report.tmp-` prefix plus checked process-local counter and
attempt values; creation retries are bounded to 64 collisions. A PID, final
report text, selected filename, username, hostname, provider, model, wall-clock
value, or machine identifier must not appear in a staging name.

An existing candidate name, including a dangling symbolic link, is only a
collision: the backend must not open, inspect, unlink, or replace it. It advances
the checked attempt counter and tries the next `create_new` candidate. Exhaustion
returns `unavailable` without changing any pre-existing entry.

On Unix the staging file is created with mode `0o600`, still subject to stricter
directory and umask policy. On Windows it inherits the selected directory's
access-control policy. Creation failure produces no destination file and is
mapped to a controlled result.

The complete retained bytes are written to the staging handle with `write_all`,
then `flush` and `sync_all` are required. The handle must be dropped before the
commit step. Any reported write, flush, or synchronization failure triggers
bounded staging cleanup and never falls back to writing the destination
directly.

## Race-safe no-clobber commit

The completed staging file is made visible at the destination through one
platform operation that cannot replace an existing name:

- **Ubuntu/Linux:** call `renameat2` with `RENAME_NOREPLACE` and no other flag.
  `EEXIST` is `destination_exists`; `ENOSYS`, `EINVAL`, `EOPNOTSUPP`, or `EXDEV`
  means the operation is unavailable on that kernel/filesystem. There is no
  fallback to `rename`, hard links, copy-and-delete, or a check-then-rename
  sequence. After success, synchronize the parent directory before returning
  `saved`.
- **Windows:** call `MoveFileExW` with a zero flag value. Neither
  `MOVEFILE_REPLACE_EXISTING`, `MOVEFILE_COPY_ALLOWED`, nor a delayed operation
  may be set.
  `ERROR_FILE_EXISTS` and `ERROR_ALREADY_EXISTS` are
  `destination_exists`. There is no fallback to `std::fs::rename`,
  `ReplaceFile`, copy-and-delete, or a preflight existence check.

The Linux call uses the already locked `libc` `0.2.189`. The Windows target
keeps `windows-sys` `0.61.2` and adds only the minimum
`Win32_Foundation` and `Win32_Storage_FileSystem` features alongside its
existing feature. Every unsafe block must state string termination, pointer
lifetime, flag, ownership, and error-retrieval invariants immediately beside
the call.

The later dependency diff is limited to the exact dialog plugin and transitive
lock additions it requires, plus feature activation on the already direct
platform bindings. `tempfile`, `atomic-write-file`, an async runtime, random
crate, serialization format, filesystem plugin, updater, shell, process, or
network dependency is not allowed.

## Cleanup and uncertainty model

Handled failure before a successful commit must close and remove the staging
file. If removal cannot be confirmed, return `cleanup_incomplete`; never claim
that no file remains. There is no recursive cleanup, directory scan, startup
cleanup, remembered directory, or deletion of any path other than the exact
staging path created by the current operation.

A process kill, operating-system crash, power loss, remote filesystem, or
filesystem implementation may still leave a partially or completely written
staging file. The implementation must not call the technique universally
atomic, crash-proof, private, or durable. Unsupported commit semantics defer
to `unavailable`. An operating-system result for which the backend cannot
prove whether the name move occurred returns `completion_uncertain`; it must
not inspect, overwrite, remove, or retry the destination.

After an unclassified commit error, the backend may attempt to remove only its
exact staging name. Confirmed removal returns `failed`; an absent staging name
or a parent-directory synchronization failure after a successful Linux move
returns `completion_uncertain`; an unremovable staging name returns
`cleanup_incomplete`. None of those outcomes permits a destination probe or an
automatic retry.

## Closed result model and exact user wording

The command returns one serialized enum only:

- `saved`;
- `cancelled`;
- `busy`;
- `preview_changed`;
- `invalid_destination`;
- `destination_exists`;
- `unavailable`;
- `failed`;
- `cleanup_incomplete`; or
- `completion_uncertain`.

No variant has a payload. Raw paths, chosen filenames, report text, tokens,
operating-system codes, error strings, or dependency errors never cross IPC or
enter application evidence.

The persistent explanation shown before the button must be:

> Save report creates a plain-text file that remains in the location you choose
> until you remove it. Other software may read or synchronize that file. AI
> Engine Room does not upload or remember it.

The controlled outcomes are:

| Result                 | Required visible wording                                                                                                   |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `saved`                | Report saved as a plain-text file.                                                                                         |
| `cancelled`            | Save cancelled. No report file was created.                                                                                |
| `busy`                 | A report save is already in progress.                                                                                      |
| `preview_changed`      | The report changed before saving. Review it and try again.                                                                 |
| `invalid_destination`  | Choose a filename ending in .txt.                                                                                          |
| `destination_exists`   | That file already exists. AI Engine Room did not replace it. Choose a different name.                                      |
| `unavailable`          | Saving is not available for that location. You can still copy the report.                                                  |
| `failed`               | Could not save the report. No completed report file was created.                                                           |
| `cleanup_incomplete`   | Saving did not complete, and AI Engine Room could not confirm removal of its temporary file. Check the location you chose. |
| `completion_uncertain` | The operating system did not confirm whether the report was saved. Check the location you chose before trying again.       |

`saved` and `cancelled` use a polite status announcement; failures and uncertain
outcomes use an alert. Colour is never the sole indicator. Focus returns to the
button, duplicate activation is disabled while pending, and Copy remains a
separate independently available action.

The wording must not say or imply private, anonymous, encrypted, authenticated,
verified, safe to publish, crash-proof, or guaranteed durable. The user guide
must explain that the chosen location may be synchronized, backed up, indexed,
or read by other software.

## Deterministic implementation checks

Ordinary tests and CI must not open a native dialog or create, inspect, move,
or delete a real file. Dialog, staging, synchronization, commit, and cleanup
are injected traits with artificial in-memory results. Production adapters
compile but are not invoked by tests.

The later implementation must deterministically establish:

- exact equality among the retained backend preview, visible preview, clipboard
  input, and staged byte slice;
- UTF-8, no BOM, unchanged LF and trailing newline, and the 1,048,576-byte
  bound;
- one retained generation, checked monotonic replacement, stale rejection
  before and after the dialog, overflow failure, and one-operation exclusion;
- cancellation, empty preview, invalid token, invalid extension, and dialog
  failure invoking no staging seam;
- bounded staging collisions, partial-write error, synchronization error,
  destination-exists race, unsupported commit, ambiguous commit, successful
  cleanup, and cleanup-incomplete states;
- no retry, overwrite, direct-write, read-back, logging, history, settings,
  provider, inference, network, or clipboard side effect;
- every result's exact plain-language message, keyboard activation, visible
  focus, pending suppression, normal/enlarged-text layout, status/alert role,
  and no accessibility violations;
- browser/mock mode exposing no active save control or Tauri invocation;
- the capability file remaining exactly `core:default` plus
  `clipboard-manager:allow-write-text`;
- absence of frontend dialog/filesystem dependencies and generic path arguments;
- exact allowed changed paths and dependency versions; and
- regenerated licence manifests plus existing Linux and Windows ephemeral
  package-licence checks.

Repository readiness must assert the production platform modules contain only
the reviewed no-replace calls and flags, while the command accepts only the
generation token. Source assertions complement tests; they do not prove native
filesystem behavior.

## Separately gated native validation

Passing implementation review does not authorize a dialog or file write. A
later validation contract must bind exact source and package hashes and request
one Ubuntu authorization and one Windows authorization separately. Each run
must use a newly created disposable directory and stop on any raw error, path
leak, overwrite, unexpected file, elevation request, or unbounded delay.

The bounded checks must cover cancellation, new `.txt` success, exact bytes and
SHA-256 equality with the visible preview, an existing sentinel destination,
stale-preview rejection, inaccessible or unsupported location handling,
keyboard/focus/status behavior, clipboard non-mutation, and cleanup of every
created report, sentinel, staging file, and disposable directory. A handled
failure must leave no staging residue; an abrupt-termination experiment is not
authorized by this contract.

No remote share, synchronized folder, provider, inference, network-observation
adapter, elevated context, package installation, publication, or release action
is part of those future checks unless separately contracted.

## Explicit exclusions

This contract and its proposed later implementation do not authorize:

- implementation, dependency resolution, a native dialog, or any file
  operation under the current preparation;
- overwrite, append, read-back, automatic save, autosave, remembered location,
  recent-files list, history, settings, diagnostics, telemetry, upload, send,
  Share, cloud integration, or directory scan;
- Markdown, HTML, JSON, PDF, archive, image, structured, encrypted, signed, or
  multi-file export;
- generic filesystem or dialog authority in the WebView;
- provider access, inference, network observation, elevation, helper, service,
  driver, firewall, packet, or payload work; or
- push, pull request, merge, package execution, native validation,
  publication, release, signing, or distribution work.

## Contract acceptance

This preparation is ready for local review only if:

- its parent is the exact merged feasibility baseline;
- only this documentation file changes;
- the no-clobber platform operations, unsupported cases, cleanup, and
  uncertainty states are explicit;
- exact allowed dependencies and paths are bounded;
- current clipboard-only product behavior and capability remain unchanged;
- abrupt-termination residue is disclosed rather than hidden by an atomicity
  claim;
- no product, dependency, capability, test, workflow, or package file changes;
  and
- documentation formatting and diff checks pass without product checks,
  providers, inference, packaging, or native file operations.

## Subsequent gates

After separate approval and push of this contract, preparing the exact local
implementation requires a new developer authorization. Pushing that local
implementation, running ordinary deterministic CI, merging it, and performing
each native platform validation remain separate gates. No gate implies the
next.

## Primary references

- [Tauri dialog plugin](https://v2.tauri.app/plugin/dialog/)
- [`FileDialogBuilder::blocking_save_file`](https://docs.rs/tauri-plugin-dialog/2.7.2/tauri_plugin_dialog/struct.FileDialogBuilder.html#method.blocking_save_file)
- [`tauri::async_runtime::spawn_blocking`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.spawn_blocking.html)
- [Linux `renameat2` and `RENAME_NOREPLACE`](https://man7.org/linux/man-pages/man2/renameat2.2.html)
- [Microsoft `MoveFileExW`](https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-movefileexw)
- [`windows-sys` `MoveFileExW`](https://docs.rs/windows-sys/0.61.2/windows_sys/Win32/Storage/FileSystem/fn.MoveFileExW.html)
- [Milestone 2A feasibility contract](milestone-2a-safe-report-save-feasibility-contract.md)
- [Milestone 2A feasibility study](../research/safe-report-save-feasibility.md)
- [Architecture](architecture.md)
- [User guide](../user-guide.md)
- [Roadmap](../roadmap.md)
