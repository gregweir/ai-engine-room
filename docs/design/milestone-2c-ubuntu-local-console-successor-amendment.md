# Milestone 2C Ubuntu local-console successor amendment

## Status

Prepared locally as a documentation-only amendment from exact public baseline
`64d1283fc2d4ccd36dcc756a427f65296a672ecd`. This preparation may add only this
file and one local commit for separate review. It does not authorize a build,
executable launch, native dialog, clipboard access or change, report-file
operation, platform run, push, pull request, publication, or release.

The original
[Milestone 2C native validation contract](milestone-2c-native-report-save-validation-contract.md)
remains the governing contract except where this amendment narrows the Ubuntu
launch and operator-coordination procedure. It is not rewritten by this file.

## Disposition of the consumed Ubuntu runs

Two separately authorized Ubuntu application launches have been consumed.
Neither is reusable authority and neither established a native-validation pass.

The first run stopped after the operator accepted the first save dialog before
the mandatory cancellation check. A later cancellation within that launch
showed the expected dialog and cancellation behavior, but the fixed sequence
had already been violated. The run was independently reviewed as an attributed
procedural stop, not a product defect. Cleanup completed and the candidate
identity remained unchanged.

The successor run completed the cancellation-first checkpoint: the expected
dialog fields, cancellation wording, focus return, empty disposable directory,
and single candidate process were observed. Before **Copy** was exercised, the
application was no longer present when the operator returned. The outer
launcher returned exit code `1`, but the available evidence did not establish
whether that result came from the application, its remote parent, or another
cause. No clipboard operation or later report-save check occurred. The empty
disposable directory was removed and the candidate identity remained
unchanged.

These outcomes establish only narrow cancellation UI evidence. They do not
establish successful saving, exact preview/file equality, clipboard
non-mutation, no-clobber behavior, stale-preview rejection, inaccessible-
location handling, final layout behavior, normal process-exit semantics, or a
Milestone 2C Ubuntu result. Windows validation remains deferred.

## Exact retained candidate

No rebuild is proposed while the retained candidate remains available and
reverifies exactly:

| Field                       | Required value                                                     |
| --------------------------- | ------------------------------------------------------------------ |
| Repository                  | `https://github.com/gregweir/ai-engine-room.git`                   |
| Product source              | `5f54ec00cbfd884a0ffbce956d586d8ac8f5a199`                         |
| Product tree                | `7aa645875fc4dcd1b28e91eb209a073990bd1877`                         |
| Platform                    | Ubuntu 24.04 LTS x86_64, ordinary local graphical-session user     |
| Rust and Cargo              | `1.98.0`, `x86_64-unknown-linux-gnu`                               |
| `Cargo.lock` SHA-256        | `8769cc560c5ed3c6f00b10a135bc6125a6f9f7e655c6520cf73f95928d9d9082` |
| `package-lock.json` SHA-256 | `013e1fcd2917509cd098dfecacc83bd25c4aea1e01633f051d0f048bdf7d8dad` |
| Executable bytes            | `16,022,208`                                                       |
| Executable SHA-256          | `482a6e302469d9340d1b95337a0f9aa864367617421125fa5ad380f13f94599f` |
| Debian package bytes        | `4,896,812`                                                        |
| Debian package SHA-256      | `eae771fcee89f31b5ecfb5154c9fa71ff2ce94634228ca0599b7dcdcae6b438e` |
| Installation                | none; execute the verified build output directly                   |

Any missing candidate, identity mismatch, source change, dependency change,
installed copy, unexpected process, or platform change stops before launch. A
replacement candidate requires a separately authorized non-executing build and
new hashes; this amendment supplies no build authority.

## Required independent review and later decision

After this amendment is merged, an independent evidence-only reviewer must
confirm that it preserves the original privacy, disposable-directory,
single-launch, cleanup, evidence, and non-claim boundaries. Only then may the
developer separately approve or defer one new Ubuntu local-console successor
run bound to the exact source, package hash, and executable hash above.

Approval of this preparation, its commit, a pull request, or a merge is not run
authority. A generic continuation or either earlier Ubuntu authorization is not
reusable authority. Any future authorization is consumed when the executable
is first launched and grants no retry.

## Local-console launch boundary

The future executable must be launched from a terminal already open in the
same ordinary user's local graphical session. The terminal, application, and
operator must remain at the physical Ubuntu console from final identity check
through normal application close. The executable must not be parented by SSH,
a remote-control shell, a scheduler, a background service, an IDE task, or an
unreviewed helper.

Read-only coordination over SSH is allowed before launch and between operator
checkpoints, but it must not start, stop, signal, focus, or otherwise control
the application. The operator must not leave the console while the application
is running. Loss of the local terminal, operator availability, application
window, or graphical session is an immediate stop.

Immediately before requesting run authorization, preflight must report every
item required by the original contract plus:

- the retained candidate's exact lockfile, executable, and package identities;
- the local terminal's ordinary-user and graphical-session context;
- the absence of an SSH or remote-shell parent in the proposed invocation;
- the exact local-console invocation and outer timeout;
- the exact disposable-directory creation and cleanup commands; and
- confirmation that the operator is physically present and available for the
  complete uninterrupted sequence.

The proposed local-console command must invoke the exact executable once under
an outer timeout, discard unreviewed standard output and error rather than
retaining raw output, and print only the numeric launcher result after exit.
The exact command and timeout must be reviewed during preflight. No command may
install, copy, move, patch, wrap, or replace the candidate.

## Checkpointed operator sequence

The original fixed native check sequence remains required, with these stronger
coordination gates:

1. The operator navigates directly to **Report** and performs no refresh.
2. The first **Save report…** activation is cancellation-only. The operator
   confirms the title, suggested filename, and sole Plain text filter, then
   presses **Escape**. The operator must not accept the suggested filename.
3. The coordinator verifies the exact cancellation wording, focus return, one
   running candidate process, and an empty disposable directory before
   authorizing another UI action.
4. The operator activates **Copy** exactly once and immediately reports the
   controlled success or failure wording. The coordinator then performs only
   the contracted in-memory hash, fixed-sentinel replacement, and sentinel
   verification.
5. Each remaining save, no-clobber, stale-preview, inaccessible-location, and
   layout check is issued one at a time. The operator waits for the coordinator
   to verify the preceding directory, hash, clipboard, process, and UI result
   before continuing.
6. The operator clears the fixed clipboard sentinel, closes the application
   normally, and remains at the console until process termination, launcher
   result, residue inspection, and cleanup are confirmed.

No photo or screenshot is retained as validation evidence. An accidental
acceptance, extra dialog activation, refresh, copy, navigation, missed
checkpoint, operator absence, unexpected process exit, or outer-timeout expiry
is a consumed stop with no retry.

## Evidence and cleanup boundary

The original sanitized evidence and cleanup rules remain unchanged. Retained
evidence may include only controlled result labels, approved platform and
privilege classes, source and candidate identities, byte lengths, SHA-256
values, one invocation count, and the numeric launcher result. It must exclude
report text, clipboard text, raw terminal output, selected or temporary paths,
machine or user identity, screenshots, directory listings, and wall-clock
time.

Cleanup remains limited to the verified disposable directory and its fixed
contract entries. The candidate, source checkout, package, installed-package
state, and unrelated paths must remain unchanged. Cleanup success does not
convert an incomplete or stopped run into a pass.

## Desktop-icon observation

A generic dock icon was observed while directly running the uninstalled build
output. That observation is outside report-save validation and does not prove
an icon regression. It is deferred to a separately scoped installed-package
desktop-integration check and must not broaden this successor run.

## Required non-claims and exclusions

All non-claims and explicit exclusions in the original Milestone 2C contract
remain binding. In particular, this amendment does not establish or authorize:

- successful native report saving on Ubuntu or Windows;
- a Windows candidate build or run;
- a third Ubuntu launch before separate review and exact developer approval;
- an application correction, launcher helper, installer, package change, or
  desktop-icon correction;
- provider access, inference, network observation, telemetry, signing,
  publication, distribution, or release work; or
- universal atomicity, durability, privacy, confidentiality, crash safety, or
  safe behavior on other filesystems, platforms, accounts, or privilege levels.

## Acceptance criteria for this preparation

This amendment is ready for separate review only if:

- its parent is exact public baseline
  `64d1283fc2d4ccd36dcc756a427f65296a672ecd`;
- only this documentation file changes;
- both consumed Ubuntu outcomes remain narrow, sanitized historical evidence;
- the exact retained candidate is identified without granting build or run
  authority;
- local-console ownership, uninterrupted operator presence, one-at-a-time
  checkpoints, one-launch consumption, and no retry are explicit;
- Windows remains deferred;
- original evidence, cleanup, non-claim, and exclusion boundaries remain
  binding; and
- documentation formatting and diff checks pass without product execution.

## References

- [Milestone 2C native validation contract](milestone-2c-native-report-save-validation-contract.md)
- [Milestone 2B implementation-preparation contract](milestone-2b-safe-report-save-implementation-preparation-contract.md)
- [Architecture](architecture.md)
