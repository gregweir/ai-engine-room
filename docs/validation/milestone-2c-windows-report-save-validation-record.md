# Milestone 2C Windows report-save validation disposition

## Disposition

Accept the bounded Windows native evidence for cancellation, clipboard copy,
new-file content, and existing-file no-clobber behavior. No product defect was
established, and no additional Windows run is required.

This is not a complete pass of every checkpoint in the earlier validation
contract. The operator saved the new report in the parent temporary location
rather than the prepared child directory, so the result establishes the exact
saved content but not the helper's intended destination-path discipline. The
outer launcher timer also expired before the application was closed normally,
so no launcher exit code was captured. Independent cleanup confirmed that the
application process, report file, controlled sentinel, and report clipboard
content were absent afterward.

## Exact candidate

- Product source: `5f54ec00cbfd884a0ffbce956d586d8ac8f5a199`
- Product tree: `7aa645875fc4dcd1b28e91eb209a073990bd1877`
- Windows NSIS package SHA-256:
  `e7345f4e5928d1d01d7f39e8e9e1a6493328df53543598715ecb80cf1dab2d1a`
- Windows executable SHA-256:
  `4e886978e0514073c2b10909f6ab379f5eb6531941197e0ce6772c72e2e4ef54`
- Execution context: Windows 11 x86-64, ordinary Medium-integrity graphical
  user

The NSIS package was retained but was not installed or executed during this
validation.

## Accepted evidence

- Cancelling the native save dialog created no report file and returned focus
  with the expected polite status.
- Copying once placed the exact 539-byte UTF-8 preview on the clipboard, with
  SHA-256
  `e5b50d3667c117c8ea1f2117e2a338cc8765392039b19a68acbaf64ef0684105`.
- Saving a new file produced the same 539 bytes and SHA-256. Strict UTF-8
  decoding passed, no UTF-8 byte-order mark was present, and saving did not
  change the clipboard sentinel. The operator-selected parent temporary
  destination qualifies this evidence as content validation rather than the
  prepared-directory path check.
- Selecting Replace for the controlled existing-file sentinel did not replace
  it. The application showed its no-clobber alert, Copy report remained
  available, and the 35-byte sentinel retained SHA-256
  `aebeb69e9c773c0c0e719bf88c36e2a035c795890ad2cc8450937ea64c0ded03`.
- No staging file remained. The application was closed through its normal
  window control after the outer timer expired. Cleanup then confirmed no
  application process, report artifact, controlled sentinel, or report text on
  the clipboard. The candidate remained retained and uninstalled.

## Claim boundary

The accepted evidence is limited to the exact Windows candidate and the four
behaviors above. It does not establish native stale-preview rejection,
inaccessible-destination handling, enlarged-text presentation, installer
behavior, other Windows versions or machines, universal filesystem safety,
publication, or release readiness.

Together with the separately accepted
[Ubuntu disposition](milestone-2c-ubuntu-report-save-validation-record.md),
this completes the currently justified Milestone 2C native evidence scope. It
does not broaden either platform's claim boundary or authorize another native
run.
