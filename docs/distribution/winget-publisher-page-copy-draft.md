# AI Engine Room publisher-page copy draft

Status: **historical repository-local source copy** on 2026-08-31. This file
was used to prepare the separately approved public
[AI Engine Room publisher page](https://www.tartanleaf.com/ai-engine-room/).
The live CMS page is authoritative and was later revised for a non-technical
audience; it is not a byte-for-byte publication of this draft. Publication does
not make the installer eligible for WinGet submission. See the
[publication record](tartanleaf-publisher-page-publication-record.md).

## Read-only audit behind this draft

The exact public release remains the `v0.1.0-preview.1` prerelease. Its Windows
asset is `AI.Engine.Room_0.1.0_x64-setup.exe`, 2,651,735 bytes, with SHA-256
`6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399`.
GitHub reports the same digest for the hosted asset.

A point-in-time check of the public WinGet catalog and the
`microsoft/winget-pkgs` publisher path found no package using the tentative
identifier `Tartanleaf.AIEngineRoom` and no existing `Tartanleaf` publisher
directory. This supports further review of the identifier but does not reserve
or approve it; uniqueness must be checked again immediately before submission.

At the time of this draft, the Tartanleaf home and Tools pages did not identify
AI Engine Room or link its installer. The later, separately approved product
page and Tools-page entry closed that publisher-source discoverability gap for
the current preview. They do not establish the remaining WinGet evidence.

## Historical source copy used for publication

---

# AI Engine Room

AI Engine Room is a free, open-source desktop utility that helps non-specialists
understand supported local-AI runtime and machine context. It organizes bounded
evidence around **Observe → Explain → Diagnose → Report** while keeping missing
or unsupported information explicitly unavailable or unknown.

- Developer: **Greg Weir**
- Publisher: **Tartanleaf.com Inc.**
- Licence: **Apache License 2.0**
- Current release: **AI Engine Room 0.1.0 unsigned preview**

AI Engine Room is a community project, not a subscription, managed service, or
supported commercial product. It comes without a promise of individual
support, fixes, uptime, security monitoring, or continued development.

## Download the unsigned preview

The current release is a prerelease and is not signed, stable,
production-ready, security-certified, privacy-certified, or broadly
compatible. Review the release notes and exact platform limitations before
downloading:

[Open the official AI Engine Room 0.1.0 unsigned-preview release](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1)

| Platform and verified baseline | Download | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| Windows 11 25H2 build 26200.7462 x64 | [`AI.Engine.Room_0.1.0_x64-setup.exe`](https://github.com/gregweir/ai-engine-room/releases/download/v0.1.0-preview.1/AI.Engine.Room_0.1.0_x64-setup.exe) | 2,651,735 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| Ubuntu 24.04 LTS x86-64 | [`AI.Engine.Room_0.1.0_amd64.deb`](https://github.com/gregweir/ai-engine-room/releases/download/v0.1.0-preview.1/AI.Engine.Room_0.1.0_amd64.deb) | 4,722,942 | `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e` |

[Download the published checksum file](https://github.com/gregweir/ai-engine-room/releases/download/v0.1.0-preview.1/SHA256SUMS.txt)

Do not install a file with a different name, byte size, or SHA-256. A matching
checksum can detect changed bytes relative to this page; it does not identify
or authenticate the publisher and is not a digital signature.

## Important Windows warning

The Windows installer and installed executable have Authenticode status
`NotSigned`. Windows may show an unknown-publisher or reputation warning, and
security software or organizational policy may block the installer. Do not
disable or weaken a security control, install a trust certificate, or bypass
organizational policy. Continue only through a normal option offered by
Windows, only after confirming the download source and exact SHA-256. If no
permitted continuation is offered, do not install the preview on that device.

## What the application does—and does not do

AI Engine Room observes supported same-machine runtime APIs and bounded machine
context. It does not promise model fit, infer compute placement, benchmark
hardware, manage providers or models, perform automatic inference, or certify
that an AI interaction is private or local. Inference observations for eligible
Ollama and LM Studio models require a separate in-application disclosure and
explicit authorization for each run.

The application has no account, telemetry, upload, or application persistence.
Session observations remain in the current application process. It contacts
only the documented fixed loopback provider endpoints in its supported scope;
text reaches the operating-system clipboard only after an explicit **Copy
report** action. Other applications may then read clipboard content.

## Package-manager availability

AI Engine Room is **not currently available through WinGet, APT, Snap, the
Microsoft Store, or another package-manager catalog**. The files above are
direct-download unsigned preview packages. Any future package-manager listing
would be separately validated and announced here only after its public catalog
entry had been verified.

AI Engine Room has no automatic updater and makes no silent background-update
claim. Each public candidate is immutable and separately reviewed.

## Source, documentation, and project policies

- [Source code](https://github.com/gregweir/ai-engine-room)
- [Quick start](https://github.com/gregweir/ai-engine-room#quick-start)
- [User guide](https://github.com/gregweir/ai-engine-room/blob/main/docs/user-guide.md)
- [Support matrix and limitations](https://github.com/gregweir/ai-engine-room/blob/main/SUPPORT.md)
- [Privacy and data boundaries](https://github.com/gregweir/ai-engine-room#privacy-and-data-boundaries)
- [Security reporting](https://github.com/gregweir/ai-engine-room/blob/main/SECURITY.md)
- [Apache License 2.0 for the exact release](https://github.com/gregweir/ai-engine-room/blob/v0.1.0-preview.1/LICENSE)
- [Application notice for the exact release](https://github.com/gregweir/ai-engine-room/blob/v0.1.0-preview.1/NOTICE)
- [Release notes](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1)

Copyright © 2026 Tartanleaf.com Inc.

---

## Publication and verification outcome

The separately approved website work completed the following checks:

1. re-resolve the live release, asset URLs, byte sizes, hashes, signing state,
   source revision, support claims, and project links;
2. review the page in the Tartanleaf site's actual layout at narrow and normal
   widths, including link labels and long hashes;
3. confirm that the page keeps Greg Weir as developer and Tartanleaf.com Inc.
   as publisher;
4. verify that the website's analytics and privacy disclosure remain accurate
   for the page and direct GitHub downloads;
5. publication followed a separate developer approval; and
6. public verification confirmed that the Tools page provides a stable
   publisher navigation path to the product page and makes the exact installer
   discoverable from it.

Publishing this page established official-source discoverability only. It did
not authorize or establish silent-install feasibility, manifest validity,
security-scan acceptance, signing, WinGet submission, or catalog availability.
