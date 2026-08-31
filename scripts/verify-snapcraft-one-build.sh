#!/usr/bin/env bash
set -Eeuo pipefail

readonly expected_repository="gregweir/ai-engine-room"
readonly expected_branch="codex/snapcraft-one-build-execution"
readonly expected_definition_blob="ff6d085085ae7dd693e2834541c5b1cb2c3c965c"
readonly expected_launcher_blob="bd5bc3af83ca13ae29d53e4331255296c1526611"

result_class="infrastructure_stop"
phase="preflight"

finish() {
  local status=$?
  printf 'AER_SNAPCRAFT_RESULT_CLASS=%s\n' "$result_class"
  printf 'AER_SNAPCRAFT_RESULT_PHASE=%s\n' "$phase"
  printf 'AER_SNAPCRAFT_EXIT_STATUS=%s\n' "$status"
}
trap finish EXIT

fail() {
  printf 'STOP: %s\n' "$1" >&2
  exit "${2:-1}"
}

[[ "${AER_SNAPCRAFT_ONE_BUILD_APPROVED:-}" == "true" ]] ||
  fail "The one-run approval variable is absent."
[[ "${GITHUB_ACTIONS:-}" == "true" ]] || fail "This procedure is GitHub Actions-only."
[[ "${RUNNER_ENVIRONMENT:-}" == "github-hosted" ]] ||
  fail "The runner is not GitHub-hosted."
[[ "${RUNNER_OS:-}" == "Linux" && "${RUNNER_ARCH:-}" == "X64" ]] ||
  fail "The runner is not Linux x64."
[[ "${GITHUB_REPOSITORY:-}" == "$expected_repository" ]] ||
  fail "The repository identity is not approved."
[[ "${GITHUB_HEAD_REF:-}" == "$expected_branch" ]] ||
  fail "The pull-request branch is not approved."
[[ -n "${RUNNER_TEMP:-}" ]] || fail "RUNNER_TEMP is unavailable."

for store_variable in \
  SNAPCRAFT_STORE_AUTH \
  SNAPCRAFT_STORE_CREDENTIALS \
  SNAPCRAFT_STORE_CREDENTIALS_FILE; do
  [[ -z "${!store_variable:-}" ]] ||
    fail "A Snap Store credential variable is present: $store_variable."
done

[[ "$(git hash-object -- snap/snapcraft.yaml)" == "$expected_definition_blob" ]] ||
  fail "The Snapcraft definition blob is not approved."
[[ "$(git hash-object -- snap/gui/ai-engine-room.desktop)" == "$expected_launcher_blob" ]] ||
  fail "The desktop launcher blob is not approved."

for generated_path in .snapcraft parts prime stage; do
  [[ ! -e "$generated_path" ]] || fail "Generated path already exists: $generated_path."
done
shopt -s nullglob
initial_artifacts=(*.snap)
shopt -u nullglob
(( ${#initial_artifacts[@]} == 0 )) || fail "A Snap artifact already exists."

phase="runner_identity"
printf 'runner_environment=%s\n' "$RUNNER_ENVIRONMENT"
printf 'runner_os=%s\n' "$RUNNER_OS"
printf 'runner_arch=%s\n' "$RUNNER_ARCH"
printf 'image_os=%s\n' "${ImageOS:-unavailable}"
printf 'image_version=%s\n' "${ImageVersion:-unavailable}"
uname -a
cat /etc/os-release
printf 'dpkg_architecture=%s\n' "$(dpkg --print-architecture)"
[[ "$(. /etc/os-release && printf '%s' "$VERSION_ID")" == "24.04" ]] ||
  fail "The runner is not Ubuntu 24.04."
[[ "$(dpkg --print-architecture)" == "amd64" ]] ||
  fail "The package architecture is not amd64."
command -v ruby >/dev/null || fail "Ruby is unavailable for independent YAML parsing."
command -v unsquashfs >/dev/null || fail "unsquashfs is unavailable for artifact inspection."

phase="tool_installation"
printf '%s\n' '--- advertised Snapcraft channels and revisions ---'
snap info snapcraft
sudo snap install snapcraft --classic --channel=latest/stable
printf '%s\n' '--- installed tool versions before expansion ---'
snapcraft --version
snap version
snap list --all
printf 'host_node=%s\n' "$(node --version)"
printf 'host_npm=%s\n' "$(npm --version)"
printf 'host_rustc=%s\n' "$(rustc --version)"
printf 'host_cargo=%s\n' "$(cargo --version)"

phase="extension_expansion"
result_class="expansion_stop"
readonly expansion_file="$RUNNER_TEMP/aer-snapcraft-expanded.yaml"
readonly expansion_report="$RUNNER_TEMP/aer-snapcraft-expansion-report.json"
readonly expansion_block="$RUNNER_TEMP/aer-snapcraft-expansion-block.txt"
snapcraft expand-extensions | tee "$expansion_file"

ruby -ryaml -rjson - "$expansion_file" "$expansion_report" "$expansion_block" <<'RUBY'
expanded_path, report_path, block_path = ARGV
data = YAML.safe_load(File.read(expanded_path), aliases: true)
app = data.fetch("apps").fetch("ai-engine-room")
parts = data.fetch("parts")
snap_plugs = data.fetch("plugs", {})
app_plugs = Array(app["plugs"]).map(&:to_s).sort
sources = parts.values.filter_map { |part| part["source"] }.map(&:to_s).uniq.sort

report = {
  identity: data.slice("name", "base", "grade", "confinement", "platforms"),
  app: app.slice("command", "common-id", "desktop", "plugs", "command-chain", "environment"),
  snap_wide_plugs: snap_plugs,
  parts: parts.transform_values do |part|
    part.slice("plugin", "source", "build-snaps", "build-packages", "stage-packages")
  end,
  layouts: data.fetch("layout", {}),
  sources: sources,
}
File.write(report_path, JSON.pretty_generate(report))

problems = []
problems << "name changed" unless data["name"] == "ai-engine-room"
problems << "base changed" unless data["base"] == "core24"
problems << "grade changed" unless data["grade"] == "devel"
problems << "confinement changed" unless data["confinement"] == "strict"
allowed_platforms = [
  { "amd64" => nil },
  { "amd64" => { "build-on" => ["amd64"], "build-for" => ["amd64"] } },
]
problems << "platform plan changed" unless allowed_platforms.include?(data["platforms"])
problems << "command changed" unless app["command"] == "bin/aiengineroom"
problems << "common ID changed" unless app["common-id"] == "com.tartanleaf.aiengineroom"
allowed_desktop_values = [nil, "meta/gui/ai-engine-room.desktop"]
problems << "desktop launcher changed" unless allowed_desktop_values.include?(app["desktop"])
problems << "source set changed: #{sources.join(', ')}" unless sources == ["."]

allowed_app_plugs = %w[desktop desktop-legacy gsettings network opengl wayland x11]
blocked_app_plugs = %w[calendar-service mount-observe]
unknown_app_plugs = app_plugs - allowed_app_plugs - blocked_app_plugs
problems << "unknown app plugs: #{unknown_app_plugs.join(', ')}" unless unknown_app_plugs.empty?
present_blocked_plugs = app_plugs & blocked_app_plugs
unless present_blocked_plugs.empty?
  problems << "unjustified app plugs: #{present_blocked_plugs.join(', ')}"
end

allowed_snap_plugs = %w[desktop gnome-46-2404 gpu-2404 gtk-3-themes icon-themes sound-themes]
unknown_snap_plugs = snap_plugs.keys.map(&:to_s).sort - allowed_snap_plugs
problems << "unknown snap-wide plugs: #{unknown_snap_plugs.join(', ')}" unless unknown_snap_plugs.empty?

File.write(block_path, problems.join("\n") + "\n") unless problems.empty?
RUBY

cat "$expansion_report"
printf 'expanded_sha256=%s\n' "$(sha256sum "$expansion_file" | cut -d' ' -f1)"
if [[ -s "$expansion_block" ]]; then
  printf '%s\n' '--- expansion stop reasons ---' >&2
  cat "$expansion_block" >&2
  fail "The expanded definition exceeds the reviewed boundary." 42
fi

phase="single_build"
result_class="build_stop"
printf '%s\n' 'Expansion gate passed; beginning the single permitted build attempt.'
sudo env \
  DEBIAN_FRONTEND=noninteractive \
  SNAPCRAFT_BUILD_INFO=1 \
  PATH="$PATH" \
  snapcraft --destructive-mode --build-for=amd64

phase="artifact_inspection"
shopt -s nullglob
artifacts=(*.snap)
shopt -u nullglob
(( ${#artifacts[@]} == 1 )) ||
  fail "Expected exactly one Snap artifact, found ${#artifacts[@]}."
readonly artifact="${artifacts[0]}"
[[ "$artifact" == "ai-engine-room_0.1.0_amd64.snap" ]] ||
  fail "Unexpected artifact filename: $artifact."
printf 'artifact_filename=%s\n' "$artifact"
printf 'artifact_bytes=%s\n' "$(stat -c '%s' "$artifact")"
sha256sum "$artifact"

readonly unpack_dir="$RUNNER_TEMP/aer-snap-unpacked"
unsquashfs -no-progress -d "$unpack_dir" "$artifact"
printf '%s\n' '--- packaged metadata ---'
cat "$unpack_dir/meta/snap.yaml"
printf '%s\n' '--- packaged desktop launcher ---'
cat "$unpack_dir/meta/gui/ai-engine-room.desktop"
printf '%s\n' '--- complete packaged file list ---'
find "$unpack_dir" -printf '%P\t%y\t%s\n' | LC_ALL=C sort

readonly packaged_binary="$unpack_dir/bin/aiengineroom"
[[ -x "$packaged_binary" ]] || fail "The packaged command is absent or not executable."
file "$packaged_binary"
printf '%s\n' '--- packaged command dynamic-library resolution ---'
ldd "$packaged_binary" | tee "$RUNNER_TEMP/aer-snap-ldd.txt"
! grep -F 'not found' "$RUNNER_TEMP/aer-snap-ldd.txt" ||
  fail "The packaged command has unresolved dynamic libraries."

for payload in LICENSE NOTICE THIRD-PARTY-LICENSES.txt THIRD-PARTY-SOURCES.txt; do
  payload_path="$unpack_dir/licenses/$payload"
  [[ -f "$payload_path" ]] || fail "Missing packaged licence payload: $payload."
  sha256sum "$payload_path"
done

if [[ -f "$unpack_dir/snap/manifest.yaml" ]]; then
  printf '%s\n' '--- Snapcraft build manifest ---'
  cat "$unpack_dir/snap/manifest.yaml"
else
  fail "The Snapcraft build manifest is absent."
fi

printf '%s\n' '--- installed snaps after build ---'
snap list --all
printf '%s\n' '--- declared Ubuntu build-package versions ---'
dpkg-query -W -f='${Package}\t${Version}\n' \
  build-essential curl desktop-file-utils file libappindicator3-dev \
  librsvg2-dev libssl-dev libwebkit2gtk-4.1-dev patchelf wget
printf '%s\n' '--- locked npm dependency versions ---'
node -e 'const p=require("./package-lock.json"); for (const [path,item] of Object.entries(p.packages??{}).sort()) if (item.version) console.log(`${path || "."}\t${item.version}`)'
printf '%s\n' '--- locked Cargo dependency versions and sources ---'
grep -E '^(name|version|source) = ' Cargo.lock

phase="complete"
result_class="build_pass"
printf '%s\n' 'The local artifact passed the bounded inspection. It was not installed or uploaded.'
