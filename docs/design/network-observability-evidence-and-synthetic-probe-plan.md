# Network-observability evidence and synthetic-probe plan

Status: **BOUNDED LOOPBACK VALIDATION COMPLETE; PRODUCT ADOPTION DEFERRED** on
2026-09-01.

## Decision

The evidence model and smallest coherent cross-platform loopback probe were
implemented and exercised under separately consumed one-run authorizations.
The reviewed Ubuntu and Windows outcomes are recorded in the
[1Z-D validation record](../validation/milestone-1z-d-loopback-adapter-validation-record.md).
No product feature is adopted.

The isolated non-product adapter library is merged under the
[1Z-C contract](milestone-1z-c-native-probe-implementation-contract.md), and
the loopback-only runner and platform executions completed under the
[1Z-D contract](milestone-1z-d-loopback-adapter-validation-contract.md). Both
one-run authorizations are consumed; no rerun or broader fixture is authorized.
The 1Z-E
[disposition](milestone-1z-e-network-observability-disposition.md) defers
product adoption and additional platform validation.

## Milestone 1Z-D narrowing decision

The first proposed real-adapter validation is narrowed to the long-lived
`loopback_ipv4_long` and `loopback_ipv6_long` fixtures only. It preserves the
11-snapshot, five-second-window, 15-second-timeout, sanitization, privilege,
termination, and separate-platform-authorization bounds below while creating
no external connection.

Externally addressed, unattributed, and short-lived fixtures remain deferred.
This loopback-only slice cannot validate those behaviors and cannot support a
product-adoption decision. Any later external fixture still requires a new
contract that pins numeric endpoints, control, payload, logging, retention,
and deletion terms with no fallback.

## Questions the probe may answer

For controlled synthetic TCP fixtures only, the proposed probe may establish:

- whether a fixture's connection row appeared in one or more scheduled
  snapshots;
- whether its numeric remote address was deterministically classified as
  same-machine loopback or externally addressed;
- whether the row could be associated with the known fixture process under the
  current user's ordinary permissions; and
- which expected fixtures were missed, unavailable, permission-limited, or
  stale.

It cannot establish what application content crossed a connection, whether a
physical interface carried traffic, where computation occurred, why a
destination was contacted, or what happened between or outside snapshots.

## Evidence model

### Ephemeral raw observation

Each platform adapter may hold one raw row in current-process memory with only
these fields:

| Field             | Meaning                                                   |
| ----------------- | --------------------------------------------------------- |
| `sample_index`    | Integer from `0` through `10`                             |
| `protocol`        | Fixed value `tcp`                                         |
| `address_family`  | `ipv4` or `ipv6`                                          |
| `tcp_state`       | Platform-reported state normalized without interpretation |
| `local_endpoint`  | Numeric address and port                                  |
| `remote_endpoint` | Numeric address and port                                  |
| `platform_owner`  | Windows PID, or Linux UID and socket inode                |
| `acquisition`     | `observed`, `row_changed`, or `adapter_error`             |

Raw rows are not report data. They must never be written to a file, log,
standard output, workflow annotation, clipboard, crash attachment, telemetry,
or application state. Rows unrelated to the allow-listed fixture identities
must be discarded immediately after in-memory matching.

### Derived address scope

The remote numeric address is projected to exactly one value:

- `same_machine_loopback` — IPv4 `127.0.0.0/8` or IPv6 `::1/128`;
- `local_or_private_scope` — a current IANA special-purpose range selected by
  the later implementation's reviewed, versioned table but not loopback;
- `externally_addressed` — not present in that reviewed special-purpose table;
- `special_or_unresolved` — special-purpose semantics are unsuitable for the
  other classes or parsing/classification failed.

Classification describes only an address. `local_or_private_scope` does not
mean same machine or private in the ordinary-language sense.
`externally_addressed` does not establish routing, ownership, trust, purpose,
or transmitted content.

### Process association

Each expected fixture observation is projected to exactly one association:

- `supported_process` — the row is tied to the exact known fixture process and
  the process identity is still current at validation time;
- `helper_descendant` — reserved for a later reviewed parent/child identity
  rule and not accepted by this first probe;
- `unattributed` — ownership evidence exists but cannot be tied to the fixture;
- `permission_limited` — the required process or descriptor identity query was
  denied under ordinary permissions;
- `stale` — the process exited, its identity changed, or PID/inode evidence
  could not be revalidated before projection.

On Windows, PID plus a separately queried executable identity must match the
known fixture process. On Linux, socket inode matching may inspect descriptor
links only for the known same-user fixture PIDs; it must not enumerate or
retain command lines or paths for unrelated processes. PID, UID, inode, and
executable path remain ephemeral even when association succeeds.

### Acquisition and fixture outcome

The probe records adapter acquisition separately from fixture outcome:

- acquisition: `complete`, `partial`, `permission_limited`, or `failed`;
- fixture outcome: `observed`, `expected_but_missed`, `fixture_unavailable`,
  `association_unavailable`, or `invalidated`.

An adapter failure does not become a missed fixture. A missed fixture does not
become evidence that its connection never existed. A permission-limited
association does not invalidate an otherwise observed endpoint class.

## Sanitized retained record

After termination, the only retainable per-platform record is:

| Field            | Allowed value                                                                             |
| ---------------- | ----------------------------------------------------------------------------------------- |
| Source identity  | Reviewed commit and probe binary SHA-256                                                  |
| Platform class   | Approved public baseline description, without machine name                                |
| Bounds           | Planned and actual sample count, interval, window, and outer timeout                      |
| Fixture identity | Allow-listed synthetic label only                                                         |
| Observation      | Address family, derived scope, TCP state, observed sample indexes, and deduplicated count |
| Attribution      | Association outcome only                                                                  |
| Completeness     | Acquisition and fixture outcome plus expected-but-missed count                            |
| Termination      | Fixture exit, probe exit, and no-observer-remains result                                  |
| Interpretation   | Fixed limitations and result class                                                        |

The record must exclude raw addresses and ports, PIDs, UIDs, inodes, executable
paths, usernames, hostnames, command lines, unrelated rows, wall-clock times,
and physical repository paths. Before retention, the procedure must scan the
candidate record and fail closed if any excluded field or non-allow-listed
text is present.

## Controlled fixture matrix

The first proposed probe has six long-lived fixtures and two deliberately
short-lived fixtures:

| Label                        | Family | Remote class          | Lifetime                                           | Purpose                                         |
| ---------------------------- | ------ | --------------------- | -------------------------------------------------- | ----------------------------------------------- |
| `loopback_ipv4_long`         | IPv4   | same-machine loopback | Entire five-second window                          | Endpoint visibility and fixture attribution     |
| `loopback_ipv6_long`         | IPv6   | same-machine loopback | Entire five-second window                          | Endpoint visibility and fixture attribution     |
| `external_ipv4_long`         | IPv4   | externally addressed  | Entire five-second window                          | External-address classification and attribution |
| `external_ipv6_long`         | IPv6   | externally addressed  | Entire five-second window                          | External-address classification and attribution |
| `loopback_ipv4_unattributed` | IPv4   | same-machine loopback | Entire five-second window                          | Explicit unmatched-row handling                 |
| `loopback_ipv6_unattributed` | IPv6   | same-machine loopback | Entire five-second window                          | Explicit unmatched-row handling                 |
| `loopback_ipv4_short`        | IPv4   | same-machine loopback | At most 100 milliseconds between scheduled samples | Demonstrate snapshot miss risk                  |
| `loopback_ipv6_short`        | IPv6   | same-machine loopback | At most 100 milliseconds between scheduled samples | Demonstrate snapshot miss risk                  |

The long loopback fixtures use listeners owned by the reviewed fixture
program. The unattributed fixtures use a separately started reviewed companion
whose PID is deliberately absent from the allow-list; the sampler must discard
its raw identity while retaining only the `unattributed` outcome.

External fixtures may send no application payload. Their acceptor must hold
the TCP connection open through the last sample and then close it. A later
execution contract must pin literal numeric IPv4 and IPv6 addresses, ports,
endpoint ownership/control, server behavior, retention, and deletion terms.
If either controlled endpoint is unavailable or its identity is not exact, the
corresponding fixture is `fixture_unavailable` and there is no fallback.

No DNS name, provider endpoint, model service, public website, telemetry
collector, or unrelated existing connection may serve as a fixture.

## Sampling and lifetime bounds

The proposed schedule is fixed:

- exactly 11 scheduled snapshots, indexes `0` through `10`;
- one snapshot every 500 milliseconds;
- no more than five seconds from the first to the last snapshot;
- long-lived fixtures established before sample `0` and held until sample `10`
  completes;
- short-lived loopback fixtures started only after a completed scheduled
  sample and closed within 100 milliseconds;
- no retry, adaptive sampling, polling after sample `10`, or second pass; and
- a 15-second fail-closed outer timeout covering setup, sampling, projection,
  fixture shutdown, and probe shutdown.

Monotonic time controls the bounds. Wall-clock timestamps are neither required
nor retained. A late snapshot remains at its assigned index; the sampler must
not add a replacement sample.

## Platform adapter boundaries

The Windows candidate uses owner-PID IPv4 and IPv6 TCP tables and only the
minimum process query needed to validate known fixture PIDs. The Linux
candidate uses `NETLINK_SOCK_DIAG` IPv4 and IPv6 TCP dumps and inspects
`/proc/<fixture-pid>/fd` links only for allow-listed same-user fixture PIDs.

Both adapters must run unelevated. They may not request administrator or root
access, capabilities, debug privilege, another network namespace, a service,
driver, helper daemon, event trace, packet capture, firewall exception, route
change, or broader process enumeration. Listeners are excluded from retained
connection observations.

## Fail-closed preconditions and stops

The later procedure must stop without sampling if source identity, platform
baseline, ordinary-user context, binary hashes, fixture labels, numeric
endpoint identity, or sanitization policy does not match its exact execution
contract.

Once started, it must stop and classify the attempt if:

- either adapter requests or appears to require elevation;
- raw evidence is written or emitted outside probe memory;
- an unrelated process path or command line is inspected or retained;
- the sample count, interval, five-second window, or 15-second outer timeout
  would be exceeded;
- a fixture contacts an unapproved address or sends application payload;
- fixture or probe termination cannot be confirmed; or
- the candidate retained record fails the exclusion scan.

No stop permits a retry, broader permission, substitute endpoint, longer
window, additional sample, or improvised evidence capture.

## Result classes

- **Probe pass:** both address families acquire snapshots within bounds; all
  four long attributed fixtures are observed with the expected address class
  and process association; unmatched and missed evidence are represented
  without overclaim; sanitization and termination pass.
- **Partial evidence:** snapshots are acquired, but at least one expected
  fixture is unavailable, missed, unattributed, permission-limited, or stale.
- **Attributed stop:** a reviewed precondition, bound, privacy rule,
  sanitization rule, or termination assertion fails.
- **Infrastructure stop:** the exact platform, source, controlled endpoint, or
  disposable execution context cannot be established.

Short-lived fixtures may be observed or missed; either outcome is acceptable
only when accurately recorded. Their purpose is to demonstrate the snapshot
boundary, not to create a pass criterion.

No result establishes complete network history, packet contents, provider
privacy, compute location, product usefulness, or release readiness.

## Review and adopt-or-defer threshold

The completed 1Z-D evidence satisfies the loopback-only threshold for the two
long attributed fixtures on the reviewed Ubuntu and Windows baselines. Both
platforms produced understandable sanitized evidence without elevation or
retained unrelated details. This result does not satisfy the broader matrix or
decide product adoption because external, unattributed, permission-limited,
stale, and short-lived behavior remains unvalidated.

After separately authorized runs, independent review must verify source and
run identity, bounds, sanitization, termination, platform parity, and every
fixed non-claim. Product adoption may be proposed only if both platforms
produce understandable evidence for all long attributed fixtures without
elevation and without retaining unrelated details.

Defer the feature if either platform requires broader privilege, Linux process
association is predictably too incomplete, sanitized output cannot explain
material gaps, externally addressed fixtures cannot be controlled safely, or
the evidence would encourage a non-specialist to infer privacy guarantees.

The 1Z-E disposition applies that threshold conservatively. Although the two
loopback fixtures passed, the evidence does not cover the externally addressed
behavior needed for the main user question, and the cost and ambiguity of the
next evidence step are not justified by demonstrated demand. The current
sequence therefore stops without product adoption or another platform run.

## References

- [Milestone 1Z-B preparation contract](milestone-1z-b-network-observability-probe-preparation-contract.md)
- [Platform feasibility assessment](../research/network-observability-platform-feasibility.md)
- [Bounded network-observability planning note](bounded-network-observability-planning-note.md)
- [IANA IPv4 special-purpose registry](https://www.iana.org/assignments/iana-ipv4-special-registry/)
- [IANA IPv6 special-purpose registry](https://www.iana.org/assignments/iana-ipv6-special-registry/)
