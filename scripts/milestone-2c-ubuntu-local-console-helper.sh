#!/usr/bin/env bash

set -euo pipefail
umask 077

# Suppress uncontrolled diagnostics from dependencies and the shell. Controlled
# operator prompts and stop labels use the preserved original diagnostic stream.
exec 3>&2
exec 2>/dev/null

readonly EXPECTED_SOURCE="5f54ec00cbfd884a0ffbce956d586d8ac8f5a199"
readonly EXPECTED_TREE="7aa645875fc4dcd1b28e91eb209a073990bd1877"
readonly EXPECTED_CARGO_LOCK="8769cc560c5ed3c6f00b10a135bc6125a6f9f7e655c6520cf73f95928d9d9082"
readonly EXPECTED_PACKAGE_LOCK="013e1fcd2917509cd098dfecacc83bd25c4aea1e01633f051d0f048bdf7d8dad"
readonly EXPECTED_EXECUTABLE_BYTES="16022208"
readonly EXPECTED_EXECUTABLE_SHA="482a6e302469d9340d1b95337a0f9aa864367617421125fa5ad380f13f94599f"
readonly EXPECTED_PACKAGE_BYTES="4896812"
readonly EXPECTED_PACKAGE_SHA="eae771fcee89f31b5ecfb5154c9fa71ff2ce94634228ca0599b7dcdcae6b438e"
readonly CLIPBOARD_SENTINEL="AI_ENGINE_ROOM_2C_CLIPBOARD_SENTINEL"
readonly EXISTING_SENTINEL="AI_ENGINE_ROOM_2C_EXISTING_SENTINEL"
readonly EXPECTED_CLIPBOARD_SENTINEL_BYTES="36"
readonly EXPECTED_CLIPBOARD_SENTINEL_SHA="8a5d29b46cc6ef572248ecba3e268b73deb4eff3ae299752a19cbe6f5b538ea2"
readonly EXPECTED_EXISTING_SENTINEL_BYTES="35"
readonly EXPECTED_EXISTING_SENTINEL_SHA="aebeb69e9c773c0c0e719bf88c36e2a035c795890ad2cc8450937ea64c0ded03"

# Construct retained local paths so no machine-specific path is retained in
# this public helper. The approved host preflight establishes these locations.
readonly TEMP_ROOT="$(printf '/%s' 'tmp')"
readonly CANDIDATE_ROOT="${TEMP_ROOT}/aer-2c-ubuntu-candidate"
readonly SOURCE_DIR="${CANDIDATE_ROOT}/src"
readonly EXECUTABLE="${CANDIDATE_ROOT}/target/release/aiengineroom"
readonly PACKAGE="${CANDIDATE_ROOT}/target/release/bundle/deb/AI Engine Room_0.1.0_amd64.deb"
readonly RUN_DIR="${TEMP_ROOT}/aer-2c-ubuntu-local-console-assisted"
readonly BLOCKED_DIR="${RUN_DIR}/blocked"

HELPER_MODE="unselected"
CLIPBOARD_CHANGED="false"
CLEANUP_COMPLETE="false"
CLEANUP_ARMED="false"
PROCESS_CLOSURE_ALLOWED="false"
FAILURE_REPORTED="false"

fail() {
  FAILURE_REPORTED="true"
  printf 'STOP: %s\n' "$1" >&3
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required preflight command is unavailable"
}

require_local_console() {
  local expected_runtime expected_bus expected_xauthority
  expected_runtime="$(printf '/%s/%s/%s' 'run' 'user' "$(id -u)")"
  expected_bus="unix:path=${expected_runtime}/bus"
  expected_xauthority="${expected_runtime}/gdm/Xauthority"
  [[ "$(id -u)" != "0" ]] || fail "ordinary-user context is required"
  [[ -t 0 && -t 1 ]] || fail "an interactive local terminal is required"
  [[ "${DISPLAY:-}" == ":0" ]] || fail "the reviewed X11 display is required"
  [[ "${XDG_SESSION_TYPE:-}" == "x11" ]] || fail "the reviewed X11 session type is required"
  [[ "${XDG_RUNTIME_DIR:-}" == "$expected_runtime" ]] || fail "the reviewed user runtime directory is required"
  [[ "${DBUS_SESSION_BUS_ADDRESS:-}" == "$expected_bus" ]] || fail "the reviewed graphical session bus is required"
  [[ "${XAUTHORITY:-}" == "$expected_xauthority" ]] || fail "the reviewed X11 authority is required"
  [[ -z "${SSH_CLIENT:-}" && -z "${SSH_CONNECTION:-}" && -z "${SSH_TTY:-}" ]] || \
    fail "SSH and remote shells are prohibited"
}

file_identity() {
  local path="$1"
  local expected_bytes="$2"
  local expected_sha="$3"
  [[ -f "$path" && ! -L "$path" ]] || fail "candidate identity is unavailable"
  [[ "$(stat -c '%s' -- "$path")" == "$expected_bytes" ]] || fail "candidate byte length changed"
  [[ "$(sha256sum -- "$path" | cut -d ' ' -f 1)" == "$expected_sha" ]] || fail "candidate hash changed"
}

require_candidate_identity() {
  [[ -d "$SOURCE_DIR" && ! -L "$SOURCE_DIR" ]] || fail "candidate source is unavailable"
  [[ "$(git -C "$SOURCE_DIR" rev-parse HEAD)" == "$EXPECTED_SOURCE" ]] || fail "candidate source changed"
  [[ "$(git -C "$SOURCE_DIR" rev-parse 'HEAD^{tree}')" == "$EXPECTED_TREE" ]] || fail "candidate tree changed"
  [[ -z "$(git -C "$SOURCE_DIR" status --porcelain=v1 --untracked-files=all)" ]] || fail "candidate source is not clean"
  [[ "$(sha256sum -- "$SOURCE_DIR/Cargo.lock" | cut -d ' ' -f 1)" == "$EXPECTED_CARGO_LOCK" ]] || fail "Cargo lockfile changed"
  [[ "$(sha256sum -- "$SOURCE_DIR/package-lock.json" | cut -d ' ' -f 1)" == "$EXPECTED_PACKAGE_LOCK" ]] || fail "package lockfile changed"
  file_identity "$EXECUTABLE" "$EXPECTED_EXECUTABLE_BYTES" "$EXPECTED_EXECUTABLE_SHA"
  file_identity "$PACKAGE" "$EXPECTED_PACKAGE_BYTES" "$EXPECTED_PACKAGE_SHA"
}

require_no_installed_copy() {
  if dpkg-query -W -f='${Status}' ai-engine-room 2>/dev/null | grep -qx 'install ok installed'; then
    fail "an installed copy is present"
  fi
}

process_count() {
  { pgrep -u "$(id -u)" -x aiengineroom 2>/dev/null || true; } | wc -l | tr -d ' '
}

require_process_count() {
  local expected="$1"
  [[ "$(process_count)" == "$expected" ]] || fail "candidate process count is unexpected"
}

require_safe_run_dir() {
  [[ -d "$RUN_DIR" && ! -L "$RUN_DIR" ]] || fail "disposable directory is unavailable"
  [[ "$(realpath -e -- "$RUN_DIR")" == "${TEMP_ROOT}/aer-2c-ubuntu-local-console-assisted" ]] || fail "disposable directory resolved unexpectedly"
  [[ "$(stat -c '%u' -- "$RUN_DIR")" == "$(id -u)" ]] || fail "disposable directory ownership changed"
  [[ "$(stat -c '%a' -- "$RUN_DIR")" == "700" ]] || fail "disposable directory mode changed"
}

require_top_level_entries() {
  local allowed="$1"
  local entry path permitted matched dotglob_was_set nullglob_was_set
  local -a allowed_entries
  local -a paths
  IFS=',' read -r -a allowed_entries <<<"$allowed"
  shopt -q dotglob && dotglob_was_set="true" || dotglob_was_set="false"
  shopt -q nullglob && nullglob_was_set="true" || nullglob_was_set="false"
  shopt -s dotglob nullglob
  paths=("$RUN_DIR"/*)
  [[ "$dotglob_was_set" == "true" ]] || shopt -u dotglob
  [[ "$nullglob_was_set" == "true" ]] || shopt -u nullglob
  for path in "${paths[@]}"; do
    entry="${path##*/}"
    matched="false"
    for permitted in "${allowed_entries[@]}"; do
      [[ "$entry" == "$permitted" ]] && matched="true"
    done
    [[ "$matched" == "true" ]] || fail "an unexpected disposable entry exists"
  done
}

directory_is_empty() {
  local path="$1"
  local dotglob_was_set nullglob_was_set
  local -a entries
  shopt -q dotglob && dotglob_was_set="true" || dotglob_was_set="false"
  shopt -q nullglob && nullglob_was_set="true" || nullglob_was_set="false"
  shopt -s dotglob nullglob
  entries=("$path"/*)
  [[ "$dotglob_was_set" == "true" ]] || shopt -u dotglob
  [[ "$nullglob_was_set" == "true" ]] || shopt -u nullglob
  [[ "${#entries[@]}" == "0" ]]
}

require_empty_directory() {
  directory_is_empty "$1" || fail "a controlled directory is not empty"
}

bytes_and_sha() {
  python3 -c 'import hashlib, sys
b = sys.stdin.buffer.read()
b.decode("utf-8")
print(len(b), hashlib.sha256(b).hexdigest())'
}

clipboard_identity() {
  xclip -selection clipboard -out 2>/dev/null | bytes_and_sha
}

file_identity_values() {
  python3 - "$1" <<'PY'
import hashlib
import pathlib
import sys

value = pathlib.Path(sys.argv[1]).read_bytes()
value.decode("utf-8")
if value.startswith(b"\xef\xbb\xbf"):
    raise SystemExit("BOM is prohibited")
print(len(value), hashlib.sha256(value).hexdigest())
PY
}

confirm() {
  local prompt="$1"
  local answer
  printf '\n%s\nType YES only when this is exact; otherwise type STOP: ' "$prompt"
  IFS= read -r answer
  [[ "$answer" == "YES" ]] || fail "operator did not confirm the checkpoint"
}

choose() {
  local prompt="$1"
  shift
  local answer option
  printf '\n%s\nAllowed responses: %s STOP\nResponse: ' "$prompt" "$*" >&3
  IFS= read -r answer
  [[ "$answer" != "STOP" ]] || fail "operator stopped at the checkpoint"
  for option in "$@"; do
    [[ "$answer" == "$option" ]] && { printf '%s' "$answer"; return 0; }
  done
  fail "operator response was not an allowed result"
}

remove_regular_file_if_present() {
  local path="$1"
  if [[ -e "$path" || -L "$path" ]]; then
    [[ -f "$path" && ! -L "$path" ]] || return 1
    rm -- "$path" || return 1
  fi
}

controlled_cleanup() {
  local cleanup_ok="true"
  local current_count="0"

  if [[ "$CLIPBOARD_CHANGED" == "true" ]]; then
    printf '' | xclip -selection clipboard -in >/dev/null 2>&1 || cleanup_ok="false"
    CLIPBOARD_CHANGED="false"
  fi

  current_count="$(process_count)"
  if [[ "$current_count" != "0" ]]; then
    if [[ "$PROCESS_CLOSURE_ALLOWED" == "true" ]]; then
      printf '%s\n' \
        'STOP_ACTION=close_application_normally' \
        'Close AI Engine Room using its window close control, then type CLOSED.' >&3
      local answer=""
      IFS= read -r answer || true
      [[ "$answer" == "CLOSED" && "$(process_count)" == "0" ]] || cleanup_ok="false"
    else
      cleanup_ok="false"
    fi
  fi

  if [[ -e "$RUN_DIR" || -L "$RUN_DIR" ]]; then
    if [[ ! -d "$RUN_DIR" || -L "$RUN_DIR" ]] || \
      [[ "$(realpath -e -- "$RUN_DIR")" != "${TEMP_ROOT}/aer-2c-ubuntu-local-console-assisted" ]] || \
      [[ "$(stat -c '%u' -- "$RUN_DIR")" != "$(id -u)" ]]; then
      cleanup_ok="false"
    elif [[ "$(process_count)" == "0" ]]; then
      if [[ -e "$BLOCKED_DIR" || -L "$BLOCKED_DIR" ]]; then
        if [[ -d "$BLOCKED_DIR" && ! -L "$BLOCKED_DIR" ]]; then
          chmod 700 "$BLOCKED_DIR" || cleanup_ok="false"
          remove_regular_file_if_present "$BLOCKED_DIR/inaccessible.txt" || cleanup_ok="false"
          directory_is_empty "$BLOCKED_DIR" || cleanup_ok="false"
          [[ "$cleanup_ok" == "true" ]] && rmdir -- "$BLOCKED_DIR" || cleanup_ok="false"
        else
          cleanup_ok="false"
        fi
      fi
      remove_regular_file_if_present "$RUN_DIR/saved.txt" || cleanup_ok="false"
      remove_regular_file_if_present "$RUN_DIR/existing.txt" || cleanup_ok="false"
      remove_regular_file_if_present "$RUN_DIR/stale.txt" || cleanup_ok="false"
      directory_is_empty "$RUN_DIR" || cleanup_ok="false"
      [[ "$cleanup_ok" == "true" ]] && rmdir -- "$RUN_DIR" || cleanup_ok="false"
    else
      cleanup_ok="false"
    fi
  fi

  if [[ "$cleanup_ok" == "true" && ! -e "$RUN_DIR" && ! -L "$RUN_DIR" ]]; then
    CLEANUP_COMPLETE="true"
    CLEANUP_ARMED="false"
    printf 'FAILURE_CLEANUP=confirmed\n' >&3
  else
    printf 'FAILURE_CLEANUP=unconfirmed\n' >&3
  fi
}

on_exit() {
  local status="$1"
  trap - EXIT
  set +e
  [[ "$status" != "0" ]] || return 0
  [[ "$FAILURE_REPORTED" == "true" ]] || printf 'STOP: helper operation failed\n' >&3
  if [[ "$CLEANUP_COMPLETE" == "true" ]]; then
    return "$status"
  elif [[ "$CLEANUP_ARMED" == "true" ]]; then
    controlled_cleanup
  else
    printf 'FAILURE_CLEANUP=not_armed\n' >&3
  fi
  return "$status"
}

trap 'on_exit $?' EXIT

preflight_commands() {
  local command
  for command in git stat sha256sum cut dpkg-query grep pgrep wc tr realpath python3 xclip timeout chmod mkdir rmdir rm sleep; do
    require_command "$command"
  done
}

terminal_a() {
  HELPER_MODE="terminal-a"
  preflight_commands
  require_local_console
  require_candidate_identity
  require_no_installed_copy
  require_process_count 0
  [[ ! -e "$RUN_DIR" && ! -L "$RUN_DIR" ]] || fail "the fixed disposable directory already exists"
  [[ "$(realpath -e -- "$TEMP_ROOT")" == "$TEMP_ROOT" ]] || fail "temporary root resolved unexpectedly"

  printf '%s\n' \
    'Terminal A preflight passed.' \
    'Remain physically present for the uninterrupted run.' \
    'Start Terminal B only after the application window appears.' \
    'Type RUN only if a separate exact run authorization has been issued.'
  local answer
  IFS= read -r answer
  [[ "$answer" == "RUN" ]] || fail "run authorization was not acknowledged"

  mkdir --mode=700 -- "$RUN_DIR"
  CLEANUP_ARMED="true"
  require_safe_run_dir
  require_empty_directory "$RUN_DIR"
  cd "$RUN_DIR"

  set +e
  timeout --signal=TERM --kill-after=10s 1800s "$EXECUTABLE" >/dev/null 2>&1
  local launcher_result=$?
  set -e
  printf 'AER_LAUNCHER_RESULT=%s\n' "$launcher_result"
  local controller_state
  controller_state="$(choose 'The application process has ended. If Terminal B is currently running the controller, answer CONTROLLER and let it classify the launcher result and clean up. If Terminal B never started or is no longer running, answer NO_CONTROLLER so Terminal A performs failure cleanup.' CONTROLLER NO_CONTROLLER)"
  if [[ "$controller_state" == "CONTROLLER" ]]; then
    CLEANUP_ARMED="false"
    printf 'TERMINAL_A_CLEANUP_HANDOFF=terminal-b\n'
    return 0
  fi
  controlled_cleanup
  fail "the application ended without an active Terminal B controller"
}

terminal_b() {
  HELPER_MODE="terminal-b"
  preflight_commands
  require_local_console
  require_candidate_identity
  require_no_installed_copy
  require_safe_run_dir
  require_empty_directory "$RUN_DIR"
  require_process_count 1
  CLEANUP_ARMED="true"
  PROCESS_CLOSURE_ALLOWED="true"

  printf '%s\n' \
    'Terminal B controller is ready.' \
    'Use only the AI Engine Room Report workspace and this terminal.' \
    'Do not refresh, copy, save, close, or leave the console until prompted.'

  confirm 'In the application, open Report without refreshing. Activate Save report… by keyboard. Confirm dialog title "Save AI Engine Room report", suggested filename "ai-engine-room-report.txt", and the sole filter "Plain text". Press Escape; do not save. The application must then show "Save cancelled. No report file was created." as polite status, with focus returned to Save report….'
  require_process_count 1
  require_empty_directory "$RUN_DIR"

  confirm 'Activate Copy report exactly once. The application must show "Report copied to the system clipboard." as polite status. Do not copy again.'
  CLIPBOARD_CHANGED="true"
  require_process_count 1
  local preview_bytes preview_sha
  read -r preview_bytes preview_sha < <(clipboard_identity)
  [[ "$preview_bytes" -le 1048576 ]] || fail "copied preview exceeds the contracted bound"
  printf '%s' "$CLIPBOARD_SENTINEL" | xclip -selection clipboard -in
  local sentinel_bytes sentinel_sha
  read -r sentinel_bytes sentinel_sha < <(clipboard_identity)
  [[ "$sentinel_bytes" == "$EXPECTED_CLIPBOARD_SENTINEL_BYTES" && "$sentinel_sha" == "$EXPECTED_CLIPBOARD_SENTINEL_SHA" ]] || fail "clipboard sentinel verification failed"
  printf 'COPY_CHECK=pass PREVIEW_BYTES=%s PREVIEW_SHA256=%s\n' "$preview_bytes" "$preview_sha"

  confirm 'Activate Save report… by keyboard. The dialog must retain title "Save AI Engine Room report", suggested filename "ai-engine-room-report.txt", and sole filter "Plain text". Save as "saved.txt" in the already-open disposable location. The button may show "Saving report…" while pending. The application must then show "Report saved as a plain-text file." as polite status, with focus returned.'
  require_process_count 1
  [[ -f "$RUN_DIR/saved.txt" && ! -L "$RUN_DIR/saved.txt" ]] || fail "saved report is missing"
  local saved_bytes saved_sha
  read -r saved_bytes saved_sha < <(file_identity_values "$RUN_DIR/saved.txt")
  [[ "$saved_bytes" == "$preview_bytes" && "$saved_sha" == "$preview_sha" ]] || fail "saved report differs from the copied preview"
  require_top_level_entries 'saved.txt'
  read -r sentinel_bytes sentinel_sha < <(clipboard_identity)
  [[ "$sentinel_bytes" == "$EXPECTED_CLIPBOARD_SENTINEL_BYTES" && "$sentinel_sha" == "$EXPECTED_CLIPBOARD_SENTINEL_SHA" ]] || fail "saving changed the clipboard sentinel"
  printf 'NEW_FILE_CHECK=pass SAVED_BYTES=%s SAVED_SHA256=%s\n' "$saved_bytes" "$saved_sha"

  require_process_count 1
  printf '%s' "$EXISTING_SENTINEL" >"$RUN_DIR/existing.txt"
  local existing_bytes existing_sha
  read -r existing_bytes existing_sha < <(file_identity_values "$RUN_DIR/existing.txt")
  [[ "$existing_bytes" == "$EXPECTED_EXISTING_SENTINEL_BYTES" && "$existing_sha" == "$EXPECTED_EXISTING_SENTINEL_SHA" ]] || fail "existing-file fixture identity failed"
  require_top_level_entries 'saved.txt,existing.txt'
  confirm 'Activate Save report… and select the existing "existing.txt" fixture. The native chooser must show "A file named \"existing.txt\" already exists. Do you want to replace it?" Select Replace only for this controlled sentinel fixture. The application must then show "That file already exists. AI Engine Room did not replace it. Choose a different name." with alert semantics. Copy report must remain available.'
  require_process_count 1
  [[ -f "$RUN_DIR/existing.txt" && ! -L "$RUN_DIR/existing.txt" ]] || fail "existing destination type changed"
  local existing_after_bytes existing_after_sha
  read -r existing_after_bytes existing_after_sha < <(file_identity_values "$RUN_DIR/existing.txt")
  [[ "$existing_after_bytes" == "$existing_bytes" && "$existing_after_sha" == "$existing_sha" ]] || fail "existing destination was changed"
  require_top_level_entries 'saved.txt,existing.txt'
  read -r sentinel_bytes sentinel_sha < <(clipboard_identity)
  [[ "$sentinel_bytes" == "$EXPECTED_CLIPBOARD_SENTINEL_BYTES" && "$sentinel_sha" == "$EXPECTED_CLIPBOARD_SENTINEL_SHA" ]] || fail "no-clobber handling changed the clipboard sentinel"
  printf 'NO_CLOBBER_CHECK=pass EXISTING_BYTES=%s EXISTING_SHA256=%s\n' "$existing_after_bytes" "$existing_after_sha"

  require_process_count 1
  local stale_result
  stale_result="$(choose 'Open Save report… once. While the native dialog is open, attempt the reviewed UI-only gesture to return to the application and activate Refresh before accepting "stale.txt". If the parented dialog prevents this, cancel it and answer UNREACHABLE. If Refresh succeeds, select "stale.txt" and require "The report changed before saving. Review it and try again." with alert semantics, then answer CHANGED.' UNREACHABLE CHANGED)"
  require_process_count 1
  [[ ! -e "$RUN_DIR/stale.txt" && ! -L "$RUN_DIR/stale.txt" ]] || fail "stale-preview attempt created a destination"
  require_top_level_entries 'saved.txt,existing.txt'
  printf 'STALE_PREVIEW_CHECK=%s\n' "${stale_result,,}"

  require_process_count 1
  mkdir --mode=700 -- "$BLOCKED_DIR"
  chmod 500 "$BLOCKED_DIR"
  [[ ! -w "$BLOCKED_DIR" ]] || fail "inaccessible fixture remained writable"
  require_empty_directory "$BLOCKED_DIR"
  local inaccessible_result
  inaccessible_result="$(choose 'Open Save report… and select "inaccessible.txt" inside the visible blocked fixture. No raw error or elevation request may appear. Require either "Saving is not available for that location. You can still copy the report." or "Could not save the report. No completed report file was created." with alert semantics. Copy report must remain available.' UNAVAILABLE FAILED)"
  require_process_count 1
  [[ ! -e "$BLOCKED_DIR/inaccessible.txt" && ! -L "$BLOCKED_DIR/inaccessible.txt" ]] || fail "inaccessible attempt created a destination"
  require_empty_directory "$BLOCKED_DIR"
  require_top_level_entries 'saved.txt,existing.txt,blocked'
  printf 'INACCESSIBLE_LOCATION_CHECK=%s\n' "${inaccessible_result,,}"

  confirm 'At normal and enlarged text, confirm the Report workspace remains usable, keyboard activation and visible focus work, Copy report remains available, and no raw error, path, overlap, or clipped required wording appears.'
  require_process_count 1
  printf 'UI_ACCESSIBILITY_CHECK=pass\n'

  printf '' | xclip -selection clipboard -in
  CLIPBOARD_CHANGED="false"
  require_process_count 1
  confirm 'The fixed clipboard sentinel has been cleared. Close AI Engine Room normally using its window close control. Do not stop it from Terminal A, Terminal B, SSH, or a process manager. Remain at the console.'
  local count
  for _ in {1..50}; do
    count="$(process_count)"
    [[ "$count" == "0" ]] && break
    sleep 0.2
  done
  require_process_count 0

  local launcher_result
  printf 'Read AER_LAUNCHER_RESULT from Terminal A and type only its number here: '
  IFS= read -r launcher_result
  [[ "$launcher_result" =~ ^(0|[1-9][0-9]{0,2})$ ]] || fail "launcher result was not a valid process result"
  [[ "$launcher_result" -le 255 ]] || fail "launcher result exceeded the process-result range"

  chmod 700 "$BLOCKED_DIR"
  require_empty_directory "$BLOCKED_DIR"
  rmdir -- "$BLOCKED_DIR"
  rm -- "$RUN_DIR/saved.txt" "$RUN_DIR/existing.txt"
  require_empty_directory "$RUN_DIR"
  rmdir -- "$RUN_DIR"
  [[ ! -e "$RUN_DIR" && ! -L "$RUN_DIR" ]] || fail "disposable cleanup was not confirmed"
  require_candidate_identity
  require_no_installed_copy
  require_process_count 0
  CLEANUP_COMPLETE="true"
  CLEANUP_ARMED="false"
  if [[ "$launcher_result" != "0" ]]; then
    printf 'TERMINATION_CHECK=fail LAUNCHER_RESULT=%s\nRESIDUE_CHECK=pass\nCLEANUP_CHECK=pass\n' "$launcher_result"
    fail "launcher result was not zero"
  fi
  printf 'TERMINATION_CHECK=pass LAUNCHER_RESULT=0\nRESIDUE_CHECK=pass\nCLEANUP_CHECK=pass\n'
}

usage() {
  printf 'Usage: %s terminal-a|terminal-b\n' "${0##*/}" >&3
  exit 2
}

[[ "$#" == "1" ]] || usage
case "$1" in
  terminal-a) terminal_a ;;
  terminal-b) terminal_b ;;
  *) usage ;;
esac
