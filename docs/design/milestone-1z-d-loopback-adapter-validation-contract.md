# Milestone 1Z-D loopback adapter-validation contract

## Status

Contract preparation merged on 2026-09-01. Isolated implementation preparation
is authorized from exact merged baseline
`5d3b8744350904f41e7faa468ca071fa80b52eb7`. This authorization covers one
local implementation commit within the path allow-list below and
artificial-only deterministic verification. It does not authorize a push, a
real adapter call, a fixture or executable run, a socket or process-table
observation, or either platform run.

The developer must separately approve the exact implementation commit before
push. After deterministic CI passes, one Ubuntu run and one Windows run each
require their own explicit authorization. Approval of either run does not
authorize the other.

## Decision and purpose

The first real-adapter validation is narrowed from the full
[1Z-B fixture matrix](network-observability-evidence-and-synthetic-probe-plan.md)
to exactly two attributed, long-lived loopback TCP fixtures:

- `loopback_ipv4_long`; and
- `loopback_ipv6_long`.

This is the smallest useful test of IPv4/IPv6 snapshot acquisition, ordinary-
user process association, bounded scheduling, structural sanitization, and
confirmed teardown. It creates no Internet connection and makes no claim
about externally addressed connections.

The externally addressed, unattributed, and short-lived fixtures remain
deferred. Their result types stay in the closed model, but this milestone
cannot validate their real-platform behavior or support an adopt-or-defer
product decision.

## Reviewed baseline

| Field               | Required value                                           |
| ------------------- | -------------------------------------------------------- |
| Repository          | `gregweir/ai-engine-room`                                |
| Contract parent     | `a236b76ffbb18e8f5cce8e629861dc241bbf12b6`               |
| Contract branch     | `codex/network-observability-bounded-execution-contract` |
| Implementation base | `5d3b8744350904f41e7faa468ca071fa80b52eb7`               |
| Ubuntu class        | Ubuntu 24.04 LTS x86_64, ordinary user                   |
| Windows class       | Windows 11 25H2 build 26200.7462 x64, ordinary user      |
| Product integration | None                                                     |

At each later gate, HEAD, tree, remotes, live `origin/main`, changed paths,
tool versions, target triple, and working-tree state must be reported. A
different platform, source identity, dependency graph, privilege context, or
binary hash is an infrastructure stop, not permission to improvise.

## Contract-preparation paths

This contract-only change may modify only:

- this contract;
- `docs/design/network-observability-evidence-and-synthetic-probe-plan.md`;
- `docs/design/bounded-network-observability-planning-note.md`;
- `docs/roadmap.md`; and
- `scripts/repository-readiness.mjs`.

No Rust source, manifest, lockfile, workflow, fixture, executable, product,
package, release, or evidence-record change is authorized in this slice.

## Later implementation boundary

A separately approved implementation may add one isolated binary target named
`aer-network-probe-loopback` inside `crates/aer-network-probe`. It must remain
unreferenced by `src-tauri`, `aer-core`, the frontend, package configuration,
and release workflows. It is a developer-operated validation instrument, not
an application feature or shipped utility.

The later implementation path allow-list is limited to:

- `crates/aer-network-probe/Cargo.toml`;
- the existing `crates/aer-network-probe/src/adapter.rs`, `model.rs`,
  `sanitize.rs`, and target-gated platform modules only where required to pass
  closed internal rows into sanitization;
- `crates/aer-network-probe/src/loopback_main.rs`;
- `crates/aer-network-probe/src/loopback/mod.rs`;
- `crates/aer-network-probe/src/loopback/fixture.rs`;
- `crates/aer-network-probe/src/loopback/sampler.rs`;
- `crates/aer-network-probe/src/loopback/supervisor.rs`;
- `crates/aer-network-probe/src/loopback/record.rs`;
- artificial-only unit tests colocated in those files;
- `scripts/repository-readiness.mjs`; and
- this contract and `docs/roadmap.md` for exact status.

No new dependency, build script, integration test, workflow, Tauri command,
capability, frontend path, package file, service, daemon, installer behavior,
or release asset may be added. The root lockfile must remain byte-for-byte
unchanged.

## Process and fixture topology

The executable must use explicit internal roles and fail closed on every
unknown argument:

1. A supervisor process starts exactly two direct fixture children and one
   direct sampler child.
2. Each fixture child owns exactly one TCP listener and one connected TCP
   client on its assigned loopback family. Distinct fixture processes provide
   distinct allow-listed process identities.
3. Listener port `0` selects an ephemeral local port. The numeric port remains
   inside its fixture process and must never cross a pipe or enter retained
   output, logs, annotations, or command-line arguments. Readiness uses one
   fixed token only.
4. Fixture children send no application payload. After readiness they hold
   the connection until the supervisor requests shutdown.
5. The sampler receives fixture PID plus the minimum platform identity through
   a captured pipe, not through user-visible arguments. It calls the selected
   native adapter only during an explicitly authorized platform run.
6. The supervisor owns every child handle, applies the outer timeout, captures
   all child output, and kills and reaps every remaining child on any failure.

The fixture listener rows must be excluded. Rows not associated with either
allow-listed fixture process must be discarded immediately and must not affect
the retained result. The fixture processes must perform no file, DNS, HTTP,
provider, telemetry, update, authentication, model, or inference access.

## Fixed schedule and bounds

The later run must preserve these 1Z-B bounds:

- exactly 11 scheduled snapshots, indexes `0` through `10`;
- one scheduled snapshot every 500 milliseconds using monotonic time;
- no more than five seconds from the first through last snapshot;
- both fixture connections ready before sample `0` and held through sample
  `10`;
- no retry, replacement sample, adaptive interval, second pass, or post-sample
  observation; and
- a 15-second fail-closed outer timeout covering setup, acquisition,
  projection, sanitization, fixture shutdown, sampler shutdown, and reaping.

A late sample keeps its assigned index. If the next sample or outer bound
would be exceeded, the attempt stops. No timeout or partial result permits a
retry under the same authorization.

## Privilege and platform boundary

Both runs must be unelevated. The preflight must stop if the user is root, an
administrator token is elevated, or the adapter requests capabilities, debug
privilege, a helper, service, driver, firewall rule, namespace change, packet
capture, event trace, or broader process access.

Windows is limited to owner-PID IPv4/IPv6 TCP tables and minimum image-identity
validation for the two allow-listed fixture PIDs. Linux is limited to
IPv4/IPv6 `NETLINK_SOCK_DIAG` snapshots and `/proc/<fixture-pid>/fd` descriptor
links for those two same-user PIDs. No command line, environment, or path for
an unrelated process may be inspected.

## Raw-data and retained-record boundary

Raw addresses, ports, PIDs, UIDs, socket inodes, image paths, descriptor
paths, usernames, hostnames, command lines, wall-clock times, and unrelated
rows may exist only in current-process memory while needed. They must never be
written to a file, terminal, log, workflow annotation, clipboard, crash
attachment, telemetry channel, or application state.

The sampler may emit a candidate record only through a captured pipe to the
supervisor. Before anything is shown or retained, the supervisor must parse a
closed grammar and reject arbitrary text. The final sanitized runtime record may
contain only:

- approved platform class without machine name;
- planned and actual sample counts, interval, window, and outer timeout;
- the two allow-listed fixture labels;
- address family, `same_machine_loopback`, normalized TCP state, observed
  sample indexes, and deduplicated count;
- association, acquisition, fixture outcome, and expected-but-missed count;
- fixture, sampler, and supervisor termination results; and
- the fixed non-claims from 1Z-B.

The reviewed source commit and probe-binary SHA-256 are established by the
preflight before execution and are not passed into the executable. Independent
review may combine those two identities with the sanitized runtime record only
after successful termination and exclusion scanning.

The supervisor cannot truthfully attest to its own completed termination before
it exits. Its sanitized line must therefore use the fixed
`supervisor_termination=pending_exit` token. Independent review may replace that
pending state with the externally observed exit result only after the process
has ended and all capture and exclusion checks succeed.

The exclusion scan must reject path separators, IP-address and socket-address
syntax, decimal port/PID-style fields, usernames, hostnames, unapproved labels,
free text, or any field outside the grammar. Failure produces only a fixed
`sanitization_stop` token and destroys the candidate in memory.

No run output may be committed automatically. Any later sanitized evidence
record requires independent review and a separately approved documentation
change.

## Deterministic implementation verification

Before any real run can be proposed, ordinary CI for the exact implementation
commit must pass on Ubuntu and Windows:

- repository and licence guardrails;
- `cargo fmt --all -- --check`;
- artificial-only `cargo test --workspace` with `RUST_TEST_THREADS=1`;
- `cargo clippy --workspace --all-targets -- -D warnings`; and
- existing package-licence builds proving the product package remains
  unaffected.

Tests may exercise argument rejection, schedule arithmetic, record grammar,
sanitization, timeout state transitions, and fake child outcomes. They must not
open a socket, call either real adapter, read `/proc`, call IP Helper or process
APIs, spawn a process, use a real timer, or inspect host state.

Passing CI authorizes no real execution.

## Separately consumed platform authorizations

After the implementation commit and its CI run are reported, the sequence is:

1. The developer may authorize exactly one Ubuntu run using the exact source
   commit and binary hash.
2. That run must finish, sanitize, terminate, and receive independent review
   before a Windows run is proposed.
3. The developer may then authorize exactly one Windows run using the exact
   reviewed source commit and independently built binary hash.
4. A stop, failure, partial result, environment mismatch, or missed fixture
   consumes that platform authorization. No automatic rerun is allowed.

The commands, source tree, binary hash, toolchain, platform version, privilege
preflight, temporary output location, and cleanup procedure must be reported
immediately before each authorization. Approval to prepare or push code is not
approval to execute it.

## Result classes and non-claims

- **Loopback validation pass:** both long fixtures are observed in both address
  families with loopback classification and supported-process association;
  bounds, sanitization, and teardown pass.
- **Partial evidence:** acquisition occurs but a fixture is missed,
  permission-limited, stale, or unattributed.
- **Attributed stop:** a source, privilege, bound, privacy, sanitization, or
  termination rule fails.
- **Infrastructure stop:** the exact baseline, toolchain, source, or binary
  identity cannot be established.

No outcome establishes externally addressed visibility, complete connection
history, packet or application content, absence of communication, compute
location, privacy, safety, trust, product usefulness, or release readiness.

## Explicit exclusions

This contract does not authorize:

- any external address, Internet connection, DNS name, provider endpoint,
  public website, shared service, or fallback destination;
- UDP, packet capture, payload inspection, TLS interception, firewall or route
  changes, continuous observation, or background monitoring;
- elevated permission, another user's processes, container or namespace
  inspection, helper processes, or unrelated-process enumeration;
- product integration, UI, report, packaging, signing, publication, release,
  or adoption; or
- either real adapter call until the exact platform run is separately
  authorized.

## Contract acceptance

The implementation change is ready for review only if its exact branch stays
within the later implementation allow-list, the root lockfile and product
isolation remain unchanged, artificial-only deterministic checks pass, and no
probe, fixture, adapter, socket, process table, endpoint, product, package, or
platform execution occurred. Its exact local commit must be reported for
separate push approval.

## References

- [Milestone 1Z-B evidence and synthetic-probe plan](network-observability-evidence-and-synthetic-probe-plan.md)
- [Milestone 1Z-C implementation contract](milestone-1z-c-native-probe-implementation-contract.md)
- [Platform feasibility assessment](../research/network-observability-platform-feasibility.md)
- [Bounded network-observability planning note](bounded-network-observability-planning-note.md)
