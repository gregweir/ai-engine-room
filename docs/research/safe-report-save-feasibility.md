# Safe plain-text report-save feasibility

## Status and disposition

Documentation-only Milestone 2A study completed on 2026-09-01 from exact
public baseline `1e08567104d57f58484030b0c2a496a2381dc579` (tree
`2678a319022dd41dcd77db2f55517d276b26a7a2`).

**Disposition: PREPARE, NOT ADOPT.** A narrow user-initiated plain-text save is
plausible and has a concrete advantage over clipboard-only copy, but no
implementation is authorized. A later source-bound implementation-preparation
contract must resolve the no-clobber staging and cleanup mechanism, dependency
set, controlled result model, and native validation procedure before product
code may change.

No file dialog, report write, provider, inference, package, publication, or
release action was performed during this study.

## Current product boundary

The current repository establishes these facts:

| Boundary             | Current evidence                                                                                                                                                                                       |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Report production    | `report_preview` independently acquires a snapshot, projects only the report allow-list, sanitizes it, and renders one Rust `String`.                                                                  |
| Visible value        | The frontend retains and displays that returned string as the report-safe preview.                                                                                                                     |
| Clipboard            | One explicit **Copy report** action passes exactly the displayed string to the narrowly permissioned clipboard plugin.                                                                                 |
| Native exposure      | The capability grants `core:default` and `clipboard-manager:allow-write-text` only. There is no dialog or filesystem plugin, capability, generic filesystem command, save path, or report persistence. |
| Current user promise | The user guide says AI Engine Room does not save, upload, send, read back, persist, or clear the report.                                                                                               |

The relevant tracked sources are
[`commands.rs`](../../src-tauri/src/commands.rs),
[`lib.rs`](../../src-tauri/src/lib.rs),
[`default.json`](../../src-tauri/capabilities/default.json),
[`App.svelte`](../../src/App.svelte),
[`ReportWorkspace.svelte`](../../src/lib/components/ReportWorkspace.svelte), and
[`reportClipboard.ts`](../../src/lib/reportClipboard.ts).

## User value and limit

A save action would let a person deliberately keep the already reviewed report
without first placing it on a clipboard that other applications may read or
replace. That is a concrete functional difference, although no non-specialist
participant evidence establishes demand or wording comprehension.

Saving also creates durable data. The chosen folder may be synchronized,
backed up, indexed, shared, or read by software outside AI Engine Room. A saved
report is minimized, not anonymous, confidential, encrypted, authenticated, or
safe to publish. The feature must explain that distinction before opening a
dialog.

## Primary-source findings

Primary documentation was reviewed on 2026-09-01.

| Finding                                                                   | Source and implication                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Native save dialogs are available on the supported desktop platforms      | Tauri's [dialog plugin](https://v2.tauri.app/plugin/dialog/) lists Windows and Linux support, supplies Rust and JavaScript APIs, and requires Rust 1.77.2. That matches the repository's declared Rust minimum, but an exact locked dependency set and build remain unproved.                                                                                                                                                                   |
| Cancellation is representable without a write                             | The Tauri Rust API returns `Option<FilePath>` and documents `None` when the user closes the dialog. The non-blocking API is intended for main-thread use; the blocking API must not run on the main thread.                                                                                                                                                                                                                                     |
| Frontend dialog scope is broader than needed                              | Tauri's [JavaScript dialog reference](https://v2.tauri.app/reference/javascript/dialog/#save) says a selected path is added to filesystem and asset-protocol scopes and recommends a dedicated command when security is more important than ease of use. This supports a Rust-owned application command instead of frontend dialog and filesystem APIs.                                                                                         |
| Frontend filesystem writes require explicit commands and path scopes      | Tauri's [filesystem plugin](https://v2.tauri.app/plugin/file-system/) and [plugin-permission guide](https://v2.tauri.app/learn/security/using-plugin-permissions/) expose separate write commands and scopes. The current application needs neither generic frontend file access nor a home/documents wildcard.                                                                                                                                 |
| The WebView-to-Rust boundary must remain explicit                         | Tauri's [security model](https://v2.tauri.app/security/) treats frontend and Rust core as separate trust groups joined by IPC. Its [capability documentation](https://v2.tauri.app/security/capabilities/) notes that registered application commands are frontend-accessible by default unless an app manifest narrows them. A save command therefore needs its own strict argument and result model even if no plugin permission is granted.  |
| The current preview already has a stable encoding basis                   | Rust documents [`String`](https://doc.rust-lang.org/std/string/struct.String.html) as valid UTF-8. Writing the existing string bytes unchanged can therefore define UTF-8 output without a byte-order mark or transcoding.                                                                                                                                                                                                                      |
| A direct new-file open can prevent clobber but not partial target content | Rust [`OpenOptions::create_new`](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new) atomically fails when the destination or a dangling symlink already exists. Rust [`Write::write_all`](https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all) may perform multiple writes before returning an error, so writing directly to the selected destination can leave a partial target after a later failure. |
| Standard rename is not a complete cross-platform no-clobber design        | Rust's [`fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html) maps to different Unix and Windows facilities, can fail across filesystems, and does not provide a portable no-replace contract. A check-then-rename sequence would also create a race.                                                                                                                                                                                  |
| A credible no-clobber staging helper still has documented limitations     | [`tempfile::NamedTempFile::persist_noclobber`](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html#method.persist_noclobber) promises not to overwrite the target, but does not guarantee atomicity on every platform and may leave the original temporary link after a crash or cleanup failure. It is not currently a dependency and is not pre-approved by this study.                                                        |
| Atomic replacement is not the same as no-clobber creation                 | [`atomic-write-file`](https://docs.rs/atomic-write-file/latest/atomic_write_file/) keeps existing contents visible until commit, but its [open options](https://docs.rs/atomic-write-file/latest/atomic_write_file/struct.OpenOptions.html) intentionally do not provide `create_new`. It does not by itself meet the proposed first-slice no-overwrite boundary.                                                                               |
| Native overwrite prompts differ by backend and are not application proof  | Microsoft's common save dialog uses an overwrite prompt by default, and GTK 4 says save-mode choosers handle overwrite confirmation. The [XDG FileChooser portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html) specifies returned save URIs but not an application-verifiable overwrite-consent token. AI Engine Room must not infer portable overwrite authority from a returned path.       |

## Options considered

| Option                                                        | Assessment                                                                                                                                                                                                                                                                                                                                              |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| JavaScript dialog plus `tauri-plugin-fs`                      | **Reject.** It would expose dialog and filesystem commands to the WebView, dynamically scope the chosen path, and require additional frontend dependencies and capabilities when a dedicated Rust command can be narrower.                                                                                                                              |
| Rust dialog plus direct `create_new` write                    | **Reject as incomplete.** It prevents overwriting but can expose a partial destination if a later write or synchronization step fails. Best-effort deletion is not a fail-closed guarantee.                                                                                                                                                             |
| Rust dialog plus atomic replacement helper                    | **Reject for the first slice.** Replacement conflicts with the proposed no-overwrite boundary, and dialog confirmation is not a portable consent token available to the command.                                                                                                                                                                        |
| Rust dialog plus same-directory staging and no-clobber commit | **Prepare for exact review.** This is the narrowest credible design: complete and synchronize hidden staging content, then make the target visible only through a no-clobber commit. The later contract must select and audit the mechanism, report cleanup failures, and defer implementation if its supported-filesystem guarantees are insufficient. |

## Proposed later architecture

The later implementation-preparation contract may propose only this shape:

1. `report_preview` returns the existing visible text plus one opaque, bounded
   in-memory generation token. The backend retains at most the current preview
   generation, replaces it on Refresh, and never persists or logs it.
2. The **Save report** control is available only in the native application and
   only when a non-empty current preview is visible. It explains before the
   action that the file will persist and may be read or synchronized by other
   software.
3. One dedicated Rust command accepts only the current generation token. It
   accepts neither a path nor arbitrary report text from the frontend.
4. The command verifies the token before opening a backend-owned Tauri save
   dialog. A stale token returns a controlled `preview_changed` result without
   opening a dialog or writing.
5. The dialog suggests `ai-engine-room-report.txt`, offers only a plain-text
   `.txt` filter, and sets no remembered or identifying default directory.
6. The backend writes exactly the retained preview's UTF-8 bytes with no BOM,
   newline conversion, added metadata, timestamp, identifier, or trailing
   content.
7. The first slice does not replace an existing destination. A race-safe
   no-clobber commit must return `destination_exists`; the user may explicitly
   choose another name in a new action.
8. Staging occurs in the destination directory so commit cannot cross a
   filesystem. Handled failures must remove staging content or return a
   controlled `cleanup_incomplete` state; raw paths and operating-system errors
   never cross IPC or enter logs.
9. The command returns only a closed result such as `saved`, `cancelled`,
   `preview_changed`, `destination_exists`, `unavailable`, `failed`, or
   `cleanup_incomplete`. It does not return the selected path.
10. The frontend gains no `dialog:*` or `fs:*` permission and no dialog or
    filesystem npm package. Clipboard copy remains separate and unchanged.

This is a feasibility shape, not an implementation specification. In
particular, the token type, staging helper, dependency versions, synchronization
sequence, and cleanup proof remain for the next contract.

## Format disposition

The smallest coherent file format is:

- exactly the already visible report preview bytes;
- UTF-8 with no BOM;
- the renderer's existing LF characters, with no platform newline conversion;
- one `.txt` file with suggested name `ai-engine-room-report.txt`; and
- no embedded path, save time, filename, hostname, provider/model detail, hash,
  signature, or additional metadata.

`text/plain; charset=utf-8` is a documentation description, not an embedded
file attribute. Markdown, HTML, JSON, PDF, encrypted, signed, structured, and
multi-file exports remain outside scope.

## Controlled behavior and wording

| Event                                                      | Required outcome                                                                                                                       |
| ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Empty or stale preview                                     | Disable the control or return a controlled state; do not open a dialog.                                                                |
| User cancels                                               | Return `cancelled`; create no file; do not present cancellation as an error.                                                           |
| New destination succeeds                                   | Return `saved`; announce success with a non-colour status; do not display or retain the path.                                          |
| Destination already exists or appears during the operation | Do not replace it; return `destination_exists`; explain that the user can choose a different name.                                     |
| Dialog unavailable                                         | Return `unavailable`; keep clipboard copy independently available.                                                                     |
| Write, synchronization, or commit fails                    | Do not expose raw details; return `failed` only after confirmed cleanup.                                                               |
| Staging cleanup cannot be confirmed                        | Return `cleanup_incomplete`; make no success claim. Later native acceptance must define safe user guidance without retaining the path. |

The user-facing explanation must say that the saved report remains on the
chosen storage until the user removes it and that AI Engine Room does not
upload or remember it. It must not call the file private, anonymous, secure, or
verified.

## Dependency and package consequences

A Rust-owned dialog would add `tauri-plugin-dialog` and its locked transitive
dependencies. The official plugin's stated Rust minimum matches the repository,
but compatibility, licence coverage, binary/package payload, and package size
must be established from the exact later lockfile.

The no-clobber staging mechanism may require another narrowly reviewed Rust
dependency or bounded platform code. Neither `tempfile` nor
`atomic-write-file` is currently locked. The later contract must compare the
exact dependency graph, licences, sources, minimum Rust version, supported
filesystems, crash residues, and cleanup behavior. Generated third-party
notices and Linux/Windows package-licence checks must remain deterministic.

No JavaScript dialog/filesystem package, `tauri-plugin-fs`, broad filesystem
scope, persisted scope, updater, shell, process, or network dependency is
justified.

## Deterministic verification plan

A later implementation may use injected dialog and staging traits so ordinary
tests perform no native dialog or real file write. Required deterministic
coverage would include:

- exact equality among the retained backend preview, visible preview, and bytes
  offered to the staging boundary;
- UTF-8, no BOM, LF preservation, filename, extension, and byte-size bounds;
- one-entry generation replacement and stale-token rejection;
- cancellation and precondition failures invoking no writer;
- new-file success, destination-exists race, partial-write error,
  synchronization error, commit error, cleanup success, and cleanup-incomplete
  states;
- controlled errors containing no path, filename chosen by the user, raw
  operating-system text, report payload, or internal identifier;
- frontend keyboard operation, visible focus, pending-state suppression,
  enlarged-text layout, and `role=status`/`role=alert` behavior;
- browser/mock mode exposing no active save control;
- a capability assertion that no `dialog:*` or `fs:*` permission is granted;
- a source assertion that no frontend dialog/filesystem package is present;
  and
- regenerated licence manifests plus existing Linux and Windows ephemeral
  package-licence checks for the exact locked dependencies.

## Later native checks

After deterministic review, separately authorized exact Windows and Ubuntu
candidates would need bounded physical checks for:

- visible persistence wording before the action;
- keyboard-only dialog opening, cancellation, focus return, and status
  announcement;
- exact saved filename, byte count, SHA-256, UTF-8/no-BOM bytes, and equality
  with the visible preview;
- no clipboard change during save;
- refusal to replace an existing sentinel file;
- controlled handling of an inaccessible destination and a stale preview;
- no retained recent-file path, application history, log entry, second file,
  or unexpected temporary residue after handled outcomes; and
- normal and enlarged-text presentation.

Each platform run would require an exact source, binary/package identity,
commands, stop conditions, cleanup plan, and separate authorization. The study
does not authorize those runs.

## Decision

The study does not adopt report saving and does not defer the idea entirely.
The concrete benefit and Tauri's Rust-side dialog support justify one later
implementation-preparation contract. That contract must remain free to defer
if it cannot prove a proportionate no-clobber staging and cleanup boundary on
the exact Windows and Ubuntu targets.

Until such a contract is separately approved and implemented, the current
clipboard-only behavior and all existing report privacy statements remain
authoritative.
