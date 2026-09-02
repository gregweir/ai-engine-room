import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const path = "scripts/milestone-2c-ubuntu-local-console-helper.sh";
const source = readFileSync(path, "utf8");
const terminalA = source.slice(
  source.indexOf("terminal_a() {"),
  source.indexOf("terminal_b() {"),
);
const terminalB = source.slice(
  source.indexOf("terminal_b() {"),
  source.indexOf("usage() {"),
);
const cleanup = source.slice(
  source.indexOf("controlled_cleanup() {"),
  source.indexOf("on_exit() {"),
);

assert.match(source, /^#!\/usr\/bin\/env bash\r?$/m);
assert.match(source, /^set -euo pipefail\r?$/m);
assert.match(source, /^umask 077\r?$/m);
assert.match(source, /^exec 3>&2\r?$/m);
assert.match(source, /^exec 2>\/dev\/null\r?$/m);
assert.match(source, /terminal-a\|terminal-b/);
assert.match(source, /SSH_CLIENT/);
assert.match(source, /SSH_CONNECTION/);
assert.match(source, /SSH_TTY/);
assert.match(source, /fail "SSH and remote shells are prohibited"/);
assert.match(source, /"\$\{DISPLAY:-\}" == ":0"/);
assert.match(source, /"\$\{XDG_SESSION_TYPE:-\}" == "x11"/);
assert.match(source, /"\$\{XDG_RUNTIME_DIR:-\}" == "\$expected_runtime"/);
assert.match(source, /"\$\{DBUS_SESSION_BUS_ADDRESS:-\}" == "\$expected_bus"/);
assert.match(source, /"\$\{XAUTHORITY:-\}" == "\$expected_xauthority"/);
assert.match(source, /timeout --signal=TERM --kill-after=10s 1800s/);
assert.match(source, /"\$EXECUTABLE" >\/dev\/null 2>&1/);
assert.match(source, /AER_LAUNCHER_RESULT=%s/);
assert.match(source, /require_process_count 0/);
assert.ok(
  (source.match(/require_process_count 1/g) ?? []).length >= 10,
  "every mutating UI checkpoint must re-establish one live candidate",
);
for (const checkpoint of [
  "Save cancelled. No report file was created.",
  "Report copied to the system clipboard.",
  "Report saved as a plain-text file.",
  "That file already exists.",
  "stale_result=",
  "inaccessible_result=",
  "At normal and enlarged text",
]) {
  const checkpointIndex = terminalB.indexOf(checkpoint);
  assert.notEqual(checkpointIndex, -1, `missing checkpoint: ${checkpoint}`);
  assert.ok(
    terminalB
      .slice(checkpointIndex, checkpointIndex + 1200)
      .includes("require_process_count 1"),
    `checkpoint does not immediately re-establish liveness: ${checkpoint}`,
  );
}
assert.match(source, /mkdir --mode=700 -- "\$RUN_DIR"/);
assert.match(
  source,
  /xclip -selection clipboard -out 2>\/dev\/null \| bytes_and_sha/,
);
assert.match(source, /b\.decode\("utf-8"\)/);
assert.ok(
  source.includes(
    ['value.startswith(b"', "\\x", "ef", "\\x", "bb", "\\x", "bf", '")'].join(
      "",
    ),
  ),
);
assert.match(
  source,
  /rm -- "\$RUN_DIR\/saved\.txt" "\$RUN_DIR\/existing\.txt"/,
);
assert.match(source, /rmdir -- "\$RUN_DIR"/);
assert.match(source, /trap 'on_exit \$\?' EXIT/);
assert.match(source, /controlled_cleanup/);
assert.match(source, /STOP_ACTION=close_application_normally/);
assert.match(source, /FAILURE_CLEANUP=confirmed/);
assert.match(source, /FAILURE_CLEANUP=unconfirmed/);
assert.match(source, /FAILURE_CLEANUP=not_armed/);
assert.match(source, /CLEANUP_ARMED="false"/);
assert.match(source, /PROCESS_CLOSURE_ALLOWED="false"/);
assert.ok(
  terminalA.indexOf('mkdir --mode=700 -- "$RUN_DIR"') <
    terminalA.indexOf('CLEANUP_ARMED="true"'),
  "Terminal A cleanup must arm only after creating its absent run directory",
);
assert.match(cleanup, /if \[\[ "\$PROCESS_CLOSURE_ALLOWED" == "true" \]\]/);
assert.match(source, /remove_regular_file_if_present "\$RUN_DIR\/stale\.txt"/);
assert.match(source, /CLIPBOARD_CHANGED="true"/);
assert.match(source, /CLIPBOARD_CHANGED="false"/);
assert.ok(
  terminalB.indexOf('CLIPBOARD_CHANGED="true"') <
    terminalB.indexOf("clipboard_identity)"),
  "clipboard cleanup must arm before copied report bytes are inspected",
);
assert.match(cleanup, /xclip -selection clipboard -in >\/dev\/null 2>&1/);
assert.match(cleanup, /process_count/);
assert.match(cleanup, /Close AI Engine Room using its window close control/);
assert.match(cleanup, /remove_regular_file_if_present/);
assert.match(cleanup, /directory_is_empty/);
assert.doesNotMatch(cleanup, /\b(?:kill|pkill|killall)\b/);
assert.match(terminalA, /CONTROLLER NO_CONTROLLER/);
assert.match(terminalA, /TERMINAL_A_CLEANUP_HANDOFF=terminal-b/);
assert.match(
  terminalA,
  /if \[\[ "\$controller_state" == "CONTROLLER" \]\][\s\S]*CLEANUP_ARMED="false"[\s\S]*return 0/,
);
assert.match(
  terminalA,
  /controlled_cleanup\r?\n\s+fail "the application ended without an active Terminal B controller"/,
);
assert.match(
  terminalB,
  /require_process_count 1\r?\n\s+CLEANUP_ARMED="true"\r?\n\s+PROCESS_CLOSURE_ALLOWED="true"/,
);
assert.match(
  terminalB,
  /require_process_count 1\r?\n\s+\[\[ -f "\$RUN_DIR\/existing\.txt" && ! -L "\$RUN_DIR\/existing\.txt" \]\] \|\| fail "existing destination type changed"\r?\n\s+local existing_after_bytes/,
);
assert.match(source, /TERMINATION_CHECK=fail LAUNCHER_RESULT=%s/);
assert.match(source, /if \[\[ "\$launcher_result" != "0" \]\]/);
assert.match(source, /fail "launcher result was not zero"/);
assert.doesNotMatch(
  source,
  /printf 'TERMINATION_CHECK=pass LAUNCHER_RESULT=%s/,
);
assert.doesNotMatch(
  source,
  /^\s*(?:kill|pkill|killall|sudo|apt(?:-get)?|curl|wget|scp|rsync)\b/m,
);
assert.doesNotMatch(source, /xdotool|wmctrl|ydotool|pyautogui/);
assert.doesNotMatch(source, /tee|logger|journalctl|dmesg/);
assert.doesNotMatch(source, /find\s+[^\n]*-maxdepth\s+[2-9]/);
assert.doesNotMatch(source, /find\s+[^\n]*-delete/);
assert.doesNotMatch(source, /rm\s+-[A-Za-z]*r/);
assert.doesNotMatch(source, /cat\s+[^\n]*(saved|clipboard|report)/i);

for (const expected of [
  "Save AI Engine Room report",
  "ai-engine-room-report.txt",
  "Plain text",
  "Saving report…",
  "Save cancelled. No report file was created.",
  "Report copied to the system clipboard.",
  "Report saved as a plain-text file.",
  "That file already exists. AI Engine Room did not replace it. Choose a different name.",
  "The report changed before saving. Review it and try again.",
  "Saving is not available for that location. You can still copy the report.",
  "Could not save the report. No completed report file was created.",
]) {
  assert.ok(source.includes(expected), `missing exact UI wording: ${expected}`);
}

console.log("Milestone 2C Ubuntu local-console helper contract verified.");
