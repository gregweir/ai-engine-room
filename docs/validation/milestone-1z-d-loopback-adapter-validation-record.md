# Milestone 1Z-D loopback adapter-validation record

## Status

**BOUNDED UBUNTU AND WINDOWS LOOPBACK VALIDATION PASSED; PRODUCT ADOPTION NOT
DECIDED.**

The developer separately authorized exactly one Ubuntu run and exactly one
Windows run. Both authorizations were consumed. Both sanitized outcomes
received independent evidence-only review and passed without a blocking
finding. No rerun is authorized by this record.

This record is the separately approved documentation change permitted by the
[1Z-D contract](../design/milestone-1z-d-loopback-adapter-validation-contract.md).
It records only reviewed source and binary identities, sanitized runtime
records, external termination evidence, cleanup facts, and non-claims. It does
not retain raw connection or process data.

## Shared reviewed source

| Field               | Value                                                                                  |
| ------------------- | -------------------------------------------------------------------------------------- |
| Repository          | `https://github.com/gregweir/ai-engine-room.git`                                       |
| Source commit       | `f99dcdbecc3d88f2e7981bbbc86733f58665aee7`                                             |
| Source tree         | `81d173649b51c8db912338cc5a69004ba7850aec`                                             |
| Merge parents       | `a2a1fa4a7011306f9d721f988a053b27ebbb84c4`, `4ecc8506ad4f64a21f2abbe08983ebd82703db18` |
| Deterministic CI    | Passed on the exact timing-correction tree before merge                                |
| Product integration | None                                                                                   |

Preflight and post-run checks on both platforms established the exact source,
tree, binary hash, ordinary-user context, clean repository state, exact public
origin, and zero local divergence from live `main`.

## Ubuntu result

| Field              | Value                                                              |
| ------------------ | ------------------------------------------------------------------ |
| Platform           | Ubuntu 24.04.4 LTS x86_64                                          |
| Privilege          | ordinary user, UID `1000`                                          |
| Toolchain          | Rust and Cargo 1.98.0, `x86_64-unknown-linux-gnu`                  |
| Binary SHA-256     | `c0baf37624ea889f6f0ce6e68b41e83b27306f044345fa0d5a46e43e7f9cc73d` |
| Binary size        | 779,800 bytes                                                      |
| Invocation count   | exactly one supervisor invocation                                  |
| Process exit       | `0`; external timeout did not fire                                 |
| Captured output    | one 574-byte sanitized line; empty stderr                          |
| Cleanup            | private temporary captures removed                                 |
| Independent review | PASS; no blocking finding                                          |

Sanitized record:

```text
record|platform=ubuntu_24_04_x86_64|planned_samples=11|actual_samples=11|planned_interval_ms=500|max_actual_interval_ms=500|planned_window_ms=5000|actual_window_ms=5000|outer_timeout_s=15|acquisition=complete|expected_missed=0|result=loopback_ipv4_long,ipv4,established,same_machine_loopback,supported_process,observed,11111111111,22|result=loopback_ipv6_long,ipv6,established,same_machine_loopback,supported_process,observed,11111111111,22|fixture_termination=clean|sampler_termination=clean|supervisor_termination=pending_exit|non_claims=bounded_loopback_observation_only
```

The shell observed the supervisor exit after this record was emitted. The
independent review therefore completed the supervisor termination state for
this attempt without altering the retained sanitized line.

## Windows result

| Field              | Value                                                              |
| ------------------ | ------------------------------------------------------------------ |
| Platform           | Windows 11 Core build 26200.7462 x64                               |
| Privilege          | Medium integrity, `S-1-16-8192`                                    |
| Toolchain          | Rust and Cargo 1.98.0, `x86_64-pc-windows-msvc`                    |
| Binary SHA-256     | `308df0da61ee16fd033a84856ef28c82218d859365a7e367a37cd97cf07d0d73` |
| Binary size        | 428,032 bytes                                                      |
| Invocation count   | exactly one supervisor invocation                                  |
| Process exit       | `0`; external timeout did not fire                                 |
| Captured output    | one 580-byte sanitized line; empty stderr                          |
| Cleanup            | capture remained in memory; interactive capture values cleared     |
| Independent review | PASS; no blocking finding                                          |

The Windows SSH session had High integrity and was rejected as a run context.
Its build was not executed. The retained Windows result instead uses the
independently built and hashed Medium-integrity candidate above.

Sanitized record:

```text
record|platform=windows_11_25h2_26200_x64|planned_samples=11|actual_samples=11|planned_interval_ms=500|max_actual_interval_ms=500|planned_window_ms=5000|actual_window_ms=5000|outer_timeout_s=15|acquisition=complete|expected_missed=0|result=loopback_ipv4_long,ipv4,established,same_machine_loopback,supported_process,observed,11111111111,22|result=loopback_ipv6_long,ipv6,established,same_machine_loopback,supported_process,observed,11111111111,22|fixture_termination=clean|sampler_termination=clean|supervisor_termination=pending_exit|non_claims=bounded_loopback_observation_only
```

The Medium-integrity PowerShell session observed the supervisor exit after this
record was emitted. The independent review therefore completed the supervisor
termination state for this attempt without altering the retained sanitized
line.

## Bounded conclusion

Both platform outcomes meet the 1Z-D **loopback validation pass** class:

- all 11 scheduled samples completed at a maximum 500-millisecond interval and
  an exact 5,000-millisecond first-to-last window;
- the attributed IPv4 and IPv6 long-lived loopback fixtures appeared in every
  sample with `same_machine_loopback` classification;
- acquisition was complete and no expected fixture was missed;
- fixture and sampler termination were clean, and supervisor exit was
  externally confirmed;
- the closed-grammar and exclusion scans passed; and
- source, binary, repository, and privilege identities remained stable.

This is useful evidence that the isolated adapters can observe the two
allow-listed loopback fixture classes within the reviewed bounds on the two
tested platform baselines. It is not product evidence and does not establish
the behavior of arbitrary AI applications.

## Required non-claims

These outcomes do **not** establish:

- externally addressed, LAN, UDP, unattributed, permission-limited, stale, or
  short-lived connection visibility;
- complete connection history or absence of unobserved communication;
- packet, prompt, response, model, authentication, telemetry, or other
  application content;
- destination ownership, purpose, safety, trustworthiness, or whether bytes
  crossed a physical interface;
- compute location, provider privacy, product usefulness, release readiness,
  or a privacy guarantee;
- behavior on another Ubuntu release, Windows build, architecture, privilege
  context, container, namespace, or user account; or
- authority to integrate the probe into the application, UI, report,
  packaging, installer, service, or release.

## Disposition and next gate

Milestone 1Z-D execution is complete. Both one-run authorizations are consumed,
and this record grants no rerun.

Product adoption remains undecided because the tested slice covers only
long-lived attributed loopback fixtures. The recommended next step is a
separately approved documentation-only disposition slice that decides whether
to stop or defer, prepare a controlled externally addressed validation
contract, or define another smallest useful evidence gap. No later fixture,
product integration, or platform execution is authorized here.
