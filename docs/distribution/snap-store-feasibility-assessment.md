# Snap Store feasibility assessment

Status: **FEASIBLE IN PRINCIPLE; NOT PACKAGING-READY; NO EXTERNAL ACTION
AUTHORIZED** on 2026-08-31.

## Recommendation

Keep the Snap Store as the next Linux package-manager candidate. Its normal
publishing flow appears financially proportionate for this free product: the
official documentation calls for a free developer account, a globally unique
registered name, an uploaded package, store review, and a release channel. The
reviewed flow does not identify a package-submission fee or require the
developer to buy a conventional code-signing certificate.

That is not yet a recommendation to publish. First prepare a separate,
developer-approved static Snapcraft prototype, then—under another exact gate—
build and test it with **strict confinement** on a disposable or designated
Linux environment. Publication decisions must wait for that evidence.

## Cost and signing interpretation

Current official documentation establishes:

- authors create a free developer account before engaging with the Snap Store;
- a snap name is globally unique and cannot be changed after registration;
- the store assigns revisions and uses digitally signed assertions to identify
  and validate snaps and store policy; and
- uploaded snaps are reviewed and released through channels.

This differs from Windows Authenticode. The reviewed normal Snap Store flow
does not ask the developer to purchase and apply a public-trust certificate to
the package. Store assertions provide the store-mediated trust and integrity
mechanism. That finding is limited to the current public documentation and
must be checked again immediately before any account or publication action.

No certificate or submission charge was identified, but the route is not
cost-free in the broader sense. Packaging, build infrastructure, testing,
updates, support, security response, and store maintenance consume developer
time and may create future costs. Account eligibility, identity presentation,
terms, taxes, private-store services, and any future policy changes have not
been evaluated or accepted.

Greg Weir remains the developer. Tartanleaf.com Inc. remains the intended
product and package publisher. How the store displays and verifies that
publisher identity is untested and requires review before registration.

## Product benefits and tradeoffs

A successful Snap route could provide:

- package-manager discovery, installation, removal, and channel selection;
- store assertions and immutable store revision identities;
- a Linux distribution route without a conventional paid signing certificate;
  and
- automatic delivery of later released revisions.

The tradeoffs are material:

- Snap requires its own packaging definition because the current Tauri bundle
  target is Debian only;
- a Snap must be maintained alongside the Debian package;
- snapd is not present or preferred on every Linux distribution;
- store-installed snaps refresh automatically by default, while AI Engine Room
  currently has no application-managed updater; and
- confinement can change what the app can observe, so existing Debian evidence
  cannot be carried over.

Snap documentation says snapd checks for updates four times per day by default,
with user and administrator controls for holding or scheduling refreshes. Any
future product page must explain that Snap updates are managed by snapd rather
than suggesting that the application itself checks for updates.

## Current application fit

The tracked application has a comparatively narrow native boundary:

- the desktop frontend receives only Tauri core defaults and clipboard-write
  permission;
- Ollama, LM Studio, and traditional single-model llama.cpp adapters use fixed
  numeric loopback endpoints;
- the Linux system evidence path reads `MemAvailable` and `MemTotal` from
  `/proc/meminfo`; and
- the current Tauri configuration emits a Debian bundle, not a Snap.

These facts make strict confinement plausible, but they do not prove it works.
The Snap `network` interface allows outbound client access and auto-connects,
which should cover a client connection in principle, including to loopback.
Only a running confined artifact can establish that each supported local
provider remains reachable with the intended behavior.

The first prototype should request only the minimum interfaces justified by
the executable. Likely areas to evaluate include `network`, `desktop`,
Wayland/X11, graphics, and any clipboard or desktop-portal interface actually
required by the packaged WebKitGTK application. This assessment does not select
a final plug set and does not authorize privileged observation interfaces.

## Strict-confinement evidence gaps

A later bounded prototype must answer all of these without falling back to
classic confinement merely for convenience:

1. Does the WebKitGTK/Tauri application launch and render correctly on the
   verified Ubuntu baseline?
2. Does explicit clipboard copy work without broad filesystem or desktop
   access?
3. Can the app reach each supported numeric loopback provider endpoint through
   the minimum network interface?
4. Is `/proc/meminfo` readable, and do `MemAvailable` and `MemTotal` retain the
   intended whole-system meaning inside the Snap's mount and process view?
5. Do desktop entry, icon, application name, version, publisher text, and
   licence/notice payload appear correctly?
6. Are install, launch, close, removal, and post-removal absence behavior
   bounded and repeatable?
7. Does the packaged artifact introduce any new connection, persistence,
   updater, telemetry, or report-content behavior?

Failure of one item would identify a packaging or interface problem, not
automatically justify classic confinement. Classic snaps have substantially
broader host access, require manual store approval, and require users to install
with `--classic`. A classic request would therefore need a later technical and
product review supported by evidence that strict confinement cannot meet the
product's bounded purpose.

Provider access and observed inference must remain separate. A first packaging
probe can validate launch, UI, static report behavior, and loopback connection
capability without sending an inference request. Any provider call or inference
run requires its own exact authorization and test data boundary.

## Proposed package identity boundary

`ai-engine-room` is only a tentative human-readable name candidate. Its
availability has not been checked and it must not be registered under this
assessment. Registration is an external, effectively durable choice because
official documentation says the globally unique registered name cannot be
changed.

The first technical prototype should be amd64-only and target the current
verified Ubuntu baseline. Architecture expansion should follow demonstrated
need and independent evidence. The current `0.1.0` source/package identity may
inform a prototype, but the existing `v0.1.0-preview.1` release must not be
silently repackaged or described as a Snap candidate. Any public Snap needs a
new immutable candidate and an aligned source, artifact, version, publisher,
licence, and release record.

## Separately gated sequence

The proportionate next sequence is:

1. **Static packaging proposal:** add a reviewable Snapcraft definition and
   supporting metadata without installing tools, building, registering, or
   uploading anything.
2. **Bounded build gate:** after exact approval, build once in a declared Linux
   environment and inspect the artifact independently.
3. **Strict-confinement acceptance gate:** after separate approval, test exact
   artifact identity, installation, launch, UI, clipboard, system evidence,
   provider reachability without inference, close, removal, and absence.
4. **Identity decision:** only after technical success, decide the store name,
   publisher presentation, account, terms, maintenance responsibility, update
   policy, and support route.
5. **Private or edge evaluation:** if still desired, authorize any account,
   registration, upload, automated review, and limited channel use explicitly.
6. **Public release decision:** separately approve exact bytes, channel, listing
   copy, evidence, hashes, provenance, support expectations, and public
   transition.

Each gate is independent. No unused authority carries forward.

## Disposition

The Snap Store is a better near-term feasibility candidate than reopening the
consumed WinGet investigation because it does not present the same conventional
certificate-cost concern and offers a native Linux distribution route. The
remaining uncertainty is substantial but testable through a small
strict-confinement prototype.

Accordingly, retain Snap on the roadmap and recommend the static packaging
proposal as the next separately approved implementation slice. Do not register
a name, create an account, build, install, upload, or contact the store yet.

## Sources checked on 2026-08-31

- [Snap publishing process](https://documentation.ubuntu.com/snapcraft/stable/explanation/snap-publishing-process/)
- [Register a snap](https://snapcraft.io/docs/registering-your-app-name/)
- [Assertions](https://snapcraft.io/docs/explanation/security/assertions/)
- [Snap security policies](https://snapcraft.io/docs/explanation/security/security-policies/)
- [Snap confinement](https://snapcraft.io/docs/explanation/security/snap-confinement/)
- [Classic confinement](https://snapcraft.io/docs/explanation/security/classic-confinement/)
- [Network interface](https://snapcraft.io/docs/reference/interfaces/network-interface/)
- [Desktop interface](https://snapcraft.io/docs/reference/interfaces/desktop-interface/)
- [Manage updates](https://snapcraft.io/docs/how-to-guides/manage-snaps/manage-updates/)
- [Craft a pre-built app](https://documentation.ubuntu.com/snapcraft/stable/how-to/integrations/craft-a-pre-built-app/)
- [`src-tauri/tauri.conf.json`](../../src-tauri/tauri.conf.json)
- [`src-tauri/capabilities/default.json`](../../src-tauri/capabilities/default.json)
- [`src-tauri/src/platform/linux.rs`](../../src-tauri/src/platform/linux.rs)
