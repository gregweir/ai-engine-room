# Unsigned preview publication and rollback runbook

Status: after approving and independently verifying the exact non-public draft,
the developer separately approved its public transition on 2026-08-30. The
exact prerelease was published and passed the post-publication checks recorded
in [`unsigned-preview-publication-record.md`](unsigned-preview-publication-record.md).
No approval is implied for changing the published release, adding a channel or
asset, signing, mirroring, promoting, or removing public material.

## Fixed scope

This runbook applies only to the two candidate hashes recorded in
[`unsigned-preview-candidate-evidence.md`](unsigned-preview-candidate-evidence.md)
and source commit `a5482e9d51657a0cfb4471215a91750c5ba7db95`.

The approved draft channel and identity are:

- repository and channel: GitHub Release in `gregweir/ai-engine-room`;
- target source commit: `a5482e9d51657a0cfb4471215a91750c5ba7db95`;
- tag: `v0.1.0-preview.1`;
- release title: `AI Engine Room 0.1.0 unsigned preview`;
- GitHub release setting: prerelease;
- checksum manifest: `SHA256SUMS.txt`; and
- initial state: non-public draft.

GitHub Releases normalizes spaces in uploaded asset names. After the first
non-public upload exposed that behavior, the developer approved these hosted
binary filenames without changing either binary's bytes:

- `AI.Engine.Room_0.1.0_amd64.deb`; and
- `AI.Engine.Room_0.1.0_x64-setup.exe`.

The original host-local candidate filenames retain spaces and remain the source
identities for local revalidation. The dotted names are the approved hosted
download identities.

The developer is Greg Weir. The application and package publisher metadata is
Tartanleaf.com Inc. Because this preview is unsigned, that metadata is not an
authenticated Authenticode publisher identity.

A different channel, identity, setting, or changed artifact requires a new
review and explicit authorization rather than adapting this runbook silently.

No updater, APT repository, Microsoft Store submission, package manager,
mirror, CDN, signing service, certificate, or automatic deployment is included.

## Entry gates

Do not begin any externally visible operation unless one evidence bundle shows
all of the following:

1. the Linux and Windows filenames, byte sizes, and SHA-256 values still match
   the candidate record;
2. physical installation, native launch, bounded graphical/keyboard/passive
   review, normal removal, and post-removal absence passed for both exact files;
3. the exact Windows installer and installed application executable both
   reported Authenticode `NotSigned`;
4. an independent reviewer found no discrepancy in the completed acceptance
   evidence;
5. the final release-page text and release notes contain no pending marker,
   placeholder, unsupported claim, or accidental private machine data;
6. every public link and checksum command was checked against the final text;
7. the developer explicitly approved, in one bounded decision:
   - both exact artifact hashes;
   - the final page and release-note text;
   - GitHub Releases as the publication channel;
   - the exact tag and release title;
   - whether GitHub must mark the release as a prerelease;
   - the checksum-manifest filename and contents; and
   - creation of the draft and upload of the three named assets; and
8. a later, separate approval authorizes changing the reviewed draft from
   non-public to public after post-upload verification.

Approval of a draft does not imply approval to publish it.

## Final local revalidation

Perform this on the machines holding the original candidates. Do not rebuild or
post-process them.

Ubuntu:

```sh
stat -c '%s' "AI Engine Room_0.1.0_amd64.deb"
sha256sum "AI Engine Room_0.1.0_amd64.deb"
dpkg-deb -f "AI Engine Room_0.1.0_amd64.deb" Package Version Architecture
```

Windows PowerShell:

```powershell
$installer = ".\AI Engine Room_0.1.0_x64-setup.exe"
(Get-Item -LiteralPath $installer).Length
(Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash
(Get-AuthenticodeSignature -LiteralPath $installer).Status
```

The local binary sizes, hashes, package metadata, and Authenticode result must
match the candidate evidence exactly. Independently verify that the approved
`SHA256SUMS.txt` contains those same two hashes paired with the exact dotted
hosted filenames. The manifest must be `198` bytes with SHA-256
`2b1c787237ad10d9262f552438d9093cea44aee8c3c3ae035c4b564da2caf2de`.
Do not copy from, generate from, or upload the obsolete draft manifest.

Stop if a candidate changed, disappeared, acquired a signature, or no longer
passes read-only package inspection. Do not rebuild under the same approved
release identity.

## Draft creation and upload

This section is authorized only for the exact approved values above and only
while the GitHub Release remains a non-public draft.

1. Confirm the authenticated GitHub account and repository target read-only.
2. Confirm that the approved tag and release title do not already exist. Do not
   overwrite, move, or reuse an existing tag or release.
3. Create a non-public draft release targeting exact source commit
   `a5482e9d51657a0cfb4471215a91750c5ba7db95` and using the approved prerelease
   setting.
4. Use
   `unsigned-preview-release-page-draft.md` as the exact reviewed GitHub
   Release body. Do not use the repository-relative release-notes copy as the
   release body. Do not weaken the unsigned warning or remove any qualification.
5. Upload exactly three assets:
   - `AI.Engine.Room_0.1.0_amd64.deb`;
   - `AI.Engine.Room_0.1.0_x64-setup.exe`; and
   - the approved final checksum manifest.
6. Do not upload source archives built locally, logs, screenshots, evidence
   bundles, signatures, certificates, updater files, MSI/AppImage packages, or
   any other artifact.
7. Keep the draft non-public while completing post-upload verification.

## Post-upload verification while still a draft

Using the hosting service's returned asset metadata and fresh downloads into a
new temporary directory:

1. confirm the repository, target source commit, tag, title, and prerelease
   setting;
2. confirm that the unsigned warning appears before the download links;
3. confirm all public links and exact filename-specific checksum commands;
4. confirm there are exactly three intended assets and no extras;
5. compare each hosted artifact's byte size with the evidence record;
6. download both hosted binaries and recompute SHA-256 independently;
7. confirm the downloaded Windows installer remains `NotSigned`;
8. validate the hosted checksum manifest against both downloaded files; and
9. obtain an independent read-only review of the draft and downloaded hashes.

Any mismatch is a stop condition. Remove or replace nothing without a new
developer decision. An asset replacement creates a new hosted object and must
repeat the entire verification; silent replacement under an approved hash or
release identity is prohibited.

## Public transition

Only a separate developer approval issued after the draft review may authorize
the public transition. Immediately before that transition, restate the exact
repository, tag, release title, source commit, three asset names, two binary
hashes, and prerelease setting.

After publication, verify from a signed-out/public view that:

- the release is visibly marked as an unsigned preview and prerelease as
  approved;
- the warning and limitations precede the downloads;
- all three assets are accessible with the approved names;
- fresh public downloads retain the approved sizes and hashes; and
- licence, NOTICE, third-party source/licence, support, privacy, and security
  links resolve correctly.

Record the public release URL and final verification outcome. Do not add an
updater, package-repository instruction, mirror, or additional announcement as
an implied follow-on action.

## Rollback and correction

### Before publication

If a non-public draft is wrong, keep it non-public and stop. With explicit
developer approval, remove the incorrect draft or assets. Correct the source
evidence or text locally, repeat independent review, and obtain a new draft
authorization before uploading again.

### After publication

Do not silently replace a public binary, move its tag, or reuse its checksum.
On a material error, suspected compromise, wrong artifact, or misleading page:

1. stop further promotion and preserve the observed public state and hashes;
2. ask the developer to choose the bounded response appropriate to the issue,
   such as clearly marking the release withdrawn, removing affected downloads,
   or removing the release;
3. use the security-reporting process for a security-sensitive issue;
4. state which exact hashes are affected and avoid claiming an unsigned file
   has authenticated publisher identity;
5. correct code or documentation through normal review and deterministic CI;
6. build and verify new artifacts under a new developer-approved release
   identity; and
7. never overwrite the historical evidence for the withdrawn hashes.

Deletion or withdrawal is not pre-authorized by this runbook. It requires an
explicit decision because it changes public state and may affect existing
users.
