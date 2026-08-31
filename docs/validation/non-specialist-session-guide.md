# Non-specialist formative-validation session guide

## Purpose and limits

This guide tests whether a person outside the project can use the public
documentation and an already installed preview to form an accurate mental
model. It is a formative session, not a participant exam, benchmark,
accessibility audit, compatibility test, or production-readiness review.

Run the same core tasks with a small round of three to five willing adults. A
participant should use desktop software but must not have contributed to AI
Engine Room. Familiarity with local AI may vary and should be recorded only in
broad, task-relevant terms.

Expected duration: 30–40 minutes.

## Prepare the environment

Before the participant arrives:

1. Use a facilitator-controlled Ubuntu 24.04 LTS x86_64 or Windows 11 25H2
   build 26200.7462 x64 machine within the accepted public-preview scope.
2. Confirm that the exact accepted `v0.1.0-preview.1` package is already
   installed. Do not make unsigned installation or a platform warning part of
   the participant session.
3. Open the public README, user guide, glossary, roadmap, and official release
   page. Make no private repository or internal evidence available.
4. Use the no-provider state by default. If a supported provider is already
   available for observation, ensure its visible identities are safe to show
   and do not run observed inference.
5. Close unrelated applications and remove visible notifications. Do not expose
   account data, personal files, precise machine identifiers, clipboard
   contents, or private model names.
6. Prepare a fresh copy of the
   [observation template](non-specialist-observation-and-synthesis-template.md)
   using a session ID such as `P01`.

Do not record audio, video, photographs, or the screen. Do not add telemetry or
analytics. Notes should contain paraphrased behavior rather than personally
identifying detail.

## Opening script

Read this in substance, without promising that the product is safe or ready:

> We are evaluating the documentation and product explanation, not you. Please
> think aloud and show what you would do. You may skip any task or stop at any
> time. I will take de-identified notes, but I will not record audio, video, or
> your screen. Please avoid opening personal files or accounts. Do you agree to
> continue?

Stop unless the participant clearly agrees. Record only `Consent confirmed:
yes`; do not record a signature, name, or contact detail.

## Facilitation rules

- Start each task with only the prompt shown below.
- Let the participant work and think aloud. Do not teach, praise, correct, or
  reveal the intended path during the task.
- If they pause, one neutral prompt is allowed: **“What would you look for
  next?”** Record that prompt was used.
- Stop an action that would expose private data, run inference, change provider
  or network configuration, weaken a security control, bypass policy, open an
  installer, or transmit clipboard contents.
- After recording an outcome, explain any important misunderstanding before
  moving on. Help is part of the session but is not independent completion.
- Use only these outcome labels: **Independent**, **One neutral prompt**, or
  **Blocked or misunderstood**.

## Core tasks

### Task 1 — First impression

Starting from the top of the README, ask:

> In your own words, what is AI Engine Room for, and what important things does
> it not do?

Observe whether the participant identifies understanding or observation rather
than provider management, optimization, benchmarking, repair, or autonomous
action.

### Task 2 — Decide what to download and trust

Provide the official release page and user guide. Ask:

> Show me how you would choose the file for this computer and decide whether
> the downloaded file matches what the project published. Tell me what that
> check can and cannot prove.

This is a documentation task. Do not download or open an installer. The
participant should find the platform asset and complete SHA-256 guidance and
should not treat a checksum as publisher authentication or a reason to bypass a
security control.

### Task 3 — Refresh and provider boundaries

Open the already installed application and ask:

> You want the newest information the app can observe. What would you do, and
> what do you expect that action to do?

Then ask:

> If a provider is not detected, what would you expect this app to do about it?

Observe whether the participant chooses **Refresh**, expects passive
reacquisition without inference, and understands that the app does not start or
configure a provider.

### Task 4 — Find and interpret the workspaces

Ask:

> Show me where you would look for a summary, model information, an explanation
> of a change, and a shareable text preview. Tell me what “Unavailable” or
> “Unknown” means when you encounter it.

Observe navigation among **Overview**, **Models**, **Diagnose**, and **Report**.
The participant should preserve missing evidence as missing rather than infer a
hidden value, fault, or recommendation.

### Task 5 — Optional observed inference

Use the workflow diagram and the observed-inference workspace without starting
a run. Ask:

> If you wanted one observed-inference attempt, what would happen before a
> request is sent? What permission, if any, would apply to a later run?

Then ask:

> If a provider is reached at a same-machine loopback address, what does that
> tell you—and what does it not prove—about where the model computes?

Observe whether the participant identifies the disclosure, authorization for
one run, fixed synthetic prompt, no carried permission, and the compute-placement
qualification. Do not authorize or run inference during the session.

### Task 6 — Report and clipboard

Open the Report workspace and ask:

> What would happen if you chose “Copy report”? Is there anything you would
> consider before doing that?

Do not copy during the session. Observe whether the participant understands
that the preview is copied only after an explicit action and that other
applications may read clipboard contents.

## Closing questions

Ask without offering feature ideas first:

1. What felt clearest?
2. What was hardest to understand or find?
3. What, if anything, did you expect the app to do that it did not do?
4. What would you want to verify before deciding whether to use this unsigned
   preview?

Thank the participant and explain any remaining safety-critical misunderstanding.
Do not promise a change or future release.

## After each session

1. Remove any accidentally captured personal or machine-specific detail from
   the notes.
2. Classify issues using the template's severity definitions.
3. Separate observed behavior from interpretation and proposed remediation.
4. Do not change public claims from one session or from unreviewed notes.
5. After the round, use the synthesis section to identify repeated findings,
   limitations, and evidence-backed next decisions.
