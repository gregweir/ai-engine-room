# AI Engine Room 0.1.0 unsigned preview

## Unsigned preview

AI Engine Room 0.1.0 is pre-release software for inspecting bounded local-AI
runtime and machine context. This preview is not signed, stable,
production-ready, security-certified, or broadly compatible.

Developer: Greg Weir. Application and package publisher metadata:
Tartanleaf.com Inc. This unsigned preview does not have an authenticated
Authenticode publisher identity.

The Windows publisher will appear as unknown. Microsoft Defender SmartScreen,
Smart App Control, antivirus software, or organizational policy may warn about
or block the installer. Do not disable or weaken any security control, install
a trust certificate, or bypass organizational policy. Continue only through a
normal option offered by Windows, only when the installer came from this
project's official release page, and only after its SHA-256 matches the value
below. If Windows does not offer an allowed continuation, do not install the
preview on that device.

The Ubuntu package is a direct-download `.deb`; it is not distributed through
an APT repository and has no repository signature or repository-trust claim.

## Exact source and verification status

- Source commit: [`a5482e9d51657a0cfb4471215a91750c5ba7db95`](https://github.com/gregweir/ai-engine-room/tree/a5482e9d51657a0cfb4471215a91750c5ba7db95)
- Source tree: `c1e8092cb5caa75a77f7e53fdc712eae6f72b318`
- Deterministic checks: [passed on Ubuntu and Windows](https://github.com/gregweir/ai-engine-room/actions/runs/33309714136)
- Repository, licence-material, real-package licence-payload, and read-only
  archive checks: passed for both exact artifacts
- Physical exact-artifact acceptance: **passed; independent final review
  passed**

## Downloads

Verify each downloaded file against the exact identity below before
installation.

| Platform and verified baseline | Candidate filename | Format / architecture | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| Ubuntu 24.04 LTS x86-64 | `AI.Engine.Room_0.1.0_amd64.deb` | Debian package / AMD64 | 4,722,942 | `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e` |
| Windows 11 25H2 build 26200.7462 x64 | `AI.Engine.Room_0.1.0_x64-setup.exe` | NSIS installer / x64 | 2,651,735 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |

Exact Windows installer and exact installed application executable
Authenticode status: `NotSigned`. A checksum detects a file change relative to
this page; it does **not** identify or authenticate the publisher and is not a
digital signature.

Compatibility claims are limited to the exact baselines listed above. Other
Windows versions, Linux distributions, architectures, and macOS are not
currently claimed.

## Verify the downloaded file

Run the command for your platform from the directory containing the download,
then compare the complete result with the matching SHA-256 above.

Windows PowerShell:

```powershell
Get-FileHash -Algorithm SHA256 ".\AI.Engine.Room_0.1.0_x64-setup.exe"
```

Ubuntu:

```sh
sha256sum "AI.Engine.Room_0.1.0_amd64.deb"
```

Do not install a file with a different name, size, or SHA-256.

## Scope and important limitations

AI Engine Room observes supported local runtime and resource context and keeps
missing evidence unavailable or unknown. It does not promise model fit, infer
compute placement, benchmark hardware, manage providers or models, or perform
automatic inference. Inference requires a separate in-application disclosure
and explicit authorization for each run.

The application has no account, telemetry, or application persistence.
Session observations remain in the current application process. Text reaches
the operating-system clipboard only after an explicit **Copy report** action;
other applications may then read that clipboard content.

Provider, endpoint, platform, privacy, and pre-release qualifications are
detailed in the project's [support matrix and limitations](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/SUPPORT.md)
and [README privacy boundary](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/README.md#privacy-and-data-boundaries).

## Licence, notices, support, and security

- [Apache License 2.0](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/LICENSE)
- [Application NOTICE](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/NOTICE)
- [Third-party licences](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/THIRD-PARTY-LICENSES.txt)
- [Third-party source locations](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/THIRD-PARTY-SOURCES.txt)
- [Support and known limitations](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/SUPPORT.md)
- [Security reporting](https://github.com/gregweir/ai-engine-room/blob/a5482e9d51657a0cfb4471215a91750c5ba7db95/SECURITY.md)

For Microsoft's description of reputation-based protection and unsigned-app
warnings, see [Microsoft's SmartScreen documentation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).
