# AI Engine Room user guide

This guide covers the bounded `v0.1.0-preview.1` public preview. AI Engine Room
helps you inspect supported local-AI runtimes and machine context. It does not
configure providers, choose a model, repair a system, or make decisions for
you.

## Before you install

The verified public-preview baselines are:

- Ubuntu 24.04 LTS x86_64 using the `.deb` package; and
- Windows 11 25H2 build 26200.7462 x64 using the NSIS installer.

Other operating-system versions, distributions, architectures, and machines
are not covered by the accepted evidence. macOS is not supported. Read the
precise [support matrix and limitations](../SUPPORT.md) before installing.

Download only from the official
[`v0.1.0-preview.1` release](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1).
The accepted assets are:

| Asset | Size in bytes | SHA-256 |
| --- | ---: | --- |
| `AI.Engine.Room_0.1.0_amd64.deb` | 4,722,942 | `9c75d669fd3dbebc4d0f72ee3d880258206f1adc0be19a15d29fabf6b1325c9e` |
| `AI.Engine.Room_0.1.0_x64-setup.exe` | 2,651,735 | `6bfa7b6aa4998efc3275eeae12917242526fb2dca8e970630d8b4f1e23f3b399` |
| `SHA256SUMS.txt` | 198 | `2b1c787237ad10d9262f552438d9093cea44aee8c3c3ae035c4b564da2caf2de` |

These preview packages are **unsigned**. Their checksums can show whether the
downloaded bytes match the accepted files, but a checksum is not a digital
signature and does not authenticate the publisher. Greg Weir is the developer;
Tartanleaf.com Inc. is the product and package publisher. The Windows package
does not contain an authenticated Authenticode publisher identity.

Do not disable or weaken a security control, install a trust certificate, or
bypass an organizational policy to install the preview. Continue only through
your platform's normal option after the official download matches all expected
details and you have decided that running unsigned preview software is
acceptable.

## Verify the download

Compare the complete hash output, filename, and byte size with the table above
and the release page. A partial match is not enough.

On Windows, open PowerShell in the download directory and run:

```powershell
Get-FileHash -Algorithm SHA256 ".\AI.Engine.Room_0.1.0_x64-setup.exe"
```

On Ubuntu, open a terminal in the download directory and run:

```sh
sha256sum "AI.Engine.Room_0.1.0_amd64.deb"
```

Stop if the filename, size, or complete SHA-256 value differs. Download a fresh
copy from the official release rather than trying to repair or reuse the file.

## Install and remove

### Ubuntu

From the directory containing the verified package, install it with:

```sh
sudo apt install "./AI.Engine.Room_0.1.0_amd64.deb"
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

Open the verified `AI.Engine.Room_0.1.0_x64-setup.exe` through the normal
Windows flow. Because platform policy and reputation state can vary, this guide
does not promise that a warning will or will not appear. Do not weaken a control
or bypass a policy to continue.

To remove the app, open **Settings → Apps → Installed apps**, find **AI Engine
Room**, and choose **Uninstall**. Confirm afterward that it no longer appears in
Installed apps.

## Your first session

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

## Reports and the clipboard

The Report workspace contains a sanitized, allow-listed plain-text preview.
**Copy report** writes exactly that preview to the operating-system clipboard.
AI Engine Room does not save, upload, send, read back, persist, or clear the
report. Other applications may read clipboard contents, so review the preview
before copying and handle it according to your own privacy needs.

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
