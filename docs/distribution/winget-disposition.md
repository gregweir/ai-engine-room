# WinGet disposition for the current preview

Status: **DEFER THE EXACT `v0.1.0-preview.1` CANDIDATE; NO SUBMISSION
AUTHORIZED** on 2026-08-31.

## Decision

Do not prepare or submit a WinGet manifest for the current public preview, and
do not run the installer again under the consumed Milestone 1X evidence gates.
Keep the existing GitHub prerelease unchanged and available through its current
direct-download route.

This is a proportionate deferral, not a conclusion that WinGet is unsuitable
for AI Engine Room and not a prediction that Microsoft would reject the
installer. A later version may reconsider WinGet under a new candidate-specific
contract.

## Evidence considered

The exact current Windows candidate remains:

| Field | Recorded value |
| --- | --- |
| Release | `v0.1.0-preview.1` prerelease |
| Source revision | `a5482e9d51657a0cfb4471215a91750c5ba7db95` |
| Hosted filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Bytes | `2,651,735` |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Authenticode | `NotSigned` |
| Installed name | `AI Engine Room` |
| Installed version | `0.1.0` |
| Installed publisher | `Tartanleaf.com Inc.` |
| Install scope | Current user |

Across the separately authorized disposable runs, the installer returned exit
code `0`, registered one application, and exposed a usable DisplayIcon that
resolved to `aiengineroom.exe` inside the installation directory. Cleanup after
each stopped execution invoked the registered silent uninstaller successfully.

Those are useful facts, but they do not complete the project's planned
feasibility contract. The primary unattended-removal and post-removal absence
assertions never ran because bounded application launch stopped first.

## Launch-time observation

Milestones 1X-H and 1X-J each stopped during bounded launch on an established
connection to `150.171.22.17:443`. The final run recorded:

| Field | Observed value |
| --- | --- |
| Owner process | `msedgewebview2.exe` |
| Owner PID | `7188` |
| Process-tree role | `descendant` |
| Immediate parent | `msedgewebview2.exe` (PID `2852`) |
| Attribution method | `owner_pid_lookup` |
| TCP state | `Established` |
| Cleanup | `registered silent uninstall completed` |

This snapshot establishes only that a WebView2 descendant held the connection
during that bounded launch. It does not identify the destination's owner or
purpose, inspect traffic, or establish that a prompt, response, model detail,
telemetry event, update request, or any other particular content was sent.

No address, process, publisher, or presumed service is allow-listed. The final
one-run gate is consumed, and this decision grants no retry with a weaker stop
condition.

## Current official-policy interpretation

The current Windows Package Manager Community Repository policy says submitted
packages must install without requiring user interaction. It also subjects
packages to security scans and automated and manual validation. The official
FAQ explains that the installed application is run after installation so the
validation system can check whether suspicious processes start.

The reviewed public policy does not say that every external connection during
that launch is automatically disqualifying. The project therefore must not
describe the observed connection as proof of likely rejection. Conversely,
successful registered installation and cleanup do not predict acceptance by
Microsoft's private scans or moderation.

Primary sources checked on 2026-08-31:

- [Community Repository Policies](https://github.com/microsoft/winget-pkgs/blob/master/doc/Policies.md)
- [Community Repository FAQ](https://github.com/microsoft/winget-pkgs/blob/master/doc/FAQ.md)
- [Submit your manifest to the repository](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)

These requirements can change and must be checked again before any future
candidate is prepared.

## Why deferral is preferable now

Preparing a manifest for these exact bytes would move the project toward an
external submission without satisfying its own final evidence contract. A
fourth bespoke run would contradict the deliberately final 1X-J gate and is
unlikely to add proportionate value unless the candidate or its launch behavior
changes.

Deferral preserves the evidence honestly, avoids treating an unknown network
purpose as either harmless or harmful, and avoids spending additional effort on
an unsigned preview whose next version may provide a cleaner version,
packaging, signing, and launch-behavior contract.

## Conditions for future reconsideration

A later WinGet proposal should start from a new immutable candidate and must
separately establish:

1. aligned installed version, release identity, filename, publisher metadata,
   official page, and immutable URL;
2. exact signing state and the expected user-warning and store-validation
   boundary;
3. a deliberate explanation or correction of launch-time WebView2 network
   behavior without inferring purpose from an address;
4. unattended install and removal with complete absence assertions in a fresh
   disposable Windows environment;
5. current manifest schema, repository policy, security-validation, and
   publisher-source requirements;
6. ownership of updates, broken-link response, deprecation, and withdrawal;
   and
7. separate developer approval before manifest creation and again before any
   external submission.

## Next planning candidate

The existing roadmap places Snap after the WinGet decision. The next proposed
step is therefore a documentation-only Snap Store feasibility study, not a
build or publication attempt.

Initial official documentation indicates that Snap publication uses a free
developer account, a globally unique registered name, automated store review,
and release channels. Strict confinement is the normal publication model;
network client and basic desktop interfaces auto-connect, while classic
confinement requires manual approval. Store-installed snaps refresh
automatically by default, which is a material product and support difference
from AI Engine Room's current no-application-updater statement.

The future study must determine whether the app's loopback provider access,
`/proc/meminfo` observations, WebKit desktop integration, clipboard behavior,
licence payload, and report boundaries work usefully under strict confinement.
It must also account for a separate Snapcraft packaging definition because the
current Tauri bundle configuration targets Debian packaging rather than Snap.

No Snap account action, name reservation, package definition, build, upload,
store review, or release is authorized by this record.

## References

- [Disposable-runner verification record](winget-disposable-runner-verification-record.md)
- [Milestone 1X-K disposition contract](../design/milestone-1x-k-winget-disposition-contract.md)
- [Snap publishing process](https://documentation.ubuntu.com/snapcraft/stable/explanation/snap-publishing-process/)
- [Snap confinement](https://snapcraft.io/docs/explanation/security/snap-confinement/)
- [Snap network interface](https://snapcraft.io/docs/reference/interfaces/network-interface/)
- [Snap update behavior](https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/)
- [Craft a pre-built app](https://documentation.ubuntu.com/snapcraft/stable/how-to/integrations/craft-a-pre-built-app/)
