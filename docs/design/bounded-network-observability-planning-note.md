# Bounded network-observability planning note

## Status

Planning direction recorded on 2026-08-31; documentation feasibility completed
under the [Milestone 1Z-A contract](milestone-1z-a-network-observability-feasibility-contract.md),
and evidence-model and synthetic-probe preparation completed under the
[Milestone 1Z-B contract](milestone-1z-b-network-observability-probe-preparation-contract.md).
This note remains a planning boundary, not an implementation contract. It
authorizes no product, platform probe, privilege, packaging, release, or
external action.

The package-manager investigation is closed for the current preview. The
[platform assessment](../research/network-observability-platform-feasibility.md)
finds a bounded TCP probe plausible on both verified platforms while excluding
UDP from the first cross-platform slice and preserving conditional Linux
process attribution. Product adoption remains undecided.

## Product question

AI Engine Room currently helps a non-specialist understand what a local-AI
environment is running and how bounded observations behave. A future capability
might also help the user understand where supported AI-related processes appear
to communicate.

The useful questions are:

- Is the observed AI interaction using a same-machine endpoint?
- Was any relevant externally addressed connection observed during an explicit
  observation window?
- Which supported AI-related process can reliably be associated with the
  observation?
- Is the destination address scope loopback, local/private-network, externally
  addressed, or unresolved?
- What does the evidence support, and what remains unknown?

This direction fits **Observe → Explain → Diagnose → Report** only if it
preserves the product's distinction between observed fact, qualified inference,
and unavailable evidence.

## Smallest useful feasibility target

The first possible future gate should be a documentation-led, read-only
feasibility study. It should determine whether the verified Windows and Linux
platforms can expose useful process-associated connection metadata without
packet-content inspection, persistent monitoring, or elevated privileges in
the main application.

If later authorized and supported by that evidence, the smallest product slice
would be an explicit, short, automatically ending observation window that:

- retains observations only in current-process memory;
- records connection metadata, not packet or application content;
- classifies address scope without interpreting purpose or trustworthiness;
- associates an observation with a supported process only when the platform
  evidence permits reliable attribution;
- reports incomplete, permission-limited, missed, or unattributed evidence as
  such; and
- produces only allow-listed, sanitized report output.

No duration, sampling method, platform API, data model, interface, or report
schema is selected by this note.

## Evidence boundaries

Any future design must preserve these qualifications:

- An externally addressed connection does not establish that a prompt,
  response, model data, or other particular content was transmitted.
- No connection observed in one bounded window is not proof that communication
  did not occur outside the window or escape the observation method.
- A same-machine loopback endpoint does not independently establish where
  compute occurred.
- Address scope is not a privacy, safety, ownership, or trust judgement.
- Process attribution may be incomplete, permission-limited, stale, obscured by
  helper processes, or unavailable.
- Snapshot-based observation can miss short-lived connections. A future study
  must not silently turn a snapshot into a historical claim.

Privacy scores and categorical `private`, `safe`, or `no data left this
machine` guarantees are excluded unless a later evidence model can defend them.

## Privilege and privacy boundaries

The main application should remain unelevated. A first study should evaluate
only unprivileged, read-only platform facilities and treat partial visibility as
partial evidence rather than a reason to overstate coverage.

If useful observation requires elevated operating-system access, a later design
review must decide whether an optional, narrowly scoped helper is justified. It
would require its own threat model, minimal-capability definition, authenticated
local interface, explicit activation and termination behavior, sanitization
rules, and independent security review. This note does not authorize such a
helper.

The following remain outside this possible capability:

- packet-content or prompt inspection;
- TLS interception, certificate installation, or traffic decryption;
- blocking, firewall, routing, DNS, provider, or model changes;
- automatic reverse-DNS or reputation lookups;
- continuous or background monitoring, alerts, or persistent history;
- telemetry, upload, remote observation, or third-party reporting; and
- undisclosed or unbounded collection.

## Platform questions for a later feasibility study

The study should establish, rather than assume:

- which TCP and UDP states are visible without elevation on each verified
  platform;
- whether IPv4 and IPv6 have equivalent useful coverage;
- whether connection-to-process attribution is reliable for same-user,
  different-user, service, container, and helper-process cases;
- whether an event mechanism is available without broad privilege, or whether
  only snapshots are supportable;
- which short-lived connections and protocol cases can be missed;
- whether observing only supported provider process families is technically
  defensible without scanning or broad process disclosure;
- whether a bounded baseline comparison provides understandable evidence rather
  than misleading noise; and
- which fields can safely enter the existing allow-listed report projection.

Relevant platform starting points include Microsoft's owner-PID TCP and UDP
tables and Linux `NETLINK_SOCK_DIAG` plus permission-controlled `/proc/<pid>/fd`
association. These are feasibility leads, not selected implementations or
evidence of cross-platform parity.

## Possible gated sequence

1. **Evidence and privilege feasibility — complete:** the 1Z-A assessment
   documents platform APIs, visibility, permissions, blind spots, and stop
   conditions without collecting live evidence.
2. **Evidence-model and probe preparation — complete:** the 1Z-B
   [plan](network-observability-evidence-and-synthetic-probe-plan.md) defines
   fact, inference, unavailable states, sanitization, retention, fixtures,
   fixed bounds, and non-claims without collecting live evidence.
3. **Native implementation and bounded platform probes — next, not
   authorized:** only under a new exact contract, implement and test known
   synthetic local and controlled external connections on the exact verified
   baselines, with a separate one-run authorization for each platform.
4. **Adopt-or-defer decision:** proceed only if the result gives a
   non-specialist useful evidence without disproportionate privilege, security,
   maintenance, or support burden.
5. **Product contract:** only after adoption, define adapters, domain types,
   presentation, reporting, tests, native verification, and independent review.

Each step requires its own developer gate. Deferral at any step is an acceptable
outcome.

## Adoption test

Adopt a product slice only if it can answer at least one user question
accurately, expose its material blind spots in plain language, preserve the
main application's privilege and privacy posture, and behave coherently on both
verified platforms. Defer it if attribution is too incomplete, meaningful
observation requires broad privilege or intrusive inspection, or the interface
would predictably create false privacy assurance.
