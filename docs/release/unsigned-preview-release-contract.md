# Unsigned preview release contract

Status: approved planning and verification contract on 2026-08-30. This record
does not authorize an upload, GitHub Release, hosting change, publication,
distribution, or public-release action.

AI Engine Room may use a deliberately unsigned preview as its first public
binary release only after every gate in this contract passes and the developer
separately authorizes the exact artifacts and publication channel. The preview
is not a stable, production-ready, security-certified, or broadly compatible
release.

## Formats and trust boundary

- Windows is limited to the existing x86-64 NSIS installer. Its application and
  installer executables must remain Authenticode `NotSigned`; a self-signed
  certificate must not be substituted.
- Linux is limited to the existing Ubuntu 24.04 LTS x86-64 `.deb`. It is a
  direct-download package, not an APT repository, and makes no repository-trust
  or broad Debian-family compatibility claim.
- No updater, automatic download, package repository, Store submission,
  signing service, certificate, trust-policy change, or security-warning bypass
  is part of this contract.

Microsoft documents that an unsigned downloaded application can show
**Windows protected your PC**, that a user may be able to choose **Run anyway**,
and that enterprise policy or Smart App Control can prevent continuation:
<https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation>.

## Required candidate evidence

Before requesting publication approval, the exact candidate artifacts must:

1. be built from one clean, pushed `main` commit whose commit and tree IDs are
   recorded;
2. follow a passing deterministic GitHub Actions run for that exact commit;
3. pass the repository and target-specific licence checks, real-package payload
   checks, and read-only archive inspection;
4. record filename, format, architecture, byte size, and lowercase SHA-256;
5. pass install, native launch, bounded graphical/keyboard review, removal, and
   post-removal absence on the already accepted narrow Linux and Windows
   baselines;
6. be reviewed independently on Windows, including an explicit `NotSigned`
   result for the exact installer and installed executable; and
7. leave source control clean, with no generated binary staged or committed.

Earlier package or UI evidence does not transfer to a rebuilt artifact. The
licence-remediation artifacts recorded in
[`binary-distribution-license-notice-review.md`](binary-distribution-license-notice-review.md)
remain evidence inputs, not automatically selected public candidates.

## Required publication material

The proposed download or release page must visibly state **Unsigned preview**
before the download links and include:

- the exact source commit and a link to its public source;
- each filename, byte size, SHA-256, platform, architecture, and verified
  baseline;
- a prominent Windows explanation that the publisher will appear unknown and
  that SmartScreen or organizational policy may warn or block execution;
- instructions to proceed only when the file came from the official project
  page and its SHA-256 matches the published value;
- `Get-FileHash -Algorithm SHA256` and `sha256sum` examples that use the exact
  published filenames;
- an explicit warning not to disable SmartScreen, Smart App Control, antivirus,
  or organizational security policy and not to install a trust certificate;
- links to the licence, NOTICE, third-party source and licence material,
  support matrix, privacy boundary, security-reporting process, and known
  pre-release limitations; and
- a statement that a checksum detects an accidental or malicious file change
  relative to the page but is not a publisher signature.

The page must not tell users to accept a warning blindly, claim that the warning
is harmless, or imply that unsigned software has verified publisher identity.
If Windows does not offer a permitted continuation path, the supported outcome
is that the preview cannot be installed on that device.

## Stop conditions

Stop and obtain a new developer decision if candidate evidence fails, an exact
artifact changes, a third-party or platform rule changes, a signing identity is
introduced, a new distribution channel is proposed, or publication would
require weakening a user security control. Publication remains a distinct
approval after the candidate evidence and final page text are reviewed.
