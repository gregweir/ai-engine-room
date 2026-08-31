# Milestone 1X-K WinGet disposition contract

## Status

Prepared for developer review on 2026-08-31. This is a documentation-only
closure proposal for the current WinGet investigation. It authorizes no
package, manifest, account, name reservation, submission, release, signing,
installer execution, or product change.

## Purpose

Milestones 1X-A through 1X-J established the current public preview's exact
identity, publisher-page discoverability, registered silent installation,
installed executable identity, and bounded cleanup behavior. The final two
disposable runs stopped during application launch on the same externally
addressed WebView2 connection. The final run attributed that connection to a
WebView2 descendant but did not establish its destination owner, purpose, or
content.

This slice records a distribution decision from that evidence. It does not
reinterpret a stopped run as a pass, weaken the network boundary, or seek a
fourth execution.

## Authorized change if merged

Merging this contract would authorize only:

- this contract;
- one source-linked WinGet disposition record; and
- a roadmap update that defers the current preview from WinGet and identifies
  a separately gated Snap feasibility study as the next distribution-planning
  candidate.

The disposition may use tracked evidence and current official public
documentation. It must not create or validate a manifest, contact a store or
moderator, accept terms, reserve an identifier, or change external state.

## Required interpretation

The disposition must preserve all of these distinctions:

1. Current WinGet policy requires installation without user interaction and
   subjects installers to automated and manual validation.
2. Microsoft's published FAQ says the installed application is run during
   validation to check for suspicious processes.
3. The published policy reviewed for this slice does not state that every
   external connection during launch is automatically disqualifying.
4. AI Engine Room's own final gate nevertheless selected any non-loopback
   launch connection as a stop. That gate is consumed and cannot be weakened
   retroactively.
5. The current evidence therefore supports neither a complete local
   silent-install/removal feasibility pass nor a prediction of repository
   acceptance.
6. Attribution to `msedgewebview2.exe` does not establish destination
   ownership, purpose, trust, or transmitted content.

## Decision boundary

The recommended outcome is **defer WinGet for the exact
`v0.1.0-preview.1` candidate**. Deferral is not a rejection of WinGet as a
future channel and is not a claim that Microsoft would reject the installer.
It means the project will not spend another disposable run or prepare a
submission for these exact bytes under the current evidence contract.

Future reconsideration requires a new candidate and a new developer-approved
contract. That contract must deliberately address version identity, signing
state, launch-time network behavior, unattended removal and absence evidence,
publisher-page accuracy, maintenance responsibility, and current repository
policy. No authority carries forward from the consumed 1X gates.

## Subsequent distribution planning

After this disposition, the next recommended planning slice is a
documentation-only Snap Store feasibility study. It must examine at least:

- the need for a free developer account and globally unique registered name;
- strict confinement, required interfaces, and whether local provider access
  and Linux system evidence remain useful without classic confinement;
- the absence of a native Snap target in the current Tauri packaging setup and
  the maintenance cost of a separate Snapcraft definition;
- automatic refresh behavior and its difference from the application's current
  no-updater statement;
- store review, release channels, architecture, licence payload, provenance,
  and publisher identity; and
- build, install, UI, provider, removal, and independent-review evidence needed
  before any publication decision.

This contract does not authorize that study's implementation. It only defines
the proposed next gate.

## Explicit exclusions

This slice does not authorize:

- another disposable or physical Windows run;
- a WinGet manifest, identifier reservation, fork, contribution, submission,
  moderator contact, or availability claim;
- a Snapcraft file, Snap account action, name registration, build, upload,
  review request, channel release, or availability claim;
- candidate rebuilding, signing, replacement, or release changes;
- website changes, provider access, inference, or network lookup; or
- product network-observability implementation.

## Acceptance criteria

This proposal is ready for merge review only if:

- the diff is documentation-only and limited to this contract, the disposition
  record, and the roadmap;
- the record cites current official WinGet sources and the exact tracked run
  evidence;
- deferral is stated without predicting Microsoft acceptance or rejection;
- no later action is described as already approved; and
- deterministic repository checks pass.

## References

- [WinGet community repository policies](https://github.com/microsoft/winget-pkgs/blob/master/doc/Policies.md)
- [WinGet community repository FAQ](https://github.com/microsoft/winget-pkgs/blob/master/doc/FAQ.md)
- [Milestone 1X-J consumed run contract](milestone-1x-j-network-attribution-run-contract.md)
- [Disposable-runner verification record](../distribution/winget-disposable-runner-verification-record.md)
- [WinGet feasibility assessment](../distribution/winget-feasibility-assessment.md)
