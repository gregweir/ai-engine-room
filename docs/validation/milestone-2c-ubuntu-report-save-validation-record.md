# Milestone 2C Ubuntu report-save validation disposition

## Disposition

Accept the bounded Ubuntu native evidence for cancellation, clipboard copy,
new-file saving, and no-clobber behavior. No product defect was established,
and no additional Ubuntu run is required.

This is not a complete pass of the earlier validation contract. The attempted
stale-preview checkpoint did not prove that the asynchronous Refresh had
completed before the native dialog returned, so it creates no native
stale-concurrency claim. Deterministic coverage remains the evidence that an
actual preview-generation change produces `PreviewChanged` without staging or
committing a file. Windows native validation remains separate and pending.

## Exact candidate

- Product source: `5f54ec00cbfd884a0ffbce956d586d8ac8f5a199`
- Product tree: `7aa645875fc4dcd1b28e91eb209a073990bd1877`
- Ubuntu package SHA-256:
  `eae771fcee89f31b5ecfb5154c9fa71ff2ce94634228ca0599b7dcdcae6b438e`
- Ubuntu executable SHA-256:
  `482a6e302469d9340d1b95337a0f9aa864367617421125fa5ad380f13f94599f`
- Corrected local-console coordinator:
  `f1336aad0e67a4fc07f804ca5e125d5ea18fe734`
- Helper blob: `13d3260100e5254265691b3767a99de9b905536e`

## Accepted evidence

- Cancelling the native save dialog created no report file and returned focus
  with the expected polite status.
- Copying once placed the exact 636-byte UTF-8 preview on the clipboard.
- Saving a new file produced the same 636 bytes and SHA-256
  `f6020605a4908536663276051f1d75a849979f3ec19c6919ba78406e05a039d4`.
- Selecting Replace for the controlled existing-file sentinel did not replace
  it. The application showed its no-clobber alert, and the 35-byte sentinel
  retained SHA-256
  `aebeb69e9c773c0c0e719bf88c36e2a035c795890ad2cc8450937ea64c0ded03`.
- The operator stopped at the stale-preview checkpoint. The application then
  closed normally, disposable files were removed, and postflight found no app
  process or installed package. The retained candidate hashes were unchanged.

## Claim boundary

The accepted evidence is limited to the exact Ubuntu candidate and the four
behaviors above. It does not establish native stale-preview rejection,
inaccessible-destination handling, enlarged-text presentation, Windows
behavior, broader Linux compatibility, packaging acceptance, publication, or
release readiness.
