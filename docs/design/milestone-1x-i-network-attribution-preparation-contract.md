# Milestone 1X-I network-attribution preparation contract

## Status

Static preparation only on 2026-08-31. This contract authorizes a bounded
procedure correction and documentation reconciliation. It authorizes no
workflow, installer execution, retry, manifest, submission, signing, release,
provider access, inference, or product network-observability implementation.

## Decision

Milestone 1X-H observed an established non-loopback connection during the
bounded application launch, recorded its owner PID, and stopped. It did not
preserve the process name or parent relationship for that PID. The evidence
therefore cannot distinguish the application process from a WebView2 helper or
another descendant, and it cannot establish the connection's purpose or
content.

The correct next step is not to ignore or allow-list the connection and not to
repeat the same run. The static verification procedure will first preserve
bounded owner-PID attribution at the moment a connection triggers the existing
stop. Any later execution remains separately gated.

## Why launch remains in scope

Current Windows Package Manager community-repository documentation says that
submitted packages must install without required user interaction. Its
[validation FAQ](https://github.com/microsoft/winget-pkgs/blob/master/doc/FAQ.md)
also says the installed application is run after installation to identify
unexpected processes. The repository's
[validation failure guide](https://github.com/microsoft/winget-pkgs/blob/master/doc/ValidationFailureGuide.md)
separately describes executable discovery and uninstall validation. An
install-only rerun would therefore answer less than the intended readiness
question and would not close Milestone 1X-H.

Microsoft documents that the Evergreen WebView2 Runtime has its own update
mechanism and network-manageable policies in its
[enterprise guidance](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/enterprise).
That general fact does not attribute the observed `150.171.22.17:443`
connection to WebView2, Microsoft, an update, telemetry, or any other purpose.
No such attribution is made here.

## Authorized static correction

For each non-loopback TCP connection already selected by the procedure, record
only:

- the existing phase, owner PID, remote address, remote port, and TCP state;
- the owner process name when an immediate owner-PID lookup can resolve it;
- whether the owner is the launched root or a discovered descendant;
- the immediate parent PID and process name when available; and
- an explicit `unavailable` attribution state when the process exits or cannot
  be resolved.

Do not record command lines, executable paths, usernames, environment values,
DNS names, packet content, payload sizes, or broader process inventory. The
existing sorted process-name set remains supporting evidence, not a substitute
for owner-PID attribution.

The lookup is a bounded snapshot after the connection-table observation. It
can be unavailable or stale if a process exits or a PID is reused, and it does
not by itself prove why that process opened the connection.

## Preserved boundaries

The procedure must continue to stop on the first observed `SynSent` or
`Established` non-loopback TCP connection in the monitored process tree. This
slice must not:

- allow-list an address, process, publisher, network owner, or presumed service;
- perform reverse DNS, WHOIS, reputation, geolocation, or other network lookup
  from the disposable runner;
- infer that a prompt, response, model, telemetry, update, or other particular
  content was transmitted;
- suppress, retry, delay past, or downgrade the connection stop;
- collect packet content or add persistent/background monitoring;
- add privileges, firewall rules, routing changes, certificates, or traffic
  interception; or
- turn this distribution probe into product feature implementation.

Cleanup after any future stopped installer execution remains mandatory. A
successful cleanup remains containment evidence, not the planned primary
silent-removal result.

## Acceptance criteria

This preparation is complete only when:

1. the tracked procedure records the bounded attribution fields above;
2. PowerShell parses the procedure successfully;
3. the repository still contains no workflow that invokes the procedure;
4. deterministic repository checks pass;
5. documentation preserves the 1X-H stopped result and its non-claims; and
6. the diff contains no manifest, submission, release, signing, product, or
   external-system change.

## Future gate

Completion grants no execution authority. A future Milestone 1X-J proposal
would have to identify an exact procedure commit, unchanged candidate and
runner boundary, one-run trigger, evidence projection, cleanup behavior, and
no-bypass stop conditions. The developer must separately approve both that
execution change and exactly one run. Deferral remains an acceptable outcome.

## References

- [Milestone 1X-H contract](milestone-1x-h-disposable-windows-rerun-contract.md)
- [Disposable-runner verification record](../distribution/winget-disposable-runner-verification-record.md)
- [Network-observability planning note](bounded-network-observability-planning-note.md)
- [WinGet community repository policies](https://github.com/microsoft/winget-pkgs/blob/master/doc/Policies.md)
- [WinGet authoring and testing guidance](https://github.com/microsoft/winget-pkgs/blob/master/doc/Authoring.md)
