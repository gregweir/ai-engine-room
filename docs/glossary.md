# AI Engine Room glossary

These terms describe the application's deliberately narrow evidence and action
boundaries.

## Available

A supported source or metric is ready and exposes the stated evidence. It does
not imply that other evidence exists or that the system is healthy, suitable,
or correctly configured.

## Bounded

Deliberately constrained in source, endpoint, data, duration, concurrency, and
action. Bounded does not mean sandboxed, independently secure, or guaranteed to
use local compute.

## Compute or execution placement

Where the model's actual computation occurs, such as CPU, GPU, or another
machine. Reaching a provider through a same-machine loopback API does not by
itself establish compute placement.

## Deterministic diagnosis

A fixed rule that turns accepted observations into a factual observation,
plain-language meaning, and safe next check. It is not a probabilistic opinion,
root-cause finding, health assessment, or automated repair.

## Explicit authorization

A deliberate user decision for one disclosed action. Authorization for one
observed-inference run does not carry over to another run and is not granted by
viewing a page or using Refresh.

## Failed

An actual acquisition attempt encountered an error, such as a timeout, parse
failure, measurement failure, or permission failure. Failed is different from
a normal source that has no value to report.

## Observation

A bounded piece of evidence acquired from an implemented operating-system or
provider source. An observation is not automatically a measurement of model
performance, a causal explanation, or a recommendation.

## Observation sequence

The newest bounded startup and explicit-Refresh observations retained in order
for the current app session. It has no timestamps or regular sampling and is
not a continuous monitor or time-based timeline.

## Observed inference

One optional, explicitly authorized request using a fixed synthetic prompt and
bounded execution controls. Its result describes that run; it is not a
benchmark or permission for later inference.

## Provider

A supported local-AI runtime whose implemented API supplies evidence. The
current integrations cover Ollama and LM Studio, plus passive-only traditional
single-model llama.cpp detection on its tested scope. AI Engine Room does not
start, stop, unload, reconfigure, or otherwise manage a provider.

## Provider-reported evidence

A value or identity returned by a provider API rather than independently
measured by AI Engine Room. It remains qualified by that source and must not be
silently combined with a different source into a new claim.

## Provenance

The source, method, unit, and qualifications that explain where an observation
came from and what it can support.

## Refresh

An explicit request to reacquire supported provider and machine observations.
Refresh does not run or authorize inference, manage a provider, copy a report,
or start continuous monitoring.

## Report-safe

Allowed by the project's fixed report policy for presentation in the
plain-text report. Report-safe does not mean anonymous in every context or safe
to share without review.

## Same-machine loopback

A numeric loopback endpoint, such as `127.0.0.1`, used to reach a service
through the network stack on the same machine. It limits the implemented API
destination but does not prove compute placement or mean that no network
request occurs.

## Session-only

Held in application memory for the current run and reset when the app restarts.
It does not mean other software cannot observe an explicitly copied clipboard
value.

## Synthetic prompt

The fixed, non-user-content prompt used for an authorized observed-inference
request. It exists to make the observation bounded, not to assess general model
quality or performance.

## Unavailable, unknown, and not detected

- **Unavailable** means the supported source cannot provide the particular
  value in the current evidence.
- **Unknown** means the evidence does not establish the answer and the app will
  not estimate it.
- **Not detected** means the bounded provider check did not find a responding
  supported provider in its implemented scope.

These states should remain visible rather than being converted into guessed
values or claims.

## Unsigned preview

A pre-release package without an accepted digital code signature. Matching its
published filename, byte size, and SHA-256 can verify exact bytes, but a
checksum does not authenticate the publisher. Greg Weir is the developer and
Tartanleaf.com Inc. is the product and package publisher; the Windows preview
does not contain an authenticated Authenticode publisher identity.
