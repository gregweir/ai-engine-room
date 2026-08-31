# AI Engine Room roadmap

AI Engine Room is a privacy-conscious desktop utility that helps people
understand local-AI runtime and resource state through **Observe → Explain →
Diagnose → Report**. It reports bounded evidence and qualifications; it does
not manage providers, promise model fit, infer compute placement, or act as a
hardware benchmark.

## Current product goals

- Make local runtime, model, and machine context understandable to non-experts.
- Distinguish observed facts from calculations, estimates, and unavailable evidence.
- Keep collection passive by default and inference explicitly authorized.
- Preserve report-safe output that excludes sensitive or unnecessary machine details.
- Maintain bounded native behavior on the verified Linux and Windows
  public-preview baselines.

## Available now

- A manual **Observe → Explain → Diagnose → Report** workflow with no continuous
  polling, account, telemetry, upload, or application persistence.
- Bounded observation of supported same-machine Ollama and LM Studio APIs, plus
  passive traditional single-model llama.cpp detection on its tested Ubuntu
  scope.
- Optional, per-run-authorized observed inference for eligible Ollama and LM
  Studio models using one fixed synthetic prompt, bounded timeout and
  concurrency, and no retry.
- Provider-qualified model metadata, platform-native memory and CPU-architecture
  context, bounded session observation history, deterministic findings, and an
  allow-listed plain-text report preview.
- The public, unsigned `.deb` and NSIS
  [`v0.1.0-preview.1` prerelease](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1)
  on the exact verified Ubuntu and Windows baselines.
- Deterministic frontend and Rust checks on Ubuntu and Windows, including
  ephemeral package and licence-payload verification without artifact upload,
  provider access, inference, signing, publication, or release.

## Next

The current sequence is documentation-led and separately gated:

- **1W-A — complete:** the public front door now states the product's purpose,
  intended audience, and safety boundaries in plain language.
- **1W-B — complete:** a concise quick start, user guide, glossary, and this
  public roadmap structure were added under the
  [bounded contract](design/milestone-1w-b-guides-roadmap-contract.md).
- **1W-C — complete:** one accessible, source-derived workflow diagram was added
  under the [bounded visual contract](design/milestone-1w-c-workflow-visual-contract.md).
  It is explicitly not a native screenshot or readiness claim.
- **1W-D — preparation complete; participant round deferred:** a
  privacy-conscious, moderated non-specialist validation kit is available under
  the [bounded preparation contract](design/milestone-1w-d-validation-preparation-contract.md).
  The developer does not currently have three to five willing non-specialist
  participants or a suitable recruitment path. No session evidence or
  resulting readiness claim exists, and this deferred round does not block
  separately gated work.
- **1X-A — feasibility documented; verification not authorized:** the evidence
  and stop conditions for evaluating WinGet as the first package-manager
  channel are recorded under the
  [bounded feasibility contract](design/milestone-1x-a-winget-feasibility-contract.md)
  and [WinGet assessment](distribution/winget-feasibility-assessment.md). This
  preparation does not execute or publish a package.
- **1X-B — verification preflight stopped safely:** the developer authorized
  one bounded silent-install feasibility run, but the available Windows Home
  host has no supported Windows Sandbox and no equivalent disposable Windows
  environment was established. The run stopped before installer download or
  execution. The
  [preflight record](distribution/winget-silent-install-preflight-record.md)
  also records the still-open publisher-page discoverability and release-
  identity clarity work. A suitable disposable environment and renewed
  developer gate are required before another run.
- **1X-C — identity and publisher-page planning documented:** current WinGet
  guidance supports using the installed Apps & Features version as
  `PackageVersion`; the existing preview tag is a separate channel label, not
  automatically a manifest-version failure. The
  [readiness plan](distribution/winget-identity-and-publisher-page-plan.md)
  records a tentative durable package identity, an official publisher-page
  content contract, candidate evidence requirements, and maintenance
  responsibility. That plan did not approve a website change, manifest,
  release, submission, or installer run; the later website work received its
  own separate approval.
- **1X-D — publisher page published and verified:** a
  [repository-local copy draft](distribution/winget-publisher-page-copy-draft.md)
  was separately reviewed and used to prepare the public
  [AI Engine Room publisher page](https://www.tartanleaf.com/ai-engine-room/).
  The public [Tools page](https://www.tartanleaf.com/tools/) now provides a
  stable route to it. The live copy was later simplified for Tartanleaf's
  non-technical audience while retaining exact downloads, hashes, warnings,
  product limits, and the explicit package-manager status.
- **1X-E — publication record complete:** the
  [bounded reconciliation contract](design/milestone-1x-e-publisher-page-publication-record-contract.md)
  and [publication record](distribution/tartanleaf-publisher-page-publication-record.md)
  distinguish the authoritative live CMS page from its historical source
  draft and record normal and narrow public verification. The publisher page
  closes the official-source discoverability gap only. The tentative
  `Tartanleaf.AIEngineRoom` identifier is still not reserved or approved, and
  no silent-install evidence, manifest, submission, signing, or WinGet
  availability claim follows from publication.
- **1X-F — disposable-Windows verification stopped without conclusion:** the
  physical Windows Home host still has no supported Windows Sandbox or other
  local disposable Windows desktop. A
  [bounded contract](design/milestone-1x-f-disposable-windows-silent-install-contract.md)
  authorized one branch-scoped run in a fresh GitHub-hosted Windows VM against
  the exact public preview. Candidate identity and registered installation were
  observed, but the run stopped before launch when the expected installed
  executable path did not exist. Registered cleanup completed. The
  [verification record](distribution/winget-disposable-runner-verification-record.md)
  establishes no silent-install or silent-removal feasibility conclusion. A
  revised procedure and another run require a new developer gate; no manifest,
  submission, signing action, or WinGet availability claim is authorized.
- **1X-G — executable-discovery correction complete; no run authorized:** the
  [preparation contract](design/milestone-1x-g-executable-discovery-preparation-contract.md)
  replaces the failed hard-coded executable-path assumption with bounded
  registry and installed-file evidence. It records relative executable
  inventory, excludes the registered uninstaller, requires a unique candidate,
  and preserves cleanup after any stopped installer execution. This milestone
  creates no installer workflow and authorizes no run. A second disposable
  execution still requires a new explicit developer gate.
- **1X-H — one-run gate consumed; stopped on external connection:** the
  [contract](design/milestone-1x-h-disposable-windows-rerun-contract.md) and
  exact workflow commit received separate developer approval. Installation,
  registered identity, DisplayIcon resolution, and executable metadata passed
  on the one disposable Windows job. The bounded launch then observed one
  established non-loopback connection and stopped; registered cleanup passed.
  The workflow was removed and no retry is authorized. The
  [verification record](distribution/winget-disposable-runner-verification-record.md)
  makes no claim about transmitted content or destination purpose and grants no
  complete silent-install/removal conclusion, manifest, or submission authority.
- **1X-I — network-attribution preparation complete; no run authorized:** the
  [static contract](design/milestone-1x-i-network-attribution-preparation-contract.md)
  keeps application launch in the readiness question, preserves the first
  non-loopback-connection stop, and corrects the inactive procedure to record
  the owner process and immediate parent when the owner PID can be resolved.
  It does not allow-list the observed address, infer destination purpose,
  create a workflow, authorize another run, or begin product network
  observability. Any one-run 1X-J proposal requires a new exact developer gate.
- **1X-J — final one-run attribution gate consumed; attributed stop:** the
  [contract](design/milestone-1x-j-network-attribution-run-contract.md), exact
  execution change, and exactly one run received separate developer approval.
  During bounded launch, the disposable Windows job attributed the established
  connection to a WebView2 descendant and then completed registered cleanup.
  The one-time workflow was removed. The
  [verification record](distribution/winget-disposable-runner-verification-record.md)
  makes no claim about destination purpose or transmitted content and grants no
  retry, complete silent-install/removal conclusion, manifest, submission, or
  product network-observability authority.
- **1X-K — WinGet disposition complete:** the
  [closure contract](design/milestone-1x-k-winget-disposition-contract.md)
  and [disposition record](distribution/winget-disposition.md) recommend
  deferring the exact `v0.1.0-preview.1` candidate from WinGet. The result is
  not a prediction of Microsoft acceptance or rejection. It preserves the
  final attributed stop, authorizes no fourth run or manifest, and identifies a
  documentation-only Snap Store feasibility study as the next separately gated
  distribution-planning candidate.
- **1Y-A — Snap Store feasibility complete:** the
  [documentation-only contract](design/milestone-1y-a-snap-store-feasibility-contract.md)
  and [feasibility assessment](distribution/snap-store-feasibility-assessment.md)
  find the route feasible in principle but not packaging-ready. Current
  official documentation identifies a free developer account and store-signed
  assertions, with no conventional certificate purchase or submission fee
  identified in the normal flow. Strict-confinement behavior remains unproved.
  The recommended next slice is a separately approved static Snapcraft
  packaging proposal; no account, name, build, install, upload, store contact,
  or release is authorized.
- **1Y-B — static Snapcraft proposal complete:** the
  [bounded contract](design/milestone-1y-b-snapcraft-static-proposal-contract.md),
  [strict/devel definition](../snap/snapcraft.yaml), desktop launcher, static
  safeguards, and [proposal record](distribution/snapcraft-static-proposal.md)
  provide an inspectable amd64 source-build hypothesis. Nothing was expanded,
  downloaded, built, installed, registered, or uploaded. The GNOME extension's
  generated permissions and the complete build remain unproved. The next
  recommended step is a separately approved one-build gate that stops before
  installation or any store action.
- **1Y-C — one-run gate consumed; expansion stop:** the
  [preparation contract](design/milestone-1y-c-snapcraft-one-build-preparation-contract.md)
  and [verification record](distribution/snapcraft-expansion-verification-record.md)
  document exactly one disposable Ubuntu 24.04 amd64 expansion. Snapcraft
  `9.0.1` did not generate the anticipated `mount-observe` or
  `calendar-service` app plugs, but it omitted the reviewed platform mapping
  and added extension-managed local sources plus an unpinned
  `canonical/gpu-snap.git` source. The procedure stopped before compilation;
  no artifact, installation, account, upload, or store action occurred. The
  workflow and procedure were removed, and no retry is authorized.

A code-signing study remains separate and non-blocking. No certificate
application or purchase, publisher-identity decision, MSIX/Store work, or
signed release is authorized.

## Under consideration — not authorized

- Evidence-scope and wording refinements based on actual non-specialist
  validation.
- A safe report export format beyond explicit clipboard copy.
- GPU observation only after a bounded evidence and provenance contract.
- Additional providers only after a provider-admission contract and evidence of
  user need.
- MSIX and Microsoft Store feasibility.
- Future signing through a no-cost eligibility route such as SignPath
  Foundation or through paid organizational identity, after the publisher
  certificate identity is deliberately resolved.
- A bounded WinGet silent-install feasibility run only after a supported
  disposable Windows environment and exact candidate receive a renewed gate,
  followed by a separate submission decision only if that candidate passes.
- A separately gated static Snapcraft source-boundary revision deciding whether
  to pin and justify all GNOME-extension generated inputs, replace the
  extension with explicit least-privilege packaging, or defer the channel. No
  additional expansion or build is authorized. PPA and official Debian/Ubuntu
  source-package work remain later, higher-effort possibilities.
- A documentation-led
  [bounded network-observability feasibility study](design/bounded-network-observability-planning-note.md)
  to determine whether the verified Windows and Linux platforms can provide
  useful process-associated connection evidence without packet-content
  inspection, persistent monitoring, or elevated privileges in the main
  application. It remains behind the active distribution sequence and requires
  a separate developer gate.

These are possibilities, not commitments, schedules, or implementation
authority.

## Explicitly outside the current product

- Agentic or autonomous actions and decisions.
- Starting, stopping, installing, unloading, reconfiguring, or otherwise
  managing providers or models.
- Continuous monitoring, background sampling, alerts, or persistent history.
- Benchmark scores, model-fit or headroom recommendations, root-cause claims,
  automated repair, or independently verified compute placement.
- LAN, cloud, remote endpoint, authenticated endpoint, or custom endpoint
  configuration.
- Unbounded, automatic, or undisclosed inference.

## Completed delivery record

1. Milestone 1U is closed: total memory and native CPU architecture include the
   Windows Clippy correction and the developer decision on 225% Text size evidence.
2. Milestone 1V is closed at accepted source revision
   `1f19ea0f249a3b07a55d6fb53a70d0c45cdb7e51`: provider-reported model-size
   and configured-context presentation passed deterministic CI, bounded native
   Linux verification, and independent Windows review under
   [`design/milestone-1v-contract.md`](design/milestone-1v-contract.md). KV-cache
   bytes, runtime overhead, physical VRAM capacity, and compute placement remain
   unavailable or unknown; do not calculate them or make fit/headroom claims.
3. The separately approved current-source packaging-readiness contract was
   completed at source revision `2293b336eaa314f4fd285737a8470a5b9abd151a`.
   Its bounded `.deb` and NSIS builds, independent review, and exact-artifact
   install/launch/removal acceptance are complete historical evidence. Those
   artifacts remain local, unsigned, and unpublished; their acceptance does
   not transfer to a rebuilt preview candidate.
4. The fuller
   [`binary-distribution licence and notice review`](release/binary-distribution-license-notice-review.md)
   and its remediation are complete. Rebuilt `.deb` and NSIS licence payloads
   passed deterministic CI and independent package inspection.
5. The developer approved an
   [`unsigned preview release contract`](release/unsigned-preview-release-contract.md)
   and a bounded
   [`SignPath Foundation eligibility assessment`](release/signpath-eligibility-assessment.md).
   The first public preview remains unsigned; no signing action was taken.
   SignPath is a plausible no-cost route but is not application-ready. Greg
   Weir is the developer and Tartanleaf.com Inc. is the product/package
   publisher; whether a future SignPath certificate may instead authenticate
   SignPath Foundation remains undecided.
6. Exact unsigned-preview candidates were built and inspected from pushed
   `main` revision `a5482e9d51657a0cfb4471215a91750c5ba7db95` after passing
   deterministic CI. Their identities and the approved release-page text are
   recorded locally on a separate documentation branch. Physical exact-artifact
   install/UI/removal acceptance, installed-Windows-executable verification,
   and independent final review passed. The bounded Linux process-close
   deviation and its completed correction are recorded in the candidate
   evidence. The developer approved the exact artifacts, final text, GitHub
   Releases channel, `v0.1.0-preview.1` prerelease identity, `SHA256SUMS.txt`,
   and creation and upload of a non-public three-asset draft. After GitHub
   normalized spaces in the uploaded binary names, the developer approved the
   resulting dotted hosted filenames; the binary hashes and accepted bytes are
   unchanged.
7. The independently verified draft received a later, separate public-transition
   approval and is now the public
   [`AI Engine Room 0.1.0 unsigned preview`](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1).
   Fresh public downloads, checksums, unsigned status, pinned documentation
   links, and an independent public review all passed. The exact outcome is in
   [`the publication record`](release/unsigned-preview-publication-record.md).
   No further promotion, additional channel, asset replacement, signing action,
   or follow-on release is authorized. Future signing certificate identity,
   broader provider/platform coverage, and macOS remain later decisions.

No roadmap entry authorizes implementation by itself. The current milestone
state and developer gate control execution.
