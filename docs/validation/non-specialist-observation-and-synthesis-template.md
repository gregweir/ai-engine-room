# Non-specialist validation observation and synthesis template

Copy the **Session observation** section for each participant. Use IDs such as
`P01`; do not record names or contact details. Complete **Round synthesis** only
after all planned sessions finish.

Completed copies are session records, not public project templates. Do not
commit them to this public repository; keep them in developer-controlled local
storage until an evidence-handling decision is approved.

## Session observation

### Session scope

- Session ID:
- Date:
- Facilitator:
- Public preview version: `v0.1.0-preview.1`
- Platform baseline: Ubuntu verified baseline / Windows verified baseline
- Provider condition: no provider / facilitator-provided safe provider
- Broad desktop-app familiarity: occasional / regular / advanced
- Broad local-AI familiarity: none / some / regular
- Consent confirmed: yes / no
- Recording used: no

Do not continue or retain observation notes when consent is `no`.

### Task outcomes

Use only **Independent**, **One neutral prompt**, or **Blocked or
misunderstood**. Paraphrase behavior; do not include private data, exact machine
metrics, precise identifiers, or private model names.

| Task | Outcome | Observed behavior or participant explanation | Issue IDs |
| --- | --- | --- | --- |
| 1. First impression |  |  |  |
| 2. Download and verification decision |  |  |  |
| 3. Refresh and provider boundaries |  |  |  |
| 4. Workspaces and missing evidence |  |  |  |
| 5. Optional observed inference |  |  |  |
| 6. Report and clipboard |  |  |  |

### Critical comprehension record

Record **Understood before explanation**, **Understood after explanation**, or
**Not understood**.

| Boundary | Result | Evidence, paraphrased |
| --- | --- | --- |
| Refresh does not run or authorize inference |  |  |
| The app does not manage providers or models |  |  |
| Observed inference is optional and authorized per run |  |  |
| Missing evidence is not estimated |  |  |
| A checksum does not authenticate the publisher |  |  |
| Loopback scope does not prove compute placement |  |  |
| Copy report writes to a clipboard other apps may read |  |  |

### Findings

Severity definitions:

- **Critical:** could encourage an unsafe action, undisclosed inference,
  weakened security control, false publisher trust, sensitive-data exposure, or
  a materially false claim about evidence or compute placement.
- **Major:** blocks a core documentation or navigation task, or creates a
  repeated incorrect mental model without immediate safety impact.
- **Moderate:** causes delay or requires help but leaves the core boundary
  understandable.
- **Minor:** wording, discoverability, or presentation friction that does not
  alter the meaning.

| ID | Severity | Observed evidence | Documentation or product location | Possible response; not yet approved | Retest needed |
| --- | --- | --- | --- | --- | --- |
|  |  |  |  |  | yes / no |

### Closing-question notes

- Clearest area:
- Hardest area:
- Unexpected expectation:
- Unsigned-preview decision concern:
- Facilitator limitations or deviations:

## Round synthesis

### Round identity and limits

- Round ID:
- Session dates:
- Number of consenting participants:
- Platforms represented:
- Provider conditions represented:
- Deviations from the session guide:
- Known sampling or facilitation limitations:

This is a small formative round. Do not describe counts as percentages or claim
that they represent users generally.

### Aggregate task evidence

| Task | Independent count | One-neutral-prompt count | Blocked-or-misunderstood count | Repeated issue IDs |
| --- | ---: | ---: | ---: | --- |
| 1. First impression |  |  |  |  |
| 2. Download and verification decision |  |  |  |  |
| 3. Refresh and provider boundaries |  |  |  |  |
| 4. Workspaces and missing evidence |  |  |  |  |
| 5. Optional observed inference |  |  |  |  |
| 6. Report and clipboard |  |  |  |  |

### Safety-critical synthesis

- Critical findings:
- Participants who required explanation for a critical boundary, by session
  ID only:
- Whether any participant attempted a stopped action:
- Required remediation before another round:

One critical finding prevents any expanded readiness claim. Remediate the
relevant documentation or product boundary under a separately approved change,
then repeat the affected tasks with fresh participants.

### Prioritized findings

| Priority | Issue IDs | Sessions affected | Evidence pattern | Proposed next decision |
| ---: | --- | --- | --- | --- |
| 1 |  |  |  |  |

A repeated major finding also requires remediation and targeted retesting.
Moderate and minor findings may be grouped only when their evidence and scope
are the same.

### Permitted conclusion

Choose one and preserve the stated limitations:

- **Preparation only:** no session evidence exists yet.
- **Round incomplete:** fewer than the planned consenting sessions completed or
  a material protocol deviation prevents synthesis.
- **Remediation required:** a critical or repeated major finding requires a
  separately approved change and retest.
- **Formative round complete:** the recorded tasks and critical boundaries were
  completed without a critical or repeated major finding. This supports only a
  narrow statement about the named participants, artifact, documentation,
  platforms, and sessions—not broad usability or readiness.

Selected conclusion:

### Evidence-backed follow-up proposals

List proposals without implementing them. Mark each as documentation, product,
validation, packaging, signing, or release work so the correct separate gate can
be applied.
