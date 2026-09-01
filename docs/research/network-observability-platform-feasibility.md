# Network-observability platform feasibility

Status: **DOCUMENTATION FEASIBILITY COMPLETE; NO PLATFORM PROBE** on
2026-08-31.

## Conclusion

A small, read-only TCP feasibility probe is supportable on both verified
platform baselines without designing an elevated main application. The public
interfaces can expose IPv4 and IPv6 TCP endpoint snapshots with remote
addresses, ports, and connection state. Windows includes an associated PID in
its owner-PID tables; Linux exposes a socket UID and inode that can sometimes
be associated with a process through permission-controlled `/proc` links.

That is not yet enough to adopt a product feature. Process identity and
short-lived-connection coverage need native evidence. UDP does not provide a
coherent cross-platform first slice because the documented Windows owner-PID
UDP rows describe bound local endpoints and do not include a remote address.

The recommended next step is a separately approved evidence-model and
synthetic-probe preparation contract. No platform probe, product
implementation, privilege change, or monitoring is authorized by this study.

## Scope

The assessed baselines are the project's tested Ubuntu 24.04 LTS x86_64 and
Windows 11 25H2 build 26200.7462 x64 environments. Public documentation is
evidence of interface shape, not proof of behavior on those exact machines.

The study asks only whether a future bounded probe could observe connection
metadata. It does not assess packet content, transmission counts, DNS names,
destination organizations, reputation, provider purpose, or compute placement.

## Platform findings

| Question | Windows | Linux | Feasibility interpretation |
| --- | --- | --- | --- |
| IPv4/IPv6 TCP snapshot | `GetExtendedTcpTable` supports owner-PID tables for both address families | `NETLINK_SOCK_DIAG` supports IPv4 and IPv6 socket queries | Plausible on both; native verification still required |
| Remote TCP endpoint | Owner-PID rows contain local/remote addresses and ports, state, and PID | `inet_diag_msg` contains local/remote addresses and ports, state, UID, and inode | Enough for a bounded TCP address-scope probe |
| Process association | PID is returned directly; executable identity requires a separate process query that may be denied | No PID is returned; socket inode must be matched to accessible `/proc/<pid>/fd` links | Linux attribution is more conditional and race-prone |
| UDP destination | Owner-PID UDP rows contain local address, local port, and PID, but no remote destination | Internet socket diagnostics can report UDP socket addresses, subject to socket state and namespace | Exclude UDP from the first cross-platform probe |
| Historical completeness | Table retrieval is a snapshot | Diagnostic dump is a snapshot | Both can miss short-lived connections |
| Cross-boundary visibility | Protected/system process identity may be inaccessible | Different users, non-dumpable processes, LSM policy, capabilities, and other network namespaces can limit attribution | Represent gaps; do not elevate to fill them |

### Windows

Microsoft documents `GetExtendedTcpTable` as returning application-visible TCP
endpoint tables. With `TCP_TABLE_OWNER_PID_*`, IPv4 and IPv6 rows include the
associated PID; the rows also expose local and remote addresses and ports and
TCP state. This is the narrowest documented Windows starting point because it
does not require packet capture or module-level ownership data.

The PID is not a durable executable identity. Resolving it requires a separate
process handle and image-name query. `OpenProcess` applies the target process's
security descriptor, can return access denied for restricted system processes,
and does not prevent a process from exiting or a PID from being reused between
observations. A future probe must treat identity lookup failure and stale PID
evidence as explicit outcomes.

`GetExtendedUdpTable` can return owner-PID tables for IPv4 and IPv6, but the
documented owner-PID row is a local bound endpoint: local address, local port,
and PID. It does not carry a peer address. It therefore cannot answer where a
UDP datagram went and cannot support parity with the TCP question.

### Linux

Linux socket diagnostics use a `NETLINK_SOCK_DIAG` socket. For IPv4 and IPv6,
`inet_diag_req_v2` can request TCP or UDP sockets and filter by state. Returned
`inet_diag_msg` values include local and destination addresses and ports,
socket state, UID, and inode. The interface reports named/bound sockets rather
than packet history.

The response does not include a PID. A process association can be attempted by
matching the socket inode to a `socket:[inode]` symbolic link under
`/proc/<pid>/fd`. The Linux manual documents that dereferencing those links is
subject to a `PTRACE_MODE_READ_FSCREDS` access check. Credentials, dumpability,
capabilities, and Linux Security Modules can therefore limit visibility. A
service, helper, container, or process in another network namespace may not be
attributable from the main application's context.

For privacy and efficiency, a future probe should not scan and retain every
process command line. It should first identify narrowly allow-listed candidate
processes using a separately reviewed identity rule, inspect only accessible
candidate file-descriptor links, and discard unrelated socket rows after
matching. Whether that sequence is reliable for supported provider families is
a native verification question, not an established fact.

## Snapshot and event boundary

Neither selected interface supplies packet history. A single snapshot can miss
a connection created and closed between calls. Comparing two snapshots can
show that a row appeared or disappeared, but cannot establish everything that
happened between them.

Platform-specific notification and tracing systems do not provide an adopted
cross-platform solution here. Linux socket diagnostics expose destroy-event
groups, but destroy notifications alone do not establish complete start and
ownership history. Windows and Linux tracing approaches would introduce a
different privilege, performance, privacy, and maintenance analysis. ETW,
eBPF, audit, netfilter, and packet capture remain outside this study.

The first probe should therefore test a strictly bounded set of snapshots and
measure what it misses. It must not call that sampling complete monitoring.

## Evidence vocabulary

A later evidence model should keep raw platform facts separate from address
classification and process inference:

- **Observed fact:** protocol, address family, connection state, local and
  remote numeric endpoint, platform PID or UID/inode, snapshot sequence, and
  acquisition outcome.
- **Derived address class:** `same_machine_loopback`, `local_or_private_scope`,
  `externally_addressed`, or `special_or_unresolved`, based on the current IANA
  IPv4 and IPv6 special-purpose registries.
- **Qualified process association:** `supported_process`, `helper_descendant`,
  `unattributed`, `permission_limited`, or `stale` only when a future contract
  defines evidence for each state.
- **Unavailable:** UDP remote destination on Windows, missed between-snapshot
  activity, inaccessible processes, other namespaces, transmitted content, and
  destination purpose.

`local_or_private_scope` must not mean same machine, secure, private, or unable
to reach the Internet. `externally_addressed` must not mean that packets
traversed a physical interface or that prompt content was transmitted. The
IANA registries themselves note special-purpose reachability properties; they
do not determine the route used by an observed socket.

## Smallest useful probe candidate

If separately authorized, the next preparation slice should specify one
synthetic, read-only native probe on each verified platform with all of these
limits:

1. TCP only, IPv4 and IPv6, with listeners excluded from user-facing results.
2. One explicit start action and an automatically ending window.
3. A small fixed maximum number of snapshots and a short fixed maximum
   duration, selected before execution.
4. Known synthetic loopback and externally addressed test connections whose
   process identities are controlled by the test procedure; no provider or
   inference activity.
5. Raw rows held only in current-process memory and no DNS, reputation, packet,
   payload, or command-line collection.
6. Exact counts for observed, deduplicated, unattributed, permission-limited,
   stale, and expected-but-missed cases.
7. Guaranteed termination and evidence that no background observer remains.
8. Sanitized retained evidence containing address classes and synthetic labels,
   not physical hostnames, usernames, paths, or unrelated endpoints.

This is a probe candidate, not an implementation design. Exact duration,
snapshot count, fixtures, executable-identity rules, data types, and retention
schema remain for the next contract.

## Decision

Proceed only to evidence-model and synthetic-probe preparation. Do not yet
adopt network observability as a product feature.

Adoption should require native evidence that both baselines can reliably show
at least loopback versus externally addressed TCP observations for controlled
same-user processes; that attribution failures are understandable; that the
bounded sampler terminates; and that the presentation does not imply complete
history or content inspection. Defer the feature if Linux attribution is too
incomplete, meaningful coverage requires elevation, or missed short-lived
connections make the result predictably misleading.

## Primary references

- [Microsoft: GetExtendedTcpTable](https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable)
- [Microsoft: MIB_TCPTABLE_OWNER_PID](https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcptable_owner_pid)
- [Microsoft: GetExtendedUdpTable](https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedudptable)
- [Microsoft: MIB_UDPROW_OWNER_PID](https://learn.microsoft.com/en-us/windows/win32/api/udpmib/ns-udpmib-mib_udprow_owner_pid)
- [Microsoft: OpenProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-openprocess)
- [Linux man-pages: sock_diag(7)](https://man7.org/linux/man-pages/man7/sock_diag.7.html)
- [Linux man-pages: proc_pid_fd(5)](https://man7.org/linux/man-pages/man5/proc_pid_fd.5.html)
- [Linux man-pages: ptrace(2)](https://man7.org/linux/man-pages/man2/ptrace.2.html)
- [Linux kernel: socket diagnostic implementation](https://github.com/torvalds/linux/blob/master/net/core/sock_diag.c)
- [IANA IPv4 special-purpose registry](https://www.iana.org/assignments/iana-ipv4-special-registry/)
- [IANA IPv6 special-purpose registry](https://www.iana.org/assignments/iana-ipv6-special-registry/)
