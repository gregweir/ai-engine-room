# Unsigned preview candidate evidence

Status: build and read-only inspection evidence recorded on 2026-08-30.
Physical exact-artifact acceptance and publication authorization remain
pending. This record does not authorize installation, upload, distribution,
signing, hosting, or release.

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
- Installed application executable Authenticode status: **pending physical
  acceptance**

The repository-readiness verifier passed on the native CRLF checkout. The
Windows real-package licence-payload verifier passed using the already-installed
7-Zip 26.00 tool. Read-only NSIS inspection reported a physical size of
`2651735`, LZMA compression, and 11 payload files. The payload included the
application executable plus `LICENSE`, `NOTICE`, `THIRD-PARTY-LICENSES.txt`,
and `THIRD-PARTY-SOURCES.txt`; the extracted third-party source-location file
was `41448` bytes. The tracked, staged, and untracked source-control state
remained clean; ignored build output was not staged or committed.

## Pending exact-artifact acceptance

Neither candidate's earlier UI evidence transfers to these rebuilt files. The
following remain publication blockers for these exact hashes:

1. normal installation on the accepted narrow Linux and Windows baselines;
2. native launch and bounded graphical/keyboard/passive-behaviour review;
3. explicit `NotSigned` verification of the exact installed Windows
   executable;
4. normal removal and post-removal package/application absence; and
5. independent review of the completed physical-acceptance evidence.

No candidate was uploaded, published, distributed, signed, or released during
this evidence collection. A separate developer decision remains required for
the exact artifacts, final release-page text, and publication channel.

