# WinGet identity and publisher-page readiness plan

Status: **planning baseline with publisher-page publication recorded** on
2026-08-31. This plan did not approve a package identifier, release candidate,
publisher-page change, WinGet manifest, external contribution, or installer
execution. The later website publication received separate approval and is
recorded in
[`tartanleaf-publisher-page-publication-record.md`](tartanleaf-publisher-page-publication-record.md).

## Decision summary

AI Engine Room can prepare a durable WinGet identity without changing the
existing unsigned preview. Current Windows Package Manager guidance recommends
using the version reported by Windows Apps & Features as `PackageVersion`. The
existing installer reports `0.1.0`, so a future manifest targeting those exact
bytes would use `0.1.0`; the GitHub tag `v0.1.0-preview.1` is a release-channel
label, not the installed-version correlation field.

The difference is therefore not automatically a manifest-schema failure. It
is a provenance and communication issue that must be explained consistently.
The official Tartanleaf publisher page now makes the exact current preview
discoverable. The existing preview still must not be submitted automatically
because its silent install and removal behavior has not been verified in a
supported disposable Windows environment, and the remaining candidate,
manifest, validation, maintenance, and submission gates are not complete.

## Current authoritative basis

The current Windows Package Manager documentation establishes that:

- package identifiers are case-sensitive, generally use
  `Publisher.Package`, and must match the repository folder structure;
- `PackageVersion` is release-specific and should use the version reported by
  Windows Apps & Features to support correct list and upgrade correlation;
- package name and publisher should match their installed Apps & Features
  values;
- `AppsAndFeaturesEntries` may record installed name, publisher, version,
  product code, and installer type for correlation;
- submitted installers must complete without required user interaction; and
- manifest URLs should use official publisher sources, with installer URLs
  discoverable from the publisher's website and preferably immutable per
  version.

Primary sources:

- Microsoft Learn,
  [Create your package manifest](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest);
- Microsoft Learn,
  [Submit your manifest to the repository](https://learn.microsoft.com/en-us/windows/package-manager/package/repository);
- Windows Package Manager Community Repository,
  [Community Repository Policies](https://github.com/microsoft/winget-pkgs/blob/master/doc/Policies.md);
- Windows Package Manager Community Repository,
  [default-locale manifest schema](https://github.com/microsoft/winget-pkgs/blob/master/doc/manifest/schema/1.12.0/defaultLocale.md); and
- Windows Package Manager Community Repository,
  [installer manifest schema](https://github.com/microsoft/winget-pkgs/blob/master/doc/manifest/schema/1.12.0/installer.md).

These requirements can change and must be checked again immediately before
manifest authoring or submission.

## Proposed identity contract

The following values are recommendations for a later candidate gate, not
approved manifest fields:

| Field | Proposed value or rule | Basis |
| --- | --- | --- |
| Package identifier | `Tartanleaf.AIEngineRoom` | Durable `Publisher.Package` form; remains tentative until uniqueness and exact casing are rechecked. |
| Package name | `AI Engine Room` | Must match the installed Apps & Features display name. |
| Publisher | `Tartanleaf.com Inc.` | Confirmed product/package publisher; must exactly match installed publisher metadata. |
| Author/developer | `Greg Weir` | Distinct from the package publisher and suitable for the optional manifest `Author` field. |
| Package version | Exact installed Apps & Features version | For the existing preview evidence, this is `0.1.0`; do not substitute the prerelease tag text. |
| Release label | Explicitly identify the distribution as a preview | May be expressed in the release title, notes, and publisher page without changing installed-version correlation. |
| Installer type | NSIS/Nullsoft x64 executable | Subject to later isolated confirmation of type and unattended behavior. |
| Licence | `Apache-2.0` | Link to a version-pinned licence source for the exact release. |
| Package URL | Publisher-controlled HTTPS product page | Must identify the application and its distribution status. |
| Installer URL | Immutable, version-specific public asset | Must resolve to the exact reviewed bytes and be discoverable from the publisher page. |
| Installer hash | SHA-256 of the exact immutable asset | Any byte change invalidates the candidate. |
| Support and security | Public, bounded routes appropriate to the project | Must not imply service levels or guarantees that do not exist. |

Do not select a new version merely to make the tag and installer text look
identical. A new candidate should instead choose its installed version first,
then align tracked build metadata, installer filename, Apps & Features entry,
manifest `PackageVersion`, release title and notes, publisher page, and support
records around that value. A preview channel label may remain separate if its
meaning is explicit and consistent.

## Publisher-page content contract

The official
[AI Engine Room publisher page](https://www.tartanleaf.com/ai-engine-room/) now
provides a stable public path to the current preview, with the
[Tartanleaf Tools page](https://www.tartanleaf.com/tools/) as its publisher
navigation route. Before any later WinGet submission, that page must be updated
and reverified for the exact candidate. The smallest useful page should state:

1. **Identity:** AI Engine Room; developer Greg Weir; publisher Tartanleaf.com
   Inc.
2. **Purpose and status:** a free, open-source, unsigned community preview for
   bounded observation of supported local-AI environments.
3. **Exact release:** installed/package version, preview label, release date,
   supported operating-system and architecture scope, immutable release-notes
   link, exact installer link, byte size, and SHA-256.
4. **Trust boundary:** the Windows installer is unsigned, may produce a
   platform warning, has no authenticated publisher, and is not a privacy,
   safety, compute-placement, or performance guarantee.
5. **Project routes:** source repository, version-pinned Apache-2.0 licence,
   documentation, support/issue route, security policy, and privacy statement
   or a clear statement that the app has no accounts, telemetry, or upload.
6. **Distribution state:** whether WinGet availability is planned, under
   review, or live. Do not claim availability before catalog verification.
7. **Update boundary:** no automatic updater or silent background update claim;
   each published candidate is immutable and separately reviewed.

The page should link the exact asset directly or through a stable release page
that exposes it. It must not be a generic company page from which the installer
cannot reasonably be discovered.

## Candidate evidence checklist

Before manifest authoring, a later approved gate must establish:

- immutable source revision, release identity, filename, bytes, SHA-256, and
  signing state;
- exact Apps & Features display name, publisher, version, architecture, scope,
  product code if present, and uninstaller identity;
- unattended install and removal commands, exit codes, absence of required
  interaction, and bounded residue results in a fresh disposable Windows
  environment;
- one passive launch without provider access or inference;
- publisher-page discoverability of the exact candidate;
- accurate licence, source, support, security, privacy, and release-notes URLs;
- local multi-file manifest validation and the repository's isolated test; and
- independent review before any external pull request.

Any changed binary, version, URL, hash, installer behavior, publisher identity,
or signing state invalidates the candidate.

## Maintenance responsibility required before submission

The developer must approve who is responsible for each future manifest update,
a maximum time for responding to broken URLs or security findings, the
supported-version policy, and a withdrawal/deprecation procedure. Every new
application release would require a new immutable asset, hash, manifest version
directory, validation pass, and reviewed contribution.

## Explicit stop boundary

This plan did not authorize editing the Tartanleaf website; that later work was
separately approved and is now recorded. The plan still does not authorize
changing version metadata, rebuilding or downloading an installer, creating a
release or manifest, installing WinGet tooling, forking or contributing to
`microsoft/winget-pkgs`, accepting contributor terms, purchasing a certificate,
signing code, or claiming WinGet availability.
