# Unsigned preview candidate evidence

Status: build, read-only inspection, exact-artifact physical acceptance, and
independent final review completed on 2026-08-30. The developer later approved
the exact non-public GitHub draft identity, final text, and three-asset upload.
The public transition remains unauthorized. This record does not authorize
signing or broader distribution.

## Source and deterministic gate

- Source commit: `a5482e9d51657a0cfb4471215a91750c5ba7db95`
- Source tree: `c1e8092cb5caa75a77f7e53fdc712eae6f72b318`
- Branch at build time: pushed `main`
- Sole remote: `https://github.com/gregweir/ai-engine-room.git`
- Deterministic GitHub Actions run:
  <https://github.com/gregweir/ai-engine-room/actions/runs/33309714136>
- CI result: all five jobs passed for the exact source commit, including
  frontend/repository contracts, Ubuntu and Windows Rust gates, and fresh Linux
  and Windows package-licence payload checks.

CI did not upload or publish these candidates. The identities below come from
the subsequent fresh host-local builds and inspections at the same source
commit.

## Ubuntu candidate

- Verified baseline: Ubuntu 24.04 LTS x86-64
- Build command: `npm run tauri build -- --bundles deb`
- Filename: `AI Engine Room_0.1.0_amd64.deb`
- Format and architecture: Debian package, `amd64`
- Package: `ai-engine-room`
- Version: `0.1.0`
- Byte size: `4722942`
- SHA-256: `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e`

The repository-readiness verifier and Linux real-package licence-payload
verifier passed after the build. Read-only `dpkg-deb` inspection confirmed the
package name, version, and architecture above. The tracked, staged, and
untracked source-control state remained clean; ignored build output was not
staged or committed.

## Windows candidate

- Verified baseline: Windows 11 25H2 build 26200.7462 x64
- Build command: `npm run tauri build -- --bundles nsis`
- Filename: `AI Engine Room_0.1.0_x64-setup.exe`
- Format and architecture: NSIS installer, x64 target
- Version: `0.1.0`
- Byte size: `2651735`
- SHA-256: `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399`
- Exact installer Authenticode status: `NotSigned`
- Build-output application executable Authenticode status: `NotSigned`
- Installed application executable Authenticode status: `NotSigned` (verified
  on the installed version `0.1.0` executable)

The repository-readiness verifier passed on the native CRLF checkout. The
Windows real-package licence-payload verifier passed using the already-installed
7-Zip 26.00 tool. Read-only NSIS inspection reported a physical size of
`2651735`, LZMA compression, and 11 payload files. The payload included the
application executable plus `LICENSE`, `NOTICE`, `THIRD-PARTY-LICENSES.txt`,
and `THIRD-PARTY-SOURCES.txt`; the extracted third-party source-location file
was `41448` bytes. The tracked, staged, and untracked source-control state
remained clean; ignored build output was not staged or committed.

## Exact-artifact physical acceptance

The bounded
[`unsigned-preview physical-acceptance checklist`](unsigned-preview-physical-acceptance-checklist.md)
was completed for the exact candidate hashes above.

On Ubuntu, pre-install package absence was confirmed, normal installation
passed, and the installed package reported `ai-engine-room` version `0.1.0`
for `amd64`. Native launch, graphical and keyboard review, and passive behavior
passed without provider action, inference, or **Copy report** use. Normal
package removal passed; the package and installed binary were absent afterward.

One bounded procedural deviation was recorded on Ubuntu: removal completed
before the application process had closed, leaving a process temporarily
running from the removed executable, reported by the operating system as
`(deleted)`. The package and binary were already absent, the condition was
disclosed, and the process was then closed normally. The final process count
was zero. Independent review found the deviation non-blocking and did not
require a rerun.

On Windows, pre-install application absence was confirmed. The installer
displayed no warning during the tested normal path; installation passed and the
Installed apps entry reported AI Engine Room version `0.1.0` with publisher
Tartanleaf.com Inc. Native launch, graphical and keyboard review, and passive
behavior passed without provider action, inference, or **Copy report** use.
The installed executable reported version `0.1.0` and Authenticode status
`NotSigned`. Normal removal passed; the Installed apps entry and application
and setup processes were absent afterward.

After acceptance, both candidate files retained their recorded byte sizes and
SHA-256 values, and both source checkouts remained clean at the recorded source
commit and tree. Independent final review returned **PASS** for the completed
evidence, including the disclosed Ubuntu deviation and correction.

No candidate was uploaded, published, distributed, signed, or released during
this evidence collection. The developer subsequently approved the exact
artifacts, final text, GitHub Releases channel, and non-public draft identity
recorded in the publication runbook. A later separate decision remains required
before the reviewed draft may become public.
