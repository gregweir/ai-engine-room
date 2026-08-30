# SignPath Foundation eligibility assessment

Status: bounded read-only assessment completed and reconciled with the public
unsigned-preview state on 2026-08-30. No application, account, certificate,
signing policy, credential, workflow, GitHub setting, or external service was
created or changed.

This is an engineering readiness assessment against the public SignPath
Foundation conditions, not an eligibility decision by SignPath and not legal
advice. SignPath Foundation's published conditions are authoritative:
<https://signpath.org/terms.html>.

## Current assessment

| Condition | Result | Public evidence or gap |
| --- | --- | --- |
| Public open-source project | Pass | `gregweir/ai-engine-room` is public and GitHub identifies Apache-2.0. |
| OSI-approved licence without commercial dual licensing | Pass for the current repository | Application metadata declares only Apache-2.0; the generated distribution inventory records open-source dependency licences. SignPath retains final review authority. |
| No proprietary project component | Pass for tracked distribution inputs, subject to external review | The public source, locked inputs, generated licence inventory, and installer-helper provenance are reviewable. Runtime interoperability with separately installed providers does not bundle those providers. |
| Actively maintained | Pass | Current public history records active implementation, verification, and remediation work. |
| Already released in the form to be signed | Pass for the current unsigned preview form, subject to SignPath review | The public [`v0.1.0-preview.1` release](https://github.com/gregweir/ai-engine-room/releases/tag/v0.1.0-preview.1) provides the exact accepted `.deb` and NSIS forms that a future workflow could rebuild for signing. |
| Functionality documented on a download page or store entry | Pass for the current preview, subject to SignPath review | The public release page documents functionality, scope, limitations, exact source, downloads, verification, privacy, support, and security links. |
| Code signing policy linked from project and release pages | Not met | No SignPath code-signing policy or release-page link has been adopted. |
| Required privacy statement | Partial | README documents the product privacy boundary, but it is not yet incorporated into the specific SignPath policy wording and release-page structure. |
| MFA for repository and SignPath access | Unverified | Public read-only evidence cannot establish every participating account's MFA state. Developer attestation and service-side verification would be required. |
| Named authors, reviewers, and signing approvers | Not met | The repository does not yet publish the role mapping required by SignPath. |
| Trusted build and origin verification | Partial | Deterministic GitHub Actions checks exist, but they intentionally do not upload artifacts, publish, or request signing. A separately reviewed signing workflow would be required. |

## Cost and identity trade-off

SignPath states that its service is free for accepted open-source projects:
<https://signpath.org/>. Under the Foundation route, the code-signing
certificate is issued to **SignPath Foundation**. It would therefore not display
`Tartanleaf.com Inc.` as the Authenticode certificate publisher. The developer
has confirmed Greg Weir as the developer and Tartanleaf.com Inc. as the product
and package publisher. Whether a future signed build may instead display
SignPath Foundation as its authenticated certificate publisher remains a
separate future decision.

The Foundation also requires controlled build origin, signing roles, approvals,
MFA, a code-signing policy, and continuing compliance. Free monetary cost does
not remove the engineering and governance work needed to qualify and remain
eligible.

## Readiness conclusion

AI Engine Room is a plausible candidate but is **not application-ready** today.
The previously identified public unsigned preview and download-page prerequisite
is complete. The remaining minimum sequence before an application decision is:

1. decide whether the certificate publisher may be SignPath Foundation rather
   than Tartanleaf.com Inc.;
2. define public signing roles and obtain the required MFA attestations;
3. draft the required code-signing policy and privacy statement; and
4. design a trusted, reviewable GitHub build-origin and signing-request workflow
   without introducing reusable signing secrets into the repository.

No step above authorizes an application, signing request, artifact upload,
publication, or GitHub/service setting change. Each remains separately gated.
