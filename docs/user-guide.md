# AI Engine Room user guide

This guide covers the `v0.1.1` stable release. AI Engine Room
helps you inspect supported local-AI runtimes and machine context. It does not
configure providers, choose a model, repair a system, or make decisions for
you.

## Before you install

The verified release baselines are:

- Ubuntu 24.04 LTS x86_64 using the `.deb` package; and
- Windows 11 25H2 build 26200.7462 x64 using the NSIS installer.

Other operating-system versions, distributions, architectures, and machines
are not covered by the accepted evidence. macOS is not supported. Read the
precise [support matrix and limitations](../SUPPORT.md) before installing.

Download only from the official
[`v0.1.1` release](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.1).
Download the platform package and `SHA256SUMS.txt` from that same release page.
The accepted release assets are:

| Platform | Exact hosted filename | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| Ubuntu | `AI.Engine.Room_0.1.1_amd64.deb` | 4,896,518 | `c73dac2320632bbb6b4a3f02c65943a95bf40cefe83c367e8e65b5b7136c617a` |
| Windows | `AI.Engine.Room_0.1.1_x64-setup.exe` | 2,762,354 | `e51fd579a5045599d99596f8b4bdfd10d3712b91b9110ed02fe72b3bc786dfb4` |

These packages are **unsigned**. Their checksums can show whether the
downloaded bytes match the accepted files, but a checksum is not a digital
signature and does not authenticate the publisher. Greg Weir is the developer;
Tartanleaf.com Inc. is the product and package publisher. The Windows package
does not contain an authenticated Authenticode publisher identity.

Do not disable or weaken a security control, install a trust certificate, or
bypass an organizational policy to install the application. Continue only through
your platform's normal option after the official download matches all expected
details and you have decided that running unsigned software is
acceptable.

## Verify the download

Compare the complete hash output, filename, and byte size with the table above,
the downloaded `SHA256SUMS.txt`, and the release page. A partial match is not
enough.

On Windows, open PowerShell in the download directory and run:

```powershell
Get-FileHash -Algorithm SHA256 ".\AI.Engine.Room_0.1.1_x64-setup.exe"
```

On Ubuntu, open a terminal in the download directory and run:

```sh
sha256sum "AI.Engine.Room_0.1.1_amd64.deb"
```

Stop if the filename, size, or complete SHA-256 value differs. Download a fresh
copy from the official release rather than trying to repair or reuse the file.

## Install and remove

### Ubuntu

From the directory containing the verified package, install it with:

```sh
sudo apt install "./AI.Engine.Room_0.1.1_amd64.deb"
```

The application should then be available through the desktop application menu.
To remove it later:

```sh
sudo apt remove ai-engine-room
```

You can verify removal with:

```sh
dpkg -s ai-engine-room
```

After removal, `dpkg` should report that `ai-engine-room` is not installed. An
`apt` message about unrelated automatically installed packages is not an
instruction from this project; review such packages separately rather than
removing them as part of this guide.

### Windows

Before opening the installer, confirm that its full SHA-256 matches the
`SHA256SUMS.txt` entry downloaded from the same official release. You may also
right-click the file and use the normal Microsoft Defender scan option. Open
the verified `AI.Engine.Room_0.1.1_x64-setup.exe` through the normal Windows
flow. Because it is unsigned, Windows may show an unknown-publisher or
reputation warning. Read the complete warning. If Windows provides a normal
**More info → Run anyway** path and your own or organizational policy permits
it, that is the bounded continuation path for this release. Do not turn off
SmartScreen or antivirus, change system-wide security settings, install a
certificate, or bypass organizational policy.

To remove the app, open **Settings → Apps → Installed apps**, find **AI Engine
Room**, and choose **Uninstall**. Confirm afterward that it no longer appears in
Installed apps.

## Your first session

![Conceptual AI Engine Room workflow showing passive observation and the separate per-run observed-inference gate](assets/ai-engine-room-workflow.svg)

_This source-derived workflow explains actions and boundaries; it is not a
native application screenshot or an exact control layout._

1. If you want provider observations, start a supported runtime yourself before
   opening AI Engine Room. The app does not start Ollama, LM Studio, or
   llama.cpp.
2. Launch AI Engine Room.
3. Choose **Refresh** to reacquire the supported provider and machine
   observations. Refresh is passive: it does not run or authorize inference.
4. Move among the workspaces to inspect the bounded evidence:
   - **Overview** summarizes current provider and machine observations.
   - **Models** shows provider-reported model identities, catalogue or loaded
     state, and qualified size or context metadata when available.
   - **Observed inference** offers an optional, separately disclosed and
     authorized observation for eligible Ollama or LM Studio models.
   - **Diagnose** compares the newest bounded observations and presents
     deterministic **Observation → Meaning → Safe next check** findings.
   - **Report** previews an allow-listed plain-text observation report and can
     copy exactly that preview after an explicit action.
5. Treat **Unavailable**, **Unknown**, **Not detected**, and **Failed** as
   meaningful limits on the evidence, not values that should be estimated.

## Optional observed inference

Observed inference is not required to use the application. For an eligible
Ollama or LM Studio model, the app first presents a disclosure and asks for
authorization for that run. If authorized, it sends one fixed synthetic prompt
with a bounded timeout and concurrency and no retry.

The result is a descriptive observation, not a benchmark. Ollama execution
location remains undetermined. LM Studio's API is reached through a
same-machine loopback endpoint, but exact compute placement is not independently
verified. An authorized LM Studio request may cause LM Studio itself to JIT-load
an unloaded model; AI Engine Room does not call model-management APIs.
llama.cpp is passive-only and is not eligible for observed inference.

## Reports, copying, and saving

The Report workspace contains a sanitized, allow-listed plain-text preview.
**Copy report** writes exactly that preview to the operating-system clipboard.
AI Engine Room does not upload, send, read back, or clear the report. Other
applications may read clipboard contents, so review the preview before copying
and handle it according to your own privacy needs.

In the native app, **Save report…** can create one plain-text `.txt` file in a
location you choose. It saves exactly the visible preview as UTF-8, with no
added metadata, and it will not replace a file that already exists. The app
does not remember the chosen location. A saved file remains until you remove
it and may be synchronized, backed up, indexed, or read by other software.
Saving is not available in browser/mock preview mode.

The save boundary uses a temporary staging file in the chosen directory. A
handled failure attempts to remove that file, but a process termination, power
loss, operating-system failure, remote filesystem, or unusual filesystem may
leave staging residue. If the app cannot confirm cleanup or completion, it
tells you to check the chosen location rather than claiming success.

## Observation history

The app retains at most the newest 12 startup and explicit-Refresh observation
bundles in memory for the current session. They form an ordinal observation
sequence with controlled gaps. They have no timestamps, background polling, or
regular sampling, and they reset when the app restarts. This is not continuous
monitoring or a time-based trend.

## If no provider appears

- Confirm that the provider is running; AI Engine Room does not start or manage
  it.
- Confirm that the provider matches the implemented fixed same-machine endpoint
  and configuration described in [SUPPORT.md](../SUPPORT.md).
- Use **Refresh** after the provider is ready.
- Preserve **Not detected**, **Unavailable**, **Unknown**, or **Failed** when
  that is what the application reports. Do not infer a hidden value.
- Do not expose a service to the LAN, widen a firewall, enable remote access, or
  change an organizational security policy just to make it visible to AI Engine
  Room.

For a problem within the supported scope, follow the project's
[support guidance](../SUPPORT.md). For terminology, see the
[glossary](glossary.md).
