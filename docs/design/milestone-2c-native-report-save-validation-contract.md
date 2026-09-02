# Milestone 2C native report-save validation contract

## Status

Prepared locally on 2026-09-02 under developer authorization from exact public
baseline `5f54ec00cbfd884a0ffbce956d586d8ac8f5a199` (tree
`7aa645875fc4dcd1b28e91eb209a073990bd1877`). This preparation authorizes only
this documentation file and one local commit for separate review. It does not
authorize a build, package execution, native dialog, clipboard change, report
file operation, platform run, push, publication, or release.

The validation target is the merged Milestone 2B product source above. Package
and executable hashes are intentionally unresolved. They may be established
only by separately authorized, non-executing platform builds and must be bound
into later Ubuntu and Windows run authorizations independently.

## Decision and purpose

Milestone 2B passed deterministic Ubuntu and Windows compilation, tests,
Clippy, frontend checks, and package-licence checks. Those checks used
artificial filesystem seams and did not establish the behavior of the real
dialog or no-clobber filesystem adapters.

Milestone 2C defines the smallest native validation that can establish whether
the exact reviewed executable behaves as designed on one supported Ubuntu
desktop and one supported Windows desktop. It is evidence collection, not a
release, durability, privacy, or universal filesystem claim.

## Reviewed validation target

| Field                      | Required value                                               |
| -------------------------- | ------------------------------------------------------------ |
| Repository                 | `https://github.com/gregweir/ai-engine-room.git`             |
| Product source commit      | `5f54ec00cbfd884a0ffbce956d586d8ac8f5a199`                   |
| Product source tree        | `7aa645875fc4dcd1b28e91eb209a073990bd1877`                   |
| Ubuntu class               | Ubuntu 24.04 LTS x86_64, ordinary graphical-session user     |
| Windows class              | Windows 11 25H2 x64, Medium-integrity graphical-session user |
| Report filename            | `ai-engine-room-report.txt`                                  |
| Maximum report bytes       | 1,048,576 UTF-8 bytes                                        |
| Ubuntu package SHA-256     | unresolved until a separately authorized build               |
| Ubuntu executable SHA-256  | unresolved until a separately authorized build               |
| Windows package SHA-256    | unresolved until a separately authorized build               |
| Windows executable SHA-256 | unresolved until a separately authorized build               |
| Provider or inference use  | none                                                         |
| Installation               | none; run the exact hashed build output as an ordinary user  |

A different source commit or tree requires a contract amendment. A different
platform class, privilege context, dependency graph, package hash, executable
hash, or repository identity is an infrastructure stop, not permission to
substitute another candidate.

## Contract-preparation boundary

This contract-only slice may add only this file. It may not change a roadmap,
readiness script, workflow, source file, test, manifest, lockfile, capability,
package configuration, generated licence material, validation record, or
release material.

Formatting and repository diff checks are allowed. Product checks, dependency
resolution, package builds, executable launches, dialogs, clipboard access,
and filesystem validation are not allowed while preparing this contract.

## Separately gated candidate builds

Ubuntu and Windows candidates must be built separately from the exact product
source and tree above. Before either build, the proposed command, official
toolchain identity, target triple, repository identity, clean state, and
output locations must be reported for explicit authorization.

Each build authorization permits exactly one ordinary candidate build and
hashing operation on its named platform. It does not permit launching the
application or installer. The build report must include:

- source commit and tree;
- package-manager lockfile hash;
- Rust and Cargo versions plus target triple;
- package path class, byte length, and SHA-256;
- runnable application path class, byte length, and SHA-256;
- deterministic check result for the same source; and
- confirmation that no provider, inference, dialog, report file, installer,
  publication, or release operation occurred.

Paths must be reported as controlled classes, not retained user or machine
paths. Generated candidates remain local and are not release assets. A failed
or identity-mismatched build stops that platform preparation.

## Separately consumed platform authorizations

After candidate identities are reviewed, the sequence is:

1. The developer may authorize exactly one Ubuntu run using the exact product
   source, Ubuntu package hash, and Ubuntu executable hash.
2. The Ubuntu run must finish, clean up, and receive independent evidence-only
   review before a Windows run is proposed.
3. The developer may authorize exactly one Windows run using the exact product
   source, Windows package hash, and Windows executable hash.
4. The Windows run must finish, clean up, and receive independent evidence-only
   review before any disposition is proposed.

Each authorization is consumed when its executable is first launched. A stop,
failure, partial result, inaccessible UI, unexpected prompt, missed timing
window, cleanup uncertainty, or identity mismatch permits no retry. Approval
of this contract, a candidate build, one platform run, or an evidence record
does not imply any later gate.

## Platform preflight

Immediately before requesting each run authorization, report:

- hostname and user for coordination only; neither enters retained evidence;
- exact working directory, branch, HEAD, tree, local and live `main`, remotes,
  divergence, and tracked, staged, and untracked state;
- platform version, architecture, graphical-session availability, and
  privilege or integrity level;
- exact package and executable byte lengths and SHA-256 values;
- absence of another AI Engine Room process and absence of an installed copy
  that could be mistaken for the candidate;
- the existing clipboard-query mechanism, without reading current clipboard
  content, and confirmation that no tool installation is required;
- the exact disposable-directory creation and cleanup commands; and
- the exact executable invocation and outer timeout.

The run must stop before launch if it would require root, `sudo`, an elevated
Windows token, an installer, a synchronized or shared folder, a remote mount,
network access, a provider, inference, a new dependency, or an unreviewed
automation helper.

## Disposable run boundary

Each platform run uses one newly created, ordinary-user-owned local directory
under that platform's temporary-directory facility. Its resolved path must be
checked to remain beneath that facility before the executable starts. It may
contain only the fixed validation entries described below:

- `saved.txt`, created by the successful save check;
- `existing.txt`, created by the validation procedure with fixed sentinel
  bytes; and
- one platform-specific non-writable-location fixture only if it can be
  created and fully restored inside the disposable directory without
  elevation.

No home, documents, desktop, source checkout, synchronized folder, system
directory, package directory, or unrelated temporary entry may be selected.
Directory inspection is limited to the disposable directory. Recursive
searches, startup cleanup, recent-file inspection, and inspection of any other
path are prohibited.

## Fixed native check sequence

The application is launched exactly once as an ordinary graphical-session
user. The run uses the native **Report** workspace only. It does not open
Overview, invoke a provider or inference, or use another application feature
except **Copy** where explicitly required to establish the preview hash.

### 1. Cancellation and basic UI behavior

1. Activate **Save report…** by keyboard.
2. Confirm the dialog title, suggested filename, and Plain text filter.
3. Cancel the dialog.
4. Confirm the exact cancelled wording, polite status semantics, focus return
   to the button, and no entry in the disposable directory.

An unexpected path disclosure in the application, elevation request, raw
error, extra filter, remembered directory, file, or staging entry is an
attributed stop.

### 2. Exact new-file success and clipboard non-mutation

1. Freeze the currently visible preview by performing no refresh after this
   point.
2. Activate **Copy** once. Use the preflighted existing clipboard reader to
   hash the clipboard's exact UTF-8 bytes in memory. Retain only byte length
   and SHA-256, never the preview text.
3. Replace the clipboard with the fixed token
   `AI_ENGINE_ROOM_2C_CLIPBOARD_SENTINEL` and verify that token without reading
   or retaining the clipboard value that existed before the run.
4. Activate **Save report…** by keyboard and select `saved.txt` inside the
   disposable directory.
5. Confirm the exact saved wording, polite status semantics, and focus return.
6. Hash and measure `saved.txt`; require equality with the frozen preview byte
   length and SHA-256. Confirm valid UTF-8, no BOM, and no additional entry in
   the disposable directory.
7. Confirm the clipboard still contains exactly the fixed sentinel.

If the existing platform clipboard reader cannot preserve and hash the exact
text bytes without a new installation or a disk capture, the run stops before
the first application launch. The saved report text and clipboard text must
not enter terminal output or retained evidence.

### 3. Existing-destination no-clobber check

1. Create `existing.txt` with the fixed ASCII bytes
   `AI_ENGINE_ROOM_2C_EXISTING_SENTINEL` and record its byte length and SHA-256.
2. Select that exact file through **Save report…**.
3. Confirm the exact destination-exists wording and alert semantics.
4. Require the sentinel length and hash to remain unchanged, the clipboard
   sentinel to remain unchanged, and no staging entry to remain.

The validation procedure must not delete or replace the sentinel until final
cleanup.

### 4. Stale-preview rejection

The preflight must demonstrate a reviewed UI-only gesture sequence that can
activate **Save report…** and then activate **Refresh report** before a
destination is accepted, without a script, console, IPC call, product change,
or second application instance. The attempted sequence occurs once.

If the parented native dialog prevents the supported UI from refreshing the
preview, the check is recorded as `stale_preview_not_reachable` and the run is
a bounded partial result, not a pass. It must not be retried or bypassed with
an unreviewed helper. If refresh succeeds, selecting a new disposable filename
must yield the exact preview-changed wording and create no file or staging
entry.

### 5. Inaccessible-location handling

This check is permitted only when preflight can create a non-writable fixture
inside the disposable directory, prove it non-writable as the same ordinary
user, and restore it without elevation. Selecting a `.txt` name within that
fixture must produce a controlled unavailable or failed outcome, no raw error,
no destination, and no staging residue.

If a safe local fixture cannot be established, record
`inaccessible_fixture_unavailable`; the run is a bounded partial result and no
system directory or broader permission change may be substituted.

### 6. Final UI and termination checks

Confirm keyboard activation, visible focus, exact status or alert role, and
normal/enlarged-text layout for every exercised result. **Copy** remains
available after handled failures. Clear the fixed clipboard sentinel, close
the application normally, and confirm the exact process terminated within the
outer timeout.

## Cleanup and residue check

After confirmed process termination:

1. inspect only the disposable directory;
2. require no name beginning `.ai-engine-room-report.tmp-`;
3. restore the non-writable fixture if it was created;
4. delete the exact `saved.txt` and `existing.txt` entries;
5. remove the empty fixture and disposable directories; and
6. confirm the candidate package, executable, source checkout, installed-app
   state, and every unrelated path are unchanged.

If cleanup cannot be confirmed, retain only the controlled
`cleanup_unconfirmed` result and stop. Do not broaden inspection, recurse into
another directory, retry the application, or claim cleanup succeeded.

## Retained evidence boundary

The run may retain only a closed, sanitized record containing:

- approved platform class and ordinary-user privilege class;
- product source commit and tree;
- package and executable byte lengths and SHA-256 values;
- one invocation count and process exit result;
- fixed result labels for cancellation, success, destination-exists,
  stale-preview, inaccessible-location, UI/accessibility, clipboard, residue,
  and cleanup checks;
- preview and saved-file byte lengths and SHA-256 values only;
- existing-sentinel before/after byte lengths and SHA-256 values only; and
- the required non-claims below.

The record must not contain report text, clipboard text, selected or temporary
paths, filenames beyond the fixed contract names, username, hostname, machine
identifier, wall-clock time, provider/model data beyond what may be represented
by the non-reversible report hash, raw errors, screenshots, or directory
listings. Candidate output is reviewed independently before any evidence file
is proposed. No run output is committed automatically.

## Result classes

- **Native validation pass:** every fixed check completes with the required
  outcome, exact-byte equality, no-clobber evidence, UI behavior, clipboard
  non-mutation, confirmed termination, and cleanup.
- **Bounded partial result:** the stale-preview or inaccessible fixture cannot
  be reached safely through the allowed UI and disposable-directory boundary,
  while all exercised checks and cleanup pass.
- **Attributed stop:** product behavior, privacy, UI, overwrite, residue,
  privilege, bound, or cleanup rule fails.
- **Infrastructure stop:** source, binary, package, platform, toolchain,
  graphical session, clipboard reader, or repository identity cannot be
  established exactly.

No result permits a rerun under the same authorization.

## Required non-claims

No Milestone 2C outcome establishes:

- universally atomic, crash-proof, private, confidential, encrypted, signed,
  authenticated, or durable saving;
- safe behavior on a remote, synchronized, shared, removable, unsupported, or
  different filesystem;
- behavior after abrupt termination, power loss, process kill, or operating-
  system crash;
- absence of indexing, backup, synchronization, antivirus, or other software
  access to a saved file;
- behavior on another operating-system version, architecture, account,
  privilege level, display server, or package format;
- correctness of report facts, provider privacy, or absence of network access;
  or
- release, signing, publication, distribution, or adoption readiness.

## Explicit exclusions

This contract does not authorize:

- either candidate build, executable launch, platform run, or rerun;
- installer execution, application installation, elevation, system permission
  changes, remote shares, synchronized folders, or system-directory writes;
- overwrite, append, autosave, arbitrary format, directory scan, startup
  cleanup, or abrupt-termination experiments;
- provider access, inference, network observation, telemetry, update checks,
  publication, release, signing, or distribution work; or
- a validation record, roadmap change, product change, push, pull request, or
  merge.

## Contract acceptance

This preparation is ready for local review only if:

- its parent and validation target equal the exact merged Milestone 2B commit;
- only this file changes;
- unknown package and executable hashes remain unresolved and separately gated;
- Ubuntu and Windows builds and runs require distinct exact authorizations;
- every file operation stays inside a newly verified disposable directory;
- stale-preview and inaccessible-location limitations are reported honestly as
  partial evidence rather than bypassed;
- evidence excludes report text, paths, machine identity, and raw errors;
- cleanup and one-run consumption are explicit; and
- documentation formatting and diff checks pass without product execution.

## References

- [Milestone 2B implementation-preparation contract](milestone-2b-safe-report-save-implementation-preparation-contract.md)
- [Milestone 2A feasibility study](../research/safe-report-save-feasibility.md)
- [Architecture](architecture.md)
- [User guide](../user-guide.md)
- [Roadmap](../roadmap.md)
