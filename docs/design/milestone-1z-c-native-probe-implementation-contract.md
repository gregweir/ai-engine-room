# Milestone 1Z-C native-probe implementation contract

## Status

Contract merged on 2026-09-01. The developer separately authorized preparation
of one local isolated, non-product Rust library change from merge baseline
`2d8078a655350b22ffe9fdc45db7616581565966`. The implementation remains local
until its exact commit, parent, changed paths, lock diff, and test commands are
reported and separately approved for push. No real adapter call, probe
executable, socket or process-table access, connection, fixture execution,
endpoint contact, product integration, or platform run is authorized.

## Purpose

The [Milestone 1Z-B plan](network-observability-evidence-and-synthetic-probe-plan.md)
defines evidence semantics, controlled fixtures, sampling bounds, sanitization,
and stop conditions. Before any fixture or live evidence is possible, the
native acquisition code needs an independently reviewable boundary that cannot
be reached by the shipped application.

This contract fixes that boundary. A later implementation change may compile
platform adapters and prove pure transformations with artificial rows, but it
may not provide or invoke an executable path.

## Reviewed baseline

| Field                            | Required value                                            |
| -------------------------------- | --------------------------------------------------------- |
| Repository                       | `gregweir/ai-engine-room`                                 |
| Contract baseline commit         | `42c605c3905eb0e314e7a73a032f36cad5a0cf4a`                |
| Authorized implementation parent | `2d8078a655350b22ffe9fdc45db7616581565966`                |
| Proposed implementation branch   | `codex/network-observability-native-probe-implementation` |
| New crate                        | `crates/aer-network-probe`                                |
| Product application crate        | `src-tauri` — must not depend on the new crate            |

Before the later implementation branch is pushed, its exact commit, parent,
changed paths, dependency-lock diff, and test commands must be reported for
developer approval. Approval to merge this contract is not approval of that
implementation commit.

## Allowed implementation paths

The later implementation change may modify only:

- root `Cargo.toml` and `Cargo.lock`;
- `crates/aer-network-probe/Cargo.toml`;
- `crates/aer-network-probe/src/lib.rs`;
- `crates/aer-network-probe/src/model.rs`;
- `crates/aer-network-probe/src/classify.rs`;
- `crates/aer-network-probe/src/sanitize.rs`;
- `crates/aer-network-probe/src/adapter.rs`;
- `crates/aer-network-probe/src/platform/mod.rs`;
- `crates/aer-network-probe/src/platform/linux.rs`;
- `crates/aer-network-probe/src/platform/windows.rs`;
- generated third-party licence and source materials required by the normal
  repository licence process;
- `scripts/repository-readiness.mjs`; and
- this contract and `docs/roadmap.md` for exact implementation status.

No binary target, example, benchmark, build script, integration test, live
test, workflow, fixture process, server, Tauri command, frontend path,
capability, report field, package configuration, or release file may be added.

## Isolation boundary

`aer-network-probe` is a workspace library used only for compilation and pure
unit tests in this slice. Neither `src-tauri` nor `aer-core` may depend on it.
The shipped application therefore has no call path to acquisition code and no
new interface or report behavior.

The crate must expose no convenience function that starts a timed sampler,
spawns a process, opens a listener, creates a test connection, resolves a name,
serializes raw rows, writes a record, or changes the host. Platform adapters
may expose one explicit snapshot operation for later orchestration, but nothing
in this slice may call the real operation.

## Required modules

### `model`

Define closed enums and bounded structures for the 1Z-B evidence vocabulary:
TCP only; IPv4/IPv6; normalized TCP state; sample index `0..=10`; acquisition
outcomes; address classes; process-association outcomes; fixture outcomes; and
sanitized per-fixture results.

Raw rows must keep numeric endpoints and platform ownership evidence in
internal-only types. Retained types must be structurally unable to contain raw
addresses, ports, PIDs, UIDs, socket inodes, executable paths, usernames,
hostnames, command lines, wall-clock times, or arbitrary strings.

### `classify`

Implement pure numeric-address classification separately from acquisition. The
implementation change must identify the exact IANA IPv4 and IPv6
special-purpose registry retrieval date and encode a reviewed, versioned table
with source comments and tests at every included range boundary. Loopback must
be classified before other special-purpose ranges.

An address outside the reviewed special-purpose table becomes
`externally_addressed`; this is an address classification only. Parsing failure
and ranges whose semantics do not fit another class become
`special_or_unresolved`. No DNS, route, interface, ownership, reputation, or
reachability query is permitted.

### `sanitize`

Project artificial raw rows and explicit fixture metadata into the closed
retained type. Deduplicate only by fixture label, address family, normalized
state, derived scope, association outcome, and sample index. Reject sample
indexes outside `0..=10`, unknown fixture labels, listeners, UDP, arbitrary
text, and inconsistent expected classifications.

Tests must demonstrate that raw endpoint and owner fields have no route into a
retained result. This is a structural projection test, not a regex-based claim
that secret text was removed after serialization.

### `adapter`

Define the smallest platform-neutral snapshot trait and controlled error
vocabulary. One call returns one bounded in-memory collection for a supplied
allow-list of known fixture process identities. The trait must not include
timing, retries, persistence, logging, DNS, process spawning, endpoint
selection, privilege escalation, or unrelated-process discovery.

### `platform`

Compile target-gated adapters without invoking them:

- Windows: owner-PID IPv4 and IPv6 TCP tables plus the minimum image-identity
  query for allow-listed fixture PIDs;
- Linux: IPv4 and IPv6 TCP `NETLINK_SOCK_DIAG` dumps plus descriptor-link
  matching limited to allow-listed same-user fixture PIDs.

The Windows adapter may use only `windows-sys` 0.61.2 with the minimum reviewed
IP Helper, Winsock, process, and foundation feature set. The Linux adapter may
use only the locked `libc` line needed for documented socket and Netlink
bindings. No asynchronous runtime, packet library, process-inspection library,
network client, DNS library, serialization library, elevated helper, or build
dependency may be added.

Every unsafe block must state pointer, buffer-size, initialization, ownership,
and lifetime invariants immediately beside the call. Platform-returned buffer
lengths, row counts, Netlink message lengths, alignment, sequence values, PIDs,
UIDs, and inodes must be checked before use. Malformed or changing tables fail
to controlled partial or failed evidence; they must not panic or retry.

## Test boundary

All tests in the implementation change must use artificial values supplied
directly to pure functions or fake adapter seams. They may test:

- every enum and validation boundary;
- every IANA range edge and representative ordinary address;
- row projection, deduplication, ordering, listener exclusion, and rejection;
- complete, partial, permission-limited, stale, missed, and invalidated states;
- buffer and message parsers with byte fixtures held in test memory; and
- structural absence of raw fields from retained types.

Tests must not open any socket, read `/proc`, call IP Helper or process APIs,
enumerate processes or descriptors, start a thread that samples, use a timer,
contact an endpoint, inspect the host, or depend on machine state. Ordinary
workspace tests must remain deterministic with `RUST_TEST_THREADS=1`.

## Dependency and product invariants

The later change must leave these facts mechanically asserted:

- `src-tauri/Cargo.toml` and `crates/aer-core/Cargo.toml` do not name
  `aer-network-probe`;
- no Tauri command or capability names network observation;
- no binary, example, integration test, live test, workflow, or fixture exists
  for the new crate;
- the crate has no default feature that performs acquisition;
- its dependency set is limited to the reviewed target bindings above; and
- generated licence materials are current and package payloads remain
  unchanged except for normal regenerated notices if required.

## Later execution remains separately gated

Even a passing implementation change would not authorize calling a real
adapter. A later Milestone 1Z-D contract must separately define and review:

- the probe and fixture executable source;
- the monotonic 11-snapshot scheduler and 15-second outer timeout;
- exact controlled numeric IPv4 and IPv6 external endpoints and their
  ownership, payload, logging, retention, and deletion terms;
- complete commands, exact build identity, unelevated checks, sanitization,
  teardown, and no-observer-remains assertions; and
- one explicit run authorization for Ubuntu and a separate explicit run
  authorization for Windows.

There is no fallback to a provider, model service, public website, DNS name,
shared CDN address, opportunistic endpoint, or unrelated existing connection.
If controlled external endpoints cannot be established without disproportionate
cost or privacy burden, deferral is the required outcome.

## Explicit exclusions

This contract and its proposed implementation do not authorize:

- any socket, process, descriptor, route, interface, namespace, DNS, firewall,
  packet, payload, provider, model, or inference observation;
- a probe or fixture executable, listener, connection, timer, background task,
  service, driver, helper, elevated permission, or physical-machine action;
- application integration, UI, report, clipboard, telemetry, persistence,
  packaging behavior, signing, publication, or release; or
- a product-adoption, privacy, safety, locality, routing, content, or
  completeness claim.

## Contract acceptance

This contract is ready for merge review only if the branch contains
documentation and deterministic-guardrail changes alone, the implementation
remains unapproved, no native source or dependency changes occur, and ordinary
deterministic checks pass.

## References

- [Milestone 1Z-B preparation contract](milestone-1z-b-network-observability-probe-preparation-contract.md)
- [Evidence and synthetic-probe plan](network-observability-evidence-and-synthetic-probe-plan.md)
- [Platform feasibility assessment](../research/network-observability-platform-feasibility.md)
- [Architecture](architecture.md)
