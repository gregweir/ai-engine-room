# WinGet distribution feasibility assessment

Status: **preparation only** on 2026-08-31. No WinGet manifest, external fork,
submission, installer run, package listing, or new release was created.

## Decision summary

WinGet is the first recommended package-manager feasibility target for AI
Engine Room. The existing Windows package format is compatible in principle,
and the community submission route does not state that an NSIS executable must
carry Authenticode. The current unsigned preview is nevertheless **not
submission-ready**: unattended behavior, package-manager metadata correlation,
isolated validation, official-site discoverability, and acceptance by
Microsoft's warning-sensitive security pipeline remain untested.

Snap is the next candidate after a WinGet decision. A PPA and official
Debian/Ubuntu inclusion remain later, higher-effort source-packaging paths.
Flathub is not a current target because its published requirements prohibit
AI-generated or AI-assisted application content and AI-operated submissions,
subject only to case-by-case exceptions.

## Authoritative requirements used

This assessment uses the following current primary sources:

- Microsoft Learn,
  [Submit your manifest to the repository](https://learn.microsoft.com/en-us/windows/package-manager/package/repository);
- Microsoft Learn,
  [Create your package manifest](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest);
- the Windows Package Manager Community Repository,
  [Community Repository Policies](https://github.com/microsoft/winget-pkgs/blob/master/doc/Policies.md);
- the Windows Package Manager Community Repository,
  [installer manifest schema](https://github.com/microsoft/winget-pkgs/blob/master/doc/manifest/schema/1.28.0/installer.md);
- Tauri,
  [Windows distribution guidance](https://v2.tauri.app/distribute/); and
- Flathub,
  [application requirements](https://docs.flathub.org/docs/for-app-authors/requirements).

These external requirements can change. Recheck them immediately before any
implementation or submission.

## Existing evidence

The public
[`v0.1.0-preview.1` publication record](../release/unsigned-preview-publication-record.md)
establishes:

| Field | Existing evidence |
| --- | --- |
| Application | AI Engine Room |
| Developer | Greg Weir |
| Package publisher metadata | Tartanleaf.com Inc. |
| Version in tracked build metadata | `0.1.0` |
| Public release identity | `v0.1.0-preview.1`, marked prerelease |
| Windows format | NSIS executable, x86-64 |
| Hosted filename | `AI.Engine.Room_0.1.0_x64-setup.exe` |
| Hosted bytes | 2,651,735 |
| SHA-256 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Authenticode | `NotSigned`; no authenticated publisher |
| Exact-artifact evidence | Developer install, UI, and removal acceptance on the named Windows baseline |

NSIS is an accepted executable installer type under the current community
repository policy. WinGet requires a version-specific installer URL and
SHA-256, accurate manifests, unattended installation, correct install/remove
behavior, automated validation, security scanning, and moderator review.
MSIX packages must be signed; the cited schema does not state the same formal
signature rule for an NSIS executable.

That distinction is narrow. Microsoft tests installers in isolated
environments and rejects packages that fail security scans or cannot install
without warnings or interaction. The existing successful manual acceptance on
one machine does not predict that result.

## Readiness gaps

| Gap | Why it matters | Required later evidence |
| --- | --- | --- |
| Release/version clarity | WinGet recommends that `PackageVersion` match installed Apps & Features metadata. The existing artifact reports `0.1.0`; its `v0.1.0-preview.1` tag is a separate channel label, not automatically a schema failure. | For one future immutable candidate, use the installed version consistently in build metadata, filename, Apps & Features, and manifest; explain any separate preview label consistently in the release and publisher page. |
| Official-source discoverability | Installer URLs must come from and be discoverable through an official publisher source. | Verify that a publisher-controlled HTTPS page identifies the app and links the exact version-specific release asset. |
| Silent install | Community packages must install without required interaction. | In Windows Sandbox or an equivalent disposable environment, verify the exact NSIS silent switch, exit code, installed files, launch identity, and absence of prompts or warnings. |
| Silent removal | Validation checks correct uninstall behavior. | Verify the registered uninstaller silently removes the exact installation without residue outside documented platform artifacts. |
| Installed metadata | Name, publisher, display version, scope, architecture, and product identity must correlate with the manifest. | Capture only the required Add/Remove Programs fields from the isolated test and compare them with the proposed manifest. |
| Unsigned validation | A schema-compatible unsigned NSIS file can still fail reputation, warning, antivirus, or potentially-unwanted-application checks. | Stop on any warning or scan failure; do not bypass policy. Decide separately whether to remediate, seek no-cost signing, or abandon the channel. |
| Manifest identity | A durable package identifier and accurate URLs must be stable across updates. | Deliberately approve the identifier—tentatively `Tartanleaf.AIEngineRoom`—and complete version, installer, default-locale, licence, support, release-notes, and Apps-and-Features fields. |
| Update responsibility | Every published version needs a new immutable installer and reviewed manifest update. | Approve a responsible developer, update checklist, supported-version policy, and withdrawal response before the first submission. |

The existing public preview must not be repurposed automatically. Its
publication record explicitly requires a new gate before adding a distribution
channel, and its prerelease/version relationship was not designed as a WinGet
contract. The
[identity and publisher-page plan](winget-identity-and-publisher-page-plan.md)
records the recommended interpretation and the remaining evidence gates.

## Bounded next verification

A later developer-approved run should use a fresh, disposable Windows Sandbox
and one separately approved exact installer candidate. Before execution, record
only its source revision, filename, byte size, SHA-256, version, and unsigned or
signed status. Then:

1. confirm the sandbox starts from a clean state and that organizational policy
   permits the test;
2. validate the candidate's hash before execution;
3. run the documented NSIS unattended installation path;
4. stop on any dialog, warning, policy block, unexpected network request,
   unexpected child process, non-zero result, or need for a bypass;
5. inspect only the required installed identity and Add/Remove Programs fields;
6. launch once without a provider and confirm bounded passive startup;
7. close the app and run the documented unattended removal path;
8. verify installed-app absence and expected file removal; and
9. discard the sandbox.

This run would establish local feasibility only. It would not reproduce
Microsoft's private security scans or guarantee community-repository approval.

## Possible submission sequence

If the bounded verification passes and the developer separately approves a
submission candidate:

1. publish or select an immutable, version-aligned installer under its own
   release gate;
2. expose the exact asset through an official publisher-controlled page;
3. generate the required multi-file manifest with WinGetCreate;
4. validate the manifest locally and test it using the repository's Windows
   Sandbox procedure;
5. independently review URLs, hashes, metadata, silent switches, licence and
   support links, installed identity, and removal;
6. separately approve the external contribution and contributor agreement;
7. submit one package/version pull request to `microsoft/winget-pkgs`;
8. respond narrowly to automated and moderator feedback without substituting a
   different binary; and
9. after merge, verify discoverability, exact downloaded bytes, installation,
   and removal from the public WinGet source.

Any changed binary, hash, version, URL, installer behavior, publisher identity,
or signing state invalidates the candidate and returns the work to review.

## Cost and signing boundary

The documented community-repository route does not identify a submission fee.
It also does not make Authenticode a formal schema requirement for NSIS, while
explicitly requiring signatures for MSIX. This assessment therefore identifies
no mandatory certificate purchase before a bounded NSIS feasibility test.

It does **not** conclude that signing is unnecessary. Unsigned status may still
cause security, warning, reputation, organizational-policy, or user-trust
failure. No certificate application, purchase, SignPath action, publisher
verification, or signing implementation is authorized by this assessment.
