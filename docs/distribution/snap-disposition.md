# Snap distribution disposition

Status: **DEFERRED; NO ADDITIONAL SNAPCRAFT RUN AUTHORIZED** on 2026-08-31.

## Decision

AI Engine Room will not pursue Snap distribution for the current
`v0.1.0-preview.1` release sequence. The project will continue to support the
already published direct `.deb` and NSIS preview downloads only on their exact
verified baselines.

This is a proportional maintenance decision. It is not a prediction that the
Snap Store would accept or reject the application, a claim that Snapcraft's
generated inputs are unsafe, or a permanent rejection of Snap as a possible
future channel.

## Evidence

The documentation-only feasibility study found Snap publication feasible in
principle and identified no conventional certificate purchase or normal
submission fee. The subsequent static proposal used strict confinement and a
development grade, limited the declared application plugs, and retained the
project's licence payload.

The separately approved one-run gate then invoked `snapcraft
expand-extensions` exactly once on a disposable Ubuntu 24.04 amd64 runner. The
run stopped before compilation because the expanded plan:

- omitted the reviewed `platforms` mapping; and
- added extension-managed inputs, including
  `https://github.com/canonical/gpu-snap.git`, without a visible immutable
  commit or tag identity.

The same expansion did not add the previously anticipated `mount-observe` or
`calendar-service` application plugs. That observation is limited to the
recorded Snapcraft 9.0.1 revision and does not establish future behavior.

No build command ran. No `.snap` existed, and nothing was installed, launched,
uploaded, registered, or submitted. The one-time workflow and procedure were
removed, and the consumed gate grants no retry.

## Why defer

Both technically plausible corrections create ongoing work:

- retaining the GNOME extension would require immutable, reviewable identities
  and transformation rules for every generated input; or
- replacing it would require explicit desktop-runtime and GPU packaging that
  the project would need to maintain and reverify.

The project has no recorded user demand that currently justifies that cost.
The existing direct Linux package already provides a bounded public preview for
the verified Ubuntu baseline. Deferral preserves effort for the product's core
observation and explanation goals.

## Retained proposal status

The tracked `snap/snapcraft.yaml`, launcher, static proposal, and expansion
record remain as historical, inspectable evidence. They are not a supported
package definition, release candidate, store submission, or authorization to
run Snapcraft. Their retention must not be presented as Snap availability.

## Reconsideration boundary

A future Snap investigation requires evidence of meaningful demand and a new
developer-approved contract. It must establish a complete immutable source
boundary, choose and justify an extension or explicit-packaging approach, and
repeat separately gated expansion, build, package inspection, install, UI,
provider, removal, and independent review. Account, name registration, upload,
and release would remain later independent decisions.

Until then, no additional Snapcraft expansion, build, installation, account,
registration, upload, store contact, or release is authorized.

## References

- [Milestone 1Y-D disposition contract](../design/milestone-1y-d-snap-disposition-contract.md)
- [Snap Store feasibility assessment](snap-store-feasibility-assessment.md)
- [Static Snapcraft packaging proposal](snapcraft-static-proposal.md)
- [Snapcraft expansion verification record](snapcraft-expansion-verification-record.md)
