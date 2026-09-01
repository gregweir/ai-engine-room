# Milestone 1Z-A network-observability feasibility contract

## Status

Documentation-only feasibility study authorized on 2026-08-31. This contract
authorizes public-source research and repository documentation only. It does
not authorize a platform probe, product implementation, connection capture,
provider access, inference, privilege change, package, publication, or release.

## Purpose

The [planning note](bounded-network-observability-planning-note.md) asks whether
AI Engine Room could help a non-specialist understand where supported local-AI
processes appear to communicate while preserving the product's passive,
bounded, evidence-qualified posture.

This first slice examines whether the verified Windows and Linux baselines
offer useful read-only, unelevated connection metadata and process association.
It must distinguish what official platform interfaces expose from what still
requires native verification.

## Authorized deliverables

This slice may add only:

- this bounded contract;
- one source-linked platform feasibility assessment;
- a reconciliation of the original planning note;
- a roadmap update; and
- deterministic readiness assertions for those documents.

The study may inspect tracked source and current primary platform
documentation. It must not run a network command, inspect a live connection or
process table, install a dependency, call a provider, or collect machine
evidence.

## Required questions

The assessment must address:

1. IPv4 and IPv6 TCP visibility on Windows and Linux;
2. whether the same facilities expose a useful UDP remote destination;
3. which process identifier, user identifier, and socket identifier fields are
   available;
4. the separate permissions required to turn a process identifier or socket
   inode into a supported executable identity;
5. snapshot and event limitations, including short-lived connections;
6. network-namespace, helper-process, protected-process, and PID-reuse limits;
7. an address-scope vocabulary that does not imply purpose, trust, privacy, or
   actual routing;
8. the smallest coherent cross-platform probe, if one is feasible; and
9. explicit adopt, probe, or defer conditions.

## Required evidence boundaries

The study must preserve these distinctions:

- an endpoint table is a point-in-time observation, not packet history;
- an externally addressed connection does not identify transmitted content;
- a loopback endpoint does not prove where computation occurred;
- a PID or socket inode is not durable process identity;
- process attribution can fail or become stale without invalidating other
  observed connection fields;
- no observed connection is not evidence that none occurred; and
- address classification is not a safety, privacy, ownership, reputation, or
  destination-purpose judgement.

## Privilege and privacy boundary

The main application must remain unelevated. Partial visibility must be
represented as partial or unavailable evidence. This study must not justify an
optional privileged helper; that would require a later threat model and its
own developer gate.

Any proposed probe must keep raw evidence in process memory only, avoid DNS and
reputation lookups, avoid packet payloads, and define an allow-list-first
projection before any report output is considered.

## Explicit exclusions

This slice does not authorize:

- packet capture, payload inspection, TLS interception, or certificate work;
- firewall, route, DNS, proxy, provider, or model changes;
- ETW, eBPF, audit, netfilter, packet-filter, or kernel-tracing setup;
- continuous, background, persistent, or undisclosed monitoring;
- scanning or recording unrelated process command lines;
- administrator, root, capability, debug-privilege, or service installation;
- a native synthetic probe or use of physical verification machines;
- product data types, commands, interface elements, or report fields; or
- a claim that an interaction was local, private, safe, or free of external
  communication.

## Acceptance criteria

This study is ready for merge review only if:

- it cites primary platform documentation;
- it states platform asymmetries and permission boundaries plainly;
- its recommendation is narrower than a product-adoption decision;
- any subsequent probe requires a new exact developer-approved contract;
- no live or external machine evidence is represented as collected; and
- deterministic repository checks pass.

## References

- [Bounded network-observability planning note](bounded-network-observability-planning-note.md)
- [Platform feasibility assessment](../research/network-observability-platform-feasibility.md)
- [Architecture](architecture.md)
