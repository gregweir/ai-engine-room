# Milestone 2C reviewed local-helper amendment

## Status and purpose

Prepared locally from exact public baseline
`01ae5f87459770825e57bc79bec832f34cbc82a2`. This preparation adds one
documentation amendment, one local-console shell helper, and one deterministic
static contract test. It grants no authority to launch an executable, create a
validation directory, access or change the clipboard, create or inspect a
report file, exercise a native dialog, run native validation, push, publish, or
release.

The original
[native report-save validation contract](milestone-2c-native-report-save-validation-contract.md)
and the merged
[Ubuntu local-console successor amendment](milestone-2c-ubuntu-local-console-successor-amendment.md)
remain governing. This amendment changes only the earlier prohibition on an
unreviewed helper: after its exact commit and helper blob receive independent
review, the fixed helper may coordinate one separately authorized successor
run. It does not automate, focus, inspect, or operate the application UI.

## Exact helper boundary

The only proposed helper is
`scripts/milestone-2c-ubuntu-local-console-helper.sh`. It has two explicit
modes:

- `terminal-a` performs exact retained-candidate checks, refuses SSH or root,
  requires an interactive local X11 session, creates the fixed absent
  mode-`0700` disposable directory, and launches the exact candidate once in
  the foreground under the reviewed outer timeout;
- `terminal-b` refuses SSH or root, rechecks the candidate and disposable
  boundary, performs only the contracted in-memory clipboard hashes, fixed
  sentinel operations, fixture creation, exact-file hashes, residue checks,
  and narrow cleanup while pausing for the physically present operator at
  every UI checkpoint.

The helper constructs temporary paths internally. It neither asks the operator
to transfer a path between terminals nor writes a coordination file. It emits
only controlled result labels, numeric byte lengths, SHA-256 values, and the
numeric launcher result. It does not emit report or clipboard text, temporary
or selected paths, directory listings, raw application output, user or machine
identity, or wall-clock time.

The helper cannot establish run authority. Terminal A requires the operator to
type `RUN`, but that acknowledgement is valid only after the developer has
separately authorized one run bound to the exact product source, package hash,
executable hash, helper commit, and helper blob. Merely merging or invoking the
helper without that authorization is prohibited.

## Physical-console procedure

The future operator uses only the approved Ubuntu machine and remains at its
physical console for the complete run. Before launch, two ordinary-user X11
terminals must already be open:

1. In Terminal A, the operator changes to the synchronized public checkout and
   runs the helper with argument `terminal-a`. After its preflight succeeds and
   a separate run authorization exists, the operator types `RUN`. Terminal A
   remains occupied by the foreground application launch until exit.
2. After the application window appears, the operator uses Terminal B to run
   the same helper with argument `terminal-b`.
3. The operator follows one on-screen checkpoint at a time, types only the
   short controlled responses requested by the helper, and performs the stated
   physical keyboard or mouse action in the application.
4. The operator does not leave the console, use another application feature,
   refresh except at the stale-preview checkpoint, or perform any extra Copy,
   Save, close, terminal, or process action.
5. After closing the application normally when prompted, the operator reads
   the numeric launcher result from Terminal A and types only that number into
   Terminal B. Terminal B then performs the contracted narrow cleanup and
   reverification.

The coordinator may use SSH only for read-only observation and communication.
SSH must not create or alter the disposable directory, run either helper mode,
access the clipboard, create fixtures, launch, focus, signal, stop, or clean up
the application or validation state.

## Exact retained candidate and expected UI wording

The retained product source, tree, lockfiles, executable, and package remain
exactly those recorded in the prior amendment. The helper refuses any identity
change. No rebuild is permitted by this amendment.

The helper presents the operator with the expected native-dialog fields:

- title: `Save AI Engine Room report`;
- suggested filename: `ai-engine-room-report.txt`;
- sole filter: `Plain text`; and
- pending button: `Saving report…`.

It also presents the exact expected application outcomes needed by the fixed
sequence:

- `Save cancelled. No report file was created.`
- `Report copied to the system clipboard.`
- `Report saved as a plain-text file.`
- `That file already exists. AI Engine Room did not replace it. Choose a different name.`
- `The report changed before saving. Review it and try again.`
- `Saving is not available for that location. You can still copy the report.`
- `Could not save the report. No completed report file was created.`

The following additional controlled product wording is stop-only in this fixed
sequence. If any of it appears unexpectedly, the operator types `STOP` and
does not improvise:

- `Could not copy the report to the system clipboard.`
- `A report save is already in progress.`
- `Choose a filename ending in .txt.`
- `Saving did not complete, and AI Engine Room could not confirm removal of its temporary file. Check the location you chose.`
- `The operating system did not confirm whether the report was saved. Check the location you chose before trying again.`

Any other wording, raw error, elevation request, extra filter, extra operation,
unexpected file, process exit, operator absence, or helper failure is a
consumed stop. The operator must not improvise or retry.

## Deterministic review and later gates

`scripts/milestone-2c-ubuntu-local-console-helper.test.mjs` checks the helper's
fixed modes, SSH refusal, single foreground timeout invocation, controlled
directory operations, in-memory hashing, exact UI wording, lack of UI
automation, and lack of broad deletion, installation, network-transfer, raw
logging, or process-control commands. This is static contract verification; it
must not execute either helper mode.

Preparation may run only this static test, ordinary repository checks, and
formatting or diff checks that do not invoke the helper, application, package,
providers, or inference. After an exact local commit is prepared:

1. the developer separately decides whether to push it and run ordinary
   deterministic CI;
2. the developer separately decides whether to merge its pull request;
3. an independent evidence-only reviewer examines the merged amendment, exact
   helper blob, test, and governing contracts;
4. a read-only host preflight confirms the synchronized helper identity,
   retained candidate identity, local-console readiness, process and installed
   state, and absence of the fixed disposable directory; and
5. only then may the developer issue one exact successor-run authorization.

No step implies the next. Windows remains deferred until a completed Ubuntu
outcome is cleaned up and independently reviewed.

## Unchanged exclusions and non-claims

All original cleanup, evidence, result-class, non-claim, privacy, one-launch,
and no-retry rules remain binding. In particular, this amendment does not
authorize or establish a product correction, candidate rebuild, installation,
provider or inference use, network observation, Windows run, signing,
publication, distribution, or release. It does not establish universal
atomicity, durability, confidentiality, crash safety, or behavior on another
platform, filesystem, account, privilege level, or display server.
