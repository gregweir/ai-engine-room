# Tartanleaf publisher-page publication record

Status: **published and publicly verified** on 2026-08-31.

## Published routes

- Official product page: <https://www.tartanleaf.com/ai-engine-room/>
- Stable publisher navigation route: <https://www.tartanleaf.com/tools/>
- Exact public prerelease:
  <https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1>

The Tools page presents AI Engine Room after HomeNetCheck and before Open Lobby
Board and links to the official product page. The product page links the exact
Windows and Ubuntu preview assets through the public GitHub prerelease.

## Verified public content

The publication and later plain-language revisions were separately approved.
Public verification established that the product page:

- identifies **Greg Weir** as developer and **Tartanleaf.com Inc.** as
  publisher;
- describes the application as a free, open-source community preview;
- identifies the release as AI Engine Room 0.1.0 and links
  `v0.1.0-preview.1`;
- exposes the accepted Windows and Ubuntu filenames, byte sizes, SHA-256
  values, and checksum file;
- keeps the Windows unsigned warning and the bounded support, privacy, and
  product claims;
- states that AI Engine Room is not available through WinGet, APT, Snap, the
  Microsoft Store, or another app catalog;
- links source, quick-start, user-guide, support, privacy, security, licence,
  notice, and release routes; and
- remains readable with its principal content and links available in both the
  normal browser layout and a 390-pixel narrow layout.

The Tools-page entry uses non-technical summary copy, remains visible at normal
and narrow widths, and reaches `/ai-engine-room/` through the expected internal
link.

## Relationship to the repository draft

[`winget-publisher-page-copy-draft.md`](winget-publisher-page-copy-draft.md) is
the historical repository-local source used to prepare the publication. The
live CMS page is authoritative. Its layout and several passages were revised
after publication for a non-technical Tartanleaf audience, so the live page is
not represented as a byte-for-byte rendering of that draft.

The public page closes the official publisher-source discoverability gap for
the current preview. It does not transfer release evidence to a rebuilt or
changed installer.

## Boundaries and remaining gates

This publication does **not** establish:

- WinGet package availability or acceptance;
- approval or reservation of `Tartanleaf.AIEngineRoom`;
- silent-install or silent-removal behavior;
- a valid WinGet manifest;
- signed publisher identity;
- broader Windows, Linux, architecture, security, privacy, or production
  compatibility; or
- authorization to modify the release, submit a manifest, or create another
  distribution channel.

The next possible distribution gate remains a bounded install and removal run
against an exact candidate in a supported disposable Windows environment. The
earlier run stopped safely because no such environment was available. Manifest
authoring, repository validation, and external submission require later,
separate approval even if that run passes.
