import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const path = "scripts/milestone-2c-ubuntu-local-console-helper.sh";
const source = readFileSync(path, "utf8");

assert.match(source, /^#!\/usr\/bin\/env bash\r?$/m);
assert.match(source, /^set -euo pipefail\r?$/m);
assert.match(source, /^umask 077\r?$/m);
assert.match(source, /terminal-a\|terminal-b/);
assert.match(source, /SSH_CLIENT/);
assert.match(source, /SSH_CONNECTION/);
assert.match(source, /SSH_TTY/);
assert.match(source, /fail "SSH and remote shells are prohibited"/);
assert.match(source, /timeout --signal=TERM --kill-after=10s 1800s/);
assert.match(source, /"\$EXECUTABLE" >\/dev\/null 2>&1/);
assert.match(source, /AER_LAUNCHER_RESULT=%s/);
assert.match(source, /require_process_count 0/);
assert.match(source, /require_process_count 1/);
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
