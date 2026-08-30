# Unsigned preview publication record

Status: **published and independently verified** on 2026-08-30.

## Public identity

- Repository: `gregweir/ai-engine-room`
- Release URL:
  <https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1>
- Tag: `v0.1.0-preview.1`
- Title: `AI Engine Room 0.1.0 unsigned preview`
- Setting: prerelease
- Source commit: `a5482e9d51657a0cfb4471215a91750c5ba7db95`
- Source tree: `c1e8092cb5caa75a77f7e53fdc712eae6f72b318`
- Developer: Greg Weir
- Application and package publisher metadata: Tartanleaf.com Inc.
- Authenticated Authenticode publisher identity: none

The public tag resolves directly to the approved source commit. The rendered
public page retains the reviewed unsigned warning, limitations, exact source
identity, compatibility boundaries, developer and publisher distinction, and
filename-specific verification commands.

## Public assets

GitHub reports exactly three uploaded release assets, excluding its generated
source archives:

| Asset | Bytes | SHA-256 |
| --- | ---: | --- |
| `AI.Engine.Room_0.1.0_amd64.deb` | 4,722,942 | `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e` |
| `AI.Engine.Room_0.1.0_x64-setup.exe` | 2,651,735 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| `SHA256SUMS.txt` | 198 | `2b1c787237ad10d9262f552438d9093cea44aee8c3c3ae035c4b564da2caf2de` |

Fresh public downloads reproduced all three byte sizes and SHA-256 values. The
downloaded Windows installer remained `NotSigned`. The downloaded manifest was
exactly the two approved dotted-name checksum lines and validated against both
downloaded binaries.

## Public-page checks

- A signed-out view showed the release publicly, with the exact title and
  `Pre-release` label.
- The tag and commit links showed the exact approved source identity.
- The release table and Windows and Ubuntu checksum commands used the approved
  dotted hosted filenames.
- The pinned source, deterministic-CI, SUPPORT, README privacy anchor, LICENSE,
  NOTICE, third-party licence, third-party source, SECURITY, and Microsoft
  SmartScreen links each returned HTTP 200.
- An independent reviewer queried the public release identity and asset
  metadata, downloaded the public assets separately, recomputed every size and
  hash, confirmed the executable had no PE certificate table, checked the exact
  manifest content, and reported **PASS with no blocker**.

## Boundary after publication

This record closes the approved `v0.1.0-preview.1` public transition. It does
not authorize editing or deleting the release, moving or replacing its tag or
assets, adding distribution channels, creating an updater or package
repository, signing these binaries, or publishing another release. Any such
action requires a new bounded review and explicit developer authorization.
