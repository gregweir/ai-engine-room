# Unsigned preview exact-artifact physical acceptance

Status: operator checklist and evidence template. Do not execute until the
developer is physically present at both accepted machines. Completion of this
checklist does not authorize upload, publication, distribution, or release.

## Fixed candidate identity

All observations must apply to these exact files. Stop if a filename, byte
size, SHA-256, source identity, or platform baseline differs.

| Platform | Filename | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| Ubuntu 24.04 LTS x86-64 | `AI Engine Room_0.1.0_amd64.deb` | 4,722,942 | `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e` |
| Windows 11 25H2 build 26200.7462 x64 | `AI Engine Room_0.1.0_x64-setup.exe` | 2,651,735 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |

- Source commit: `a5482e9d51657a0cfb4471215a91750c5ba7db95`
- Source tree: `c1e8092cb5caa75a77f7e53fdc712eae6f72b318`
- Deterministic CI: <https://github.com/gregweir/ai-engine-room/actions/runs/33309714136>

Do not rebuild either file during acceptance. Earlier package or UI evidence
does not transfer to these hashes.

## Safety boundary

- Do not start, stop, probe, or configure Ollama, LM Studio, or llama.cpp.
- Do not authorize or run inference.
- Do not exercise **Copy report** or inspect clipboard contents.
- Do not disable SmartScreen, Smart App Control, antivirus, or organizational
  policy, and do not install a trust certificate.
- Do not record a username, hostname, private path, address, machine-specific
  memory value, provider inventory, model identity, generated output, or raw
  environment dump in durable evidence.
- A provider being absent or unreachable is not a package failure.
- Stop on an identity mismatch, policy block, unexpected installer/package
  behavior, automatic inference/provider action/clipboard write, uncontrolled
  error, or material UI/accessibility defect.

## Ubuntu procedure

From the verified public checkout:

```sh
sha256sum "target/release/bundle/deb/AI Engine Room_0.1.0_amd64.deb"
dpkg-deb -f "target/release/bundle/deb/AI Engine Room_0.1.0_amd64.deb" Package Version Architecture
sudo apt install "./target/release/bundle/deb/AI Engine Room_0.1.0_amd64.deb"
dpkg-query -W -f='${Status} ${Version}\n' ai-engine-room
```

Expected metadata is package `ai-engine-room`, version `0.1.0`, architecture
`amd64`, followed after installation by `install ok installed 0.1.0`. An `_apt`
warning that its sandbox user cannot read the local source file may be recorded;
it is not itself a failure when package verification, unpacking, and
configuration complete normally.

Launch **AI Engine Room** natively from the application menu. Complete the
common graphical and passive-behaviour review below. Close the application and
pause before removal so the installed-package state can receive a bounded
read-only verification.

After that verification, remove only the application package:

```sh
sudo apt remove ai-engine-room
dpkg -s ai-engine-room
```

The final command must report that `ai-engine-room` is not installed. Do not
run `apt autoremove` as part of this acceptance.

## Windows procedure

From native PowerShell in the verified public checkout:

```powershell
$installer = ".\target\release\bundle\nsis\AI Engine Room_0.1.0_x64-setup.exe"
(Get-Item -LiteralPath $installer).Length
(Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash
(Get-AuthenticodeSignature -LiteralPath $installer).Status
Start-Process -FilePath $installer
```

The expected installer Authenticode status is `NotSigned`. Record whether
Windows presents an unknown-publisher, SmartScreen, antivirus, or policy
warning. Continue only through a normal Windows option allowed on the machine.
If policy blocks installation without an allowed continuation, stop and record
the block.

Complete the normal per-user installer flow and launch **AI Engine Room**
natively. Complete the common graphical and passive-behaviour review below.
While the application remains open, verify the installed executable without
recording its private path:

```powershell
$app = Get-Process aiengineroom -ErrorAction Stop | Select-Object -First 1
(Get-AuthenticodeSignature -LiteralPath $app.Path).Status
```

The installed executable must report `NotSigned`. Close the application and
pause before removal so an independent reviewer can confirm the installer
identity, installed-executable status, installed-app entry, and absence of a
running application or setup process.

After that verification, uninstall through the normal Windows **Installed
apps** flow. Confirm that **AI Engine Room** no longer appears there. Do not
invent a manual filesystem-cleanup requirement. A bounded read-only final check
must confirm no Installed apps entry or running application/setup process.

## Common graphical, keyboard, and passive-behaviour review

Record pass/fail for each observation:

1. The native product identity, version where exposed, and approved C2R1 icon
   appear correctly on the operating-system surfaces that are actually shown.
2. The dashboard is native and contains no browser-fixture or mock banner.
3. Startup and navigation cause no automatic inference, provider/model action,
   or clipboard write.
4. Available memory, total memory, native CPU architecture, provider-reported
   model size, and configured-context evidence use controlled values or
   unavailable/unknown states without fabricating zero, KV-cache bytes, runtime
   overhead, VRAM capacity, compute placement, model fit, or headroom.
5. All five workspaces—**Overview**, **Models**, **Observed inference**,
   **Diagnose**, and **Report**—are reachable by keyboard and retain visible
   focus.
6. Disclosure controls operate by keyboard and do not accidentally authorize
   inference.
7. Normal, narrow-window, and enlarged-text/zoom presentation remain readable
   without material clipping, overlap, unreadable text, or unreachable controls.
8. The application closes normally.

These are bounded acceptance observations, not a WCAG conformance, broad
compatibility, privacy-certification, security-certification, performance, or
provider-functionality claim.

## Completion evidence template

Keep the pending markers until direct observation is complete.

```text
Exact source commit/tree: PASS | FAIL | PENDING
Linux filename/bytes/SHA-256: PASS | FAIL | PENDING
Linux install/package metadata: PASS | FAIL | PENDING
Linux native UI/keyboard/passive behavior: PASS | FAIL | PENDING
Linux removal/post-removal absence: PASS | FAIL | PENDING
Windows filename/bytes/SHA-256: PASS | FAIL | PENDING
Windows installer Authenticode NotSigned: PASS | FAIL | PENDING
Windows warning or policy outcome: <bounded description | PENDING>
Windows install/product identity: PASS | FAIL | PENDING
Windows native UI/keyboard/passive behavior: PASS | FAIL | PENDING
Windows installed executable NotSigned: PASS | FAIL | PENDING
Windows removal/post-removal absence: PASS | FAIL | PENDING
Independent final review: PASS | FAIL | PENDING
Provider access or inference performed: NO required; otherwise STOP
Clipboard action performed: NO required; otherwise record approved exception
Unexpected behavior or limitation: NONE | <bounded description> | PENDING
```

If every required field passes, preserve the completed evidence without adding
private machine data. Publication still requires separate developer approval
of the exact artifacts, final release-page text, and publication channel.
