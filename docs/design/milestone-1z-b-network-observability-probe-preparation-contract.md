# Milestone 1Z-B network-observability probe-preparation contract

## Status

Documentation-only preparation authorized on 2026-08-31. This contract defines
an evidence model and a possible later synthetic platform probe. It does not
authorize implementing or running that probe, inspecting a live connection or
process table, contacting an endpoint, or adopting a product feature.

## Purpose

The [Milestone 1Z-A assessment](../research/network-observability-platform-feasibility.md)
found that a small, unelevated TCP snapshot probe is plausible on the verified
Ubuntu and Windows baselines, but that process attribution and missed
short-lived connections need native evidence.

This slice resolves the meanings, bounds, fixtures, retained fields, and stop
conditions before any machine evidence may be collected. The complete
preparation is in the
[evidence and synthetic-probe plan](network-observability-evidence-and-synthetic-probe-plan.md).

## Authorized deliverables

This slice may add only:

- this preparation contract;
- one evidence-model and synthetic-probe design;
- status updates to the planning note and roadmap; and
- deterministic repository-readiness assertions for those documents.

Tracked source, tests, and public platform documentation may be read. No native
probe source, executable fixture, workflow, product type, command, interface,
report field, dependency, or release material may be added or changed.

## Fixed preparation boundary

The proposed later probe is limited to TCP IPv4 and IPv6 snapshots. It defines
exactly 11 samples at 500-millisecond intervals, spanning no more than five
seconds from the first through the last sample, with a 15-second fail-closed
outer timeout for the entire probe process. A later execution contract may
narrow these values but must not broaden them.

The plan defines controlled long-lived loopback and externally addressed
connections plus deliberately short-lived loopback connections. External
fixtures remain unusable until a later execution contract pins numeric
addresses, ports, endpoint control, retention, and data-handling terms. There
is no fallback to a public website, provider, model service, DNS name, or
opportunistically available endpoint.

## Required evidence separation

The preparation must keep these layers distinct:

1. **Raw observed fact** — numeric endpoint and platform ownership fields held
   only in probe-process memory.
2. **Derived classification** — address scope and bounded process-association
   outcomes produced by deterministic rules.
3. **Sanitized retained result** — fixture labels, counts, sample indexes,
   classifications, acquisition outcomes, and limitations only.
4. **Unavailable evidence** — packet content, byte counts, DNS identity,
   destination purpose, complete history, inaccessible processes, and activity
   outside the window.

An observed connection must never become a statement about transmitted prompt
or response content. A missed fixture must never become a statement that no
communication occurred.

## Approval boundary for later execution

Merging this preparation does not authorize a run. Any native evidence
collection requires all of the following in a new developer-approved contract:

- exact source commit and complete probe/fixture source;
- exact Ubuntu and Windows baseline identities;
- exact numeric external endpoints and evidence that they are controlled for
  this narrow test;
- complete commands, expected process identities, and termination checks;
- one explicitly bounded run authorization per platform; and
- independent review of sanitized evidence before an adopt-or-defer proposal.

A general continuation instruction, this contract's merge approval, or an
approval that does not identify the execution commit and platform is
insufficient. A cancelled, stopped, failed, or incomplete attempt consumes its
platform's one-run authorization.

## Explicit exclusions

This slice does not authorize:

- socket-table, process-table, network-namespace, route, DNS, or firewall
  inspection;
- opening a listener or connection, sending application data, or contacting a
  provider, model service, website, or test endpoint;
- packet capture, payload inspection, TLS interception, ETW, eBPF, audit,
  netfilter, packet-filter, or kernel-tracing setup;
- administrator, root, capability, debug-privilege, service, driver, or helper
  installation;
- continuous, background, persistent, undisclosed, or unrelated-process
  monitoring;
- storing raw IP addresses, ports, PIDs, UIDs, inodes, executable paths,
  usernames, hostnames, command lines, or unrelated connection rows;
- product implementation, interface or report changes, packaging, signing,
  publication, or release; or
- claims that an interaction is local, private, safe, or free of external
  communication.

## Preparation acceptance

This contract is ready for merge review only if the branch contains
documentation and deterministic-guardrail changes alone, defines exact
non-expanding bounds and fail-closed conditions, leaves every platform run
separately gated, and ordinary deterministic checks pass.

## References

- [Milestone 1Z-A contract](milestone-1z-a-network-observability-feasibility-contract.md)
- [Platform feasibility assessment](../research/network-observability-platform-feasibility.md)
- [Evidence and synthetic-probe plan](network-observability-evidence-and-synthetic-probe-plan.md)
- [Bounded network-observability planning note](bounded-network-observability-planning-note.md)
