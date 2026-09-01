import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { extname } from "node:path";
import { TextDecoder } from "node:util";
import "./license-readiness.mjs";

const read = (path) => readFileSync(path, "utf8");
const json = (path) => JSON.parse(read(path));
const required = [
  "README.md",
  "LICENSE",
  "NOTICE",
  "THIRD-PARTY-LICENSES.txt",
  "THIRD-PARTY-SOURCES.txt",
  "CONTRIBUTING.md",
  "CODE_OF_CONDUCT.md",
  "SECURITY.md",
  "SUPPORT.md",
  "CHANGELOG.md",
  ".github/ISSUE_TEMPLATE/bug_report.md",
  ".github/ISSUE_TEMPLATE/feature_request.md",
  ".github/PULL_REQUEST_TEMPLATE.md",
  ".github/workflows/deterministic.yml",
  "docs/roadmap.md",
  "docs/design/milestone-1y-c-snapcraft-one-build-preparation-contract.md",
  "docs/design/milestone-1y-d-snap-disposition-contract.md",
  "docs/distribution/snapcraft-expansion-verification-record.md",
  "docs/distribution/snap-disposition.md",
  "docs/release/linux-pre-release-verification.md",
  "docs/release/linux-pre-release-build-evidence.md",
  "docs/release/lm-studio-live-verification.md",
  "docs/release/windows-pre-release-verification.md",
  "docs/release/windows-provider-verification.md",
  "docs/release/windows-available-memory-verification.md",
  "docs/release/llamacpp-passive-verification.md",
  "docs/release/unsigned-preview-release-contract.md",
  "docs/release/signpath-eligibility-assessment.md",
  "src-tauri/tauri.windows.conf.json",
  "packaging/debian/copyright",
  "snap/snapcraft.yaml",
  "snap/gui/ai-engine-room.desktop",
  "third-party/generated-manifest.json",
];
for (const path of required) assert.ok(existsSync(path), `missing ${path}`);

const excludedPrivatePaths = [
  ".codex",
  "AGENTS.md",
  "docs/handoffs",
  "docs/plans",
  "scripts/milestone-lib.mjs",
  "scripts/milestone-lib.test.mjs",
  "scripts/milestone-status.mjs",
  "scripts/verify-evidence.mjs",
  "scripts/verify-handoff.mjs",
];
for (const path of excludedPrivatePaths) {
  assert.equal(
    existsSync(path),
    false,
    `private coordination path must remain absent: ${path}`,
  );
}

// Opaque hashes represent developer-controlled internal machine identifiers. The
// plaintext values are intentionally absent from the public readiness policy.
const forbiddenMachineIdentifierHashes = new Set([
  "1855b3b0b6b3bd0be712f822f3e19fed372c06007b298efa3af86594ed6540fa",
  "97ea43a59c9a701de4648dd7418cd392cebfd54d9bd7f4dd8eeffb586a298fdd",
  "1cae4b6d48146ea65b85d5c85b2ca672a3c273814343e8572a0064f8a6c242ef",
  "1c553e8b70d1b71102e3991c454465fff46ab0d2200ee8702c51511853aea235",
  "a149f9ddae0b8cef766afae28b5580d995466726eed5e3aebd7f983125b79fc8",
  "c8e545c66e03b75442b1140caeb0e3315d5826724a99f0c640d0ee9afba087d8",
]);

// These opaque hashes allow only artificial paths used by deterministic
// privacy tests. Their plaintext values are not retained in the public policy.
const allowedSyntheticAbsolutePathHashes = new Set([
  "31af7cc95c1376bdb9bc6e5fa2937a0f4591025d82280375431fb55cb6017332",
  "1e1cd37141efac6dee6bb65fba445cca8b9c629ae9fa54eaf1f616f4125947ed",
  "9f05bff1ab1201a0eca99b041037251aab90619998f16d64681e1012f5a991b0",
  "55295dd6905efe3932fc1aca2b45ac2cf1cda997bc2cd77500fa2c4c1d8369e9",
  "62991e2c7c25ce1c2b982659ff9b82e0b93a7861974f5117d2e6a814b51166c7",
  "4cfd63af51ebcf5b29d38d6a3e6e7792fe6b4a07447a483fed1ddc6930773d9a",
  "25faa7c8c8eeb9c97e1afdef3908ca92b25a2f65ee9016203e07f8a35d991123",
  "c111727f910d743f8647ed4184cd582f1557f517dac8fde2a71cc2f837a0c941",
  "ae18ff958097c391b7d06600eb00d9828947792dd94b8d8779fde160b24d2bfa",
  "06eadeffd41685d973ff6974b9995e0233e403ee0f0fc9b83ad5b69457ccbf67",
]);
const consumedSyntheticAbsolutePathHashes = new Set();

const binaryMediaExtensions = new Set([
  ".7z",
  ".avif",
  ".bmp",
  ".exe",
  ".gif",
  ".gz",
  ".icns",
  ".ico",
  ".jpeg",
  ".jpg",
  ".otf",
  ".pdf",
  ".png",
  ".tar",
  ".tif",
  ".tiff",
  ".ttf",
  ".webp",
  ".woff",
  ".woff2",
  ".zip",
]);
const rootGeneratedDirectoryNames = new Set([
  ".git",
  "dist",
  "node_modules",
  "target",
]);
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const normalizeAbsolutePath = (value) =>
  value
    .replace(/\\+/g, "/")
    .replace(/\/{2,}/g, "/")
    .toLowerCase();
const absolutePathPatterns = [
  /(?<![A-Za-z0-9_-])[A-Za-z]:[\\/]+(?:[^\\/\s"'`<>|:]+[\\/]+)*[^\\/\s"'`<>|:]+/g,
  /(?<![A-Za-z]:)\\\\[A-Za-z0-9._-]+[\\/]+(?:[A-Za-z0-9._-]+[\\/]+)*[A-Za-z0-9._-]+/g,
  /(?<![A-Za-z0-9_:+/-])\/\/[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)+/g,
  /(?<![A-Za-z0-9._-])\/(?:home|root|Users|tmp|workspace|workspaces|var\/(?:tmp|folders)|private\/(?:tmp|var\/folders)|mnt\/[A-Za-z])(?:\/[A-Za-z0-9._*?-]+)+/g,
];

const assertRetainedTextIsPublicSafe = (
  path,
  source,
  machineIdentifierHashes = forbiddenMachineIdentifierHashes,
  syntheticAbsolutePathHashes = allowedSyntheticAbsolutePathHashes,
) => {
  const pathAndSource = `${path}\n${source}`;
  for (const pattern of absolutePathPatterns) {
    for (const match of pathAndSource.matchAll(pattern)) {
      const normalized = normalizeAbsolutePath(match[0]);
      const normalizedHash = sha256(normalized);
      assert.ok(
        syntheticAbsolutePathHashes.has(normalizedHash),
        `retained text contains a non-artificial absolute path: ${path} (${normalizedHash})`,
      );
      if (syntheticAbsolutePathHashes === allowedSyntheticAbsolutePathHashes) {
        consumedSyntheticAbsolutePathHashes.add(normalizedHash);
      }
    }
  }
  assert.doesNotMatch(
    source,
    /^\s*(?:[-*]\s*)?(?:machine class|device model|hostname|ssh alias|repository path)\s*:/im,
    `retained text contains an internal machine or repository label: ${path}`,
  );
  const tokens = pathAndSource.toLowerCase().match(/[a-z0-9]+/g) ?? [];
  for (let width = 1; width <= 3; width += 1) {
    for (let index = 0; index + width <= tokens.length; index += 1) {
      const candidate = tokens.slice(index, index + width).join(" ");
      assert.ok(
        !machineIdentifierHashes.has(sha256(candidate)),
        `retained text contains a forbidden internal machine identifier: ${path}`,
      );
    }
  }
};

const classifyRetainedEntry = (root, entry) => {
  const path = root === "." ? entry.name : `${root}/${entry.name}`;
  assert.ok(
    !entry.isSymbolicLink(),
    `retained path must not be a symlink: ${path}`,
  );
  return {
    path,
    skip:
      root === "." &&
      entry.isDirectory() &&
      rootGeneratedDirectoryNames.has(entry.name),
  };
};

const walkRetainedFiles = (root) =>
  readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const { path, skip } = classifyRetainedEntry(root, entry);
    if (skip) return [];
    return entry.isDirectory() ? walkRetainedFiles(path) : [path];
  });
const retainedPaths = walkRetainedFiles(".").sort();
const utf8Decoder = new TextDecoder("utf-8", { fatal: true });
for (const path of retainedPaths) {
  assertRetainedTextIsPublicSafe(path, "");
  if (binaryMediaExtensions.has(extname(path).toLowerCase())) continue;
  const bytes = readFileSync(path);
  assert.equal(bytes.includes(0), false, `unclassified binary file: ${path}`);
  let source;
  try {
    source = utf8Decoder.decode(bytes);
  } catch {
    assert.fail(`retained non-media file must be valid UTF-8 text: ${path}`);
  }
  assertRetainedTextIsPublicSafe(path, source);
}
assert.deepEqual(
  [...consumedSyntheticAbsolutePathHashes].sort(),
  [...allowedSyntheticAbsolutePathHashes].sort(),
  "every synthetic absolute-path allowlist entry must be consumed",
);

const syntheticPath = ["/", "home", "/", "alice", "/", "secret"].join("");
assert.doesNotThrow(() =>
  assertRetainedTextIsPublicSafe("fixture.rs", syntheticPath),
);
const negativeAbsolutePathFixtures = [
  ["C:", "\\", "home", "\\", "developer", "\\", "repo"].join(""),
  ["D:", "/", "workspaces", "/", "project"].join(""),
  ["\\", "\\", "server", "\\", "share", "\\", "repo"].join(""),
  ["/", "/", "server", "/", "share", "/", "repo"].join(""),
  ["/", "home", "/", "developer", "/", "repo"].join(""),
  ["/", "root", "/", "repo"].join(""),
  ["/", "tmp", "/", "project"].join(""),
  ["/", "workspace", "/", "project"].join(""),
  ["/", "workspaces", "/", "project"].join(""),
  ["/", "var", "/", "tmp", "/", "project"].join(""),
  ["/", "var", "/", "folders", "/", "xy", "/", "project"].join(""),
  ["/", "private", "/", "tmp", "/", "project"].join(""),
  ["/", "private", "/", "var", "/", "folders", "/", "xy", "/", "project"].join(
    "",
  ),
  ["/", "mnt", "/", "c", "/", "project"].join(""),
];
for (const fixture of negativeAbsolutePathFixtures) {
  assert.throws(
    () =>
      assertRetainedTextIsPublicSafe(
        "fixture.txt",
        fixture,
        new Set(),
        new Set(),
      ),
    /non-artificial absolute path/,
  );
}
const fakeDirectoryEntry = (name, symbolicLink = false) => ({
  name,
  isDirectory: () => true,
  isSymbolicLink: () => symbolicLink,
});
for (const name of rootGeneratedDirectoryNames) {
  assert.equal(classifyRetainedEntry(".", fakeDirectoryEntry(name)).skip, true);
  assert.equal(
    classifyRetainedEntry("docs", fakeDirectoryEntry(name)).skip,
    false,
  );
  assert.throws(
    () => classifyRetainedEntry(".", fakeDirectoryEntry(name, true)),
    /retained path must not be a symlink/,
  );
  assert.throws(
    () => classifyRetainedEntry("docs", fakeDirectoryEntry(name, true)),
    /retained path must not be a symlink/,
  );
}
const fixtureIdentifier = ["fixture", "host"].join("-");
assert.throws(
  () =>
    assertRetainedTextIsPublicSafe(
      "fixture.ts",
      `const machine = "${fixtureIdentifier}";`,
      new Set([sha256("fixture host")]),
      new Set(),
    ),
  /forbidden internal machine identifier/,
);

const pkg = json("package.json");
const tauri = json("src-tauri/tauri.conf.json");
const windowsTauri = json("src-tauri/tauri.windows.conf.json");
const capability = json("src-tauri/capabilities/default.json");
const cargo = read("src-tauri/Cargo.toml");
const readme = read("README.md");
const support = read("SUPPORT.md");
const lmStudioVerification = read(
  "docs/release/lm-studio-live-verification.md",
);
const security = read("SECURITY.md");
const conduct = read("CODE_OF_CONDUCT.md");
const deterministicWorkflow = read(".github/workflows/deterministic.yml");
const checklist = read("docs/release/linux-pre-release-verification.md");
const windowsChecklist = read(
  "docs/release/windows-pre-release-verification.md",
);
const windowsProviderVerification = read(
  "docs/release/windows-provider-verification.md",
);
const windowsAvailableMemoryVerification = read(
  "docs/release/windows-available-memory-verification.md",
);
const llamaCppVerification = read(
  "docs/release/llamacpp-passive-verification.md",
);
const unsignedPreviewContract = read(
  "docs/release/unsigned-preview-release-contract.md",
);
const signpathAssessment = read(
  "docs/release/signpath-eligibility-assessment.md",
);
const ollamaSource = read("src-tauri/src/runtime/ollama.rs");
const inventorySource = read("src-tauri/src/runtime/inventory.rs");
const loadedModelsSource = read("src-tauri/src/runtime/loaded_models.rs");
const inferenceSource = read("src-tauri/src/runtime/inference.rs");
const lmStudioSource = read("src-tauri/src/runtime/lm_studio.rs");
const llamaCppSource = read("src-tauri/src/runtime/llama_cpp.rs");
const commandsSource = read("src-tauri/src/commands.rs");
const platformSource = read("src-tauri/src/platform/mod.rs");
const linuxMetricsSource = read("src-tauri/src/platform/linux.rs");
const windowsMetricsSource = read("src-tauri/src/platform/windows.rs");
const machineContextSource = read("src-tauri/src/machine.rs");
const typesSource = read("src/lib/types.ts");
const reportSource = read("crates/aer-core/src/report.rs");
const appSource = read("src/App.svelte");
const appShellSource = read("src/lib/components/AppShell.svelte");
const availableMemoryHistorySource = read("src/lib/availableMemoryHistory.ts");
const availableMemoryComponentSource = read(
  "src/lib/components/AvailableMemoryHistory.svelte",
);
const inferenceHistorySource = read("src/lib/observationHistory.ts");
const resourceContextSource = read("src/lib/components/ResourceContext.svelte");
const reportWorkspaceSource = read("src/lib/components/ReportWorkspace.svelte");
const reportPreviewSource = read("src/lib/components/ReportPreview.svelte");
const diagnosticObservationSource = read("src/lib/diagnosticObservation.ts");
const diagnoseWorkspaceSource = read(
  "src/lib/components/DiagnoseWorkspace.svelte",
);
const diagnosisSource = read("src-tauri/src/diagnosis.rs");
const tauriLibSource = read("src-tauri/src/lib.rs");
const snapcraftProposal = read("snap/snapcraft.yaml");
const snapDesktop = read("snap/gui/ai-engine-room.desktop");
const snapcraftBuildPreparation = read(
  "docs/design/milestone-1y-c-snapcraft-one-build-preparation-contract.md",
);
const snapcraftExpansionVerification = read(
  "docs/distribution/snapcraft-expansion-verification-record.md",
);
const snapDispositionContract = read(
  "docs/design/milestone-1y-d-snap-disposition-contract.md",
);
const snapDisposition = read("docs/distribution/snap-disposition.md");

const sourceSection = (source, start, end) => {
  const startIndex = source.indexOf(start);
  assert.notEqual(startIndex, -1, `missing source section start: ${start}`);
  const endIndex = source.indexOf(end, startIndex);
  assert.notEqual(endIndex, -1, `missing source section end: ${end}`);
  return source.slice(startIndex, endIndex);
};

const assertHardenedClient = (section, name) => {
  assert.match(
    section,
    /reqwest::Client::builder\(\)/,
    `${name} client builder`,
  );
  assert.match(section, /Policy::none\(\)/, `${name} redirect rejection`);
  assert.match(section, /\.no_proxy\(\)/, `${name} proxy disablement`);
};

assert.equal(pkg.version, "0.1.0");
assert.equal(pkg.author, "Greg Weir");
assert.equal(pkg.license, "Apache-2.0");
assert.equal(tauri.productName, "AI Engine Room");
assert.equal(tauri.version, "0.1.0");
assert.equal(tauri.identifier, "com.tartanleaf.aiengineroom");
assert.equal(tauri.bundle.category, "Utility");
assert.equal(tauri.bundle.publisher, "Tartanleaf.com Inc.");
assert.equal(tauri.bundle.copyright, "Copyright © 2026 Tartanleaf.com Inc.");
assert.equal(tauri.bundle.license, "Apache-2.0");
assert.deepEqual(tauri.bundle.targets, ["deb"]);
assert.deepEqual(windowsTauri.bundle.targets, ["nsis"]);
assert.ok(
  !windowsTauri.bundle.targets.includes("msi"),
  "Windows packaging must remain NSIS-only",
);
assert.ok(
  !tauri.bundle.targets.includes("appimage"),
  "AppImage must remain outside the current packaging contract",
);
assert.ok(
  !("appimage" in (tauri.bundle.linux ?? {})),
  "deferred AppImage configuration must remain absent",
);
assert.equal(tauri.bundle.createUpdaterArtifacts, false);
assert.equal(
  Object.prototype.hasOwnProperty.call(tauri.bundle, "windows"),
  false,
  "base Tauri bundle must not configure Windows signing or installer customization",
);
assert.ok(
  !("windows" in (windowsTauri.bundle ?? {})),
  "Windows override must not add signing or installer customization",
);
assert.ok(
  !("android" in tauri.bundle),
  "unused Android bundle config must be absent",
);
assert.deepEqual(capability.permissions, [
  "core:default",
  "clipboard-manager:allow-write-text",
]);
assert.match(snapcraftProposal, /^name: ai-engine-room$/m);
assert.match(snapcraftProposal, /^version: "0\.1\.0"$/m);
assert.match(snapcraftProposal, /^license: Apache-2\.0$/m);
assert.match(snapcraftProposal, /^base: core24$/m);
assert.match(snapcraftProposal, /^grade: devel$/m);
assert.match(snapcraftProposal, /^confinement: strict$/m);
assert.match(snapcraftProposal, /^ {2}amd64:$/m);
assert.doesNotMatch(
  snapcraftProposal,
  /^ {2}(arm64|armhf|i386|riscv64|s390x):$/m,
);
assert.match(snapcraftProposal, /^ {4}command: bin\/aiengineroom$/m);
assert.match(
  snapcraftProposal,
  /^ {4}common-id: com\.tartanleaf\.aiengineroom$/m,
);
assert.match(snapcraftProposal, /^ {6}- gnome$/m);
assert.match(snapcraftProposal, /^ {6}- network$/m);
assert.doesNotMatch(snapcraftProposal, /confinement: (classic|devmode)/);
for (const forbiddenPlug of [
  "home",
  "network-bind",
  "network-control",
  "process-control",
  "system-observe",
]) {
  assert.doesNotMatch(
    snapcraftProposal,
    new RegExp(`^\\s+- ${forbiddenPlug}$`, "m"),
    `static Snap proposal must not add ${forbiddenPlug}`,
  );
}
for (const payload of [
  "LICENSE",
  "NOTICE",
  "THIRD-PARTY-LICENSES.txt",
  "THIRD-PARTY-SOURCES.txt",
]) {
  assert.match(
    snapcraftProposal,
    new RegExp(`CRAFT_PART_INSTALL/licenses/${payload.replaceAll(".", "\\.")}`),
    `static Snap proposal must retain ${payload}`,
  );
}
assert.match(snapDesktop, /^Exec=ai-engine-room$/m);
assert.match(
  snapDesktop,
  /^Icon=\$\{SNAP\}\/meta\/gui\/ai-engine-room\.png$/m,
);
assert.match(snapDesktop, /^Terminal=false$/m);
assert.doesNotMatch(snapDesktop, /(?:sh|bash|powershell|cmd)(?:\s|$)/im);
assert.match(
  snapcraftBuildPreparation,
  /Source and baseline commit \| `54d58e209399b830e929e4793e06dc778960689f`/,
);
assert.match(
  snapcraftBuildPreparation,
  /Definition Git blob \| `ff6d085085ae7dd693e2834541c5b1cb2c3c965c`/,
);
assert.match(snapcraftBuildPreparation, /exactly one fresh GitHub-hosted/);
assert.match(snapcraftBuildPreparation, /`ubuntu-24\.04` amd64/);
assert.match(snapcraftBuildPreparation, /`contents: read`/);
assert.match(snapcraftBuildPreparation, /45-minute job timeout/);
assert.match(snapcraftBuildPreparation, /`snapcraft expand-extensions`/);
assert.match(snapcraftBuildPreparation, /`mount-observe` and `calendar-service`/);
assert.match(snapcraftBuildPreparation, /`--destructive-mode`/);
assert.match(snapcraftBuildPreparation, /Exactly one `\.snap` must exist/);
assert.match(
  snapcraftBuildPreparation,
  /must not be uploaded as a workflow\s+artifact, installed, launched, refreshed, published, or retained/,
);
assert.match(
  snapcraftBuildPreparation,
  /must\s+separately approve:[\s\S]*execution change[\s\S]*exactly one run/,
);
assert.match(
  snapcraftBuildPreparation,
  /does not authorize installing or invoking\s+Snapcraft/,
);
assert.match(snapcraftBuildPreparation, /Gate consumed on 2026-08-31/);
assert.match(snapcraftBuildPreparation, /Result class \| `expansion_stop`/);
assert.match(
  snapcraftExpansionVerification,
  /ONE-RUN GATE CONSUMED; EXPANSION STOP; NO BUILD/,
);
assert.match(
  snapcraftExpansionVerification,
  /Execution commit \| `e9ef30a6f21772cfc7e21607bb9ca9caf2107590`/,
);
assert.match(
  snapcraftExpansionVerification,
  /Workflow run \| \[`33450652377`\]/,
);
assert.match(
  snapcraftExpansionVerification,
  /e20a2da87d3d4d0368235f34611febe54f8067499407926a2b4899a4f7c3c649/,
);
assert.match(
  snapcraftExpansionVerification,
  /did \*\*not\*\* add `mount-observe` or `calendar-service`/,
);
assert.match(snapcraftExpansionVerification, /canonical\/gpu-snap\.git/);
assert.match(
  snapcraftExpansionVerification,
  /No build command ran\. No `\.snap` was produced/,
);
assert.match(snapcraftExpansionVerification, /no retry is authorized/i);
assert.match(
  snapDispositionContract,
  /documentation-only closure proposal/,
);
assert.match(
  snapDispositionContract,
  /Defer Snap distribution for the current preview/,
);
assert.match(
  snapDispositionContract,
  /next recommended\s+planning slice is the already identified documentation-led bounded\s+network-observability feasibility study/,
);
assert.match(
  snapDisposition,
  /DEFERRED; NO ADDITIONAL SNAPCRAFT RUN AUTHORIZED/,
);
assert.match(snapDisposition, /canonical\/gpu-snap\.git/);
assert.match(
  snapDisposition,
  /not a supported\s+package definition, release candidate, store submission, or authorization to\s+run Snapcraft/,
);
assert.match(
  snapDisposition,
  /evidence of meaningful demand and a new\s+developer-approved contract/,
);
assert.match(
  snapcraftExpansionVerification,
  /developer selected deferral in the separately reviewed/,
);
assert.ok(!("remote" in capability), "capability must remain local");
assert.match(cargo, /^version = "0\.1\.0"$/m);
assert.match(
  cargo,
  /^description = "A pre-release desktop utility for inspecting Ollama and LM Studio runtime and resource context"$/m,
);
assert.match(cargo, /^authors = \["Greg Weir"\]$/m);
assert.match(cargo, /^license = "Apache-2\.0"$/m);
assert.match(cargo, /^homepage = "https:\/\/tartanleaf\.com"$/m);
assert.match(
  cargo,
  /\[target\.'cfg\(windows\)'\.dependencies\]\r?\nwindows-sys = \{ version = "0\.61\.2", default-features = false, features = \["Win32_System_SystemInformation"\] \}/,
  "Windows must use only the approved narrow windows-sys binding feature",
);
assert.doesNotMatch(
  cargo,
  /^(sysinfo|systemstat|heim|winapi)\s*=/m,
  "no broad or alternate Windows telemetry library",
);

const licenseHash = createHash("sha256").update(read("LICENSE")).digest("hex");
assert.equal(
  licenseHash,
  "cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30",
  "LICENSE must match the canonical Apache-2.0 text",
);
assert.equal(
  read(".gitattributes"),
  ".gitattributes text eol=lf\nLICENSE text eol=lf\n",
  "LICENSE must be checked out with canonical LF line endings",
);
const icon = readFileSync("src-tauri/icons/icon.ico");
assert.equal(
  createHash("sha256").update(icon).digest("hex"),
  "9edc4eefd796a55d08f945c5a911461f0c5fcf61ded9ab7665bd4f51f9be3a7f",
  "the approved C2R1 Windows icon must remain unchanged",
);
assert.equal(icon.readUInt16LE(0), 0);
assert.equal(icon.readUInt16LE(2), 1);
const iconLayers = Array.from({ length: icon.readUInt16LE(4) }, (_, index) => {
  const offset = 6 + index * 16;
  return [icon[offset] || 256, icon[offset + 1] || 256];
});
assert.deepEqual(iconLayers, [
  [32, 32],
  [16, 16],
  [24, 24],
  [48, 48],
  [64, 64],
  [256, 256],
]);
for (const text of [readme, support, checklist]) {
  assert.match(text, /Ubuntu 24\.04 LTS x86_64/);
  assert.match(text, /pre-release/i);
}
assert.match(readme, /Developed by Greg Weir/);
assert.match(readme, /Released by Tartanleaf\.com Inc\./);
assert.match(
  readme,
  /Verified public-preview and `\.deb` packaging baseline: Ubuntu 24\.04 LTS x86_64\. Developer install, launch, graphical, and removal verification passed for the exact published unsigned preview package\./,
);
assert.match(
  support,
  /\| Ubuntu 24\.04 LTS x86_64\s+\| Verified development and public unsigned `\.deb` preview baseline; exact-artifact install, launch, graphical, removal, and post-removal acceptance passed\s+\|/,
);
assert.match(
  support,
  /\| `\.deb`\s+\| Exact accepted unsigned package is published in `v0\.1\.0-preview\.1`/,
);
assert.match(
  support,
  /\| NSIS installer\s+\| Exact accepted unsigned installer is published in `v0\.1\.0-preview\.1`/,
);
assert.match(
  support,
  /\| AppImage\s+\| Deferred; the tested 1K package failed runtime acceptance/,
);
assert.match(checklist, /npm run tauri build -- --bundles deb/);
assert.match(checklist, /AppImage is not part of the current Milestone 1K/);
assert.doesNotMatch(checklist, /--bundles deb,appimage/);
assert.match(
  readme,
  /Verified public-preview and Windows packaging baseline: Windows 11 25H2 build 26200\.7462 x64\. Developer install, native launch, graphical\/accessibility, passive-behavior, and removal verification passed for the exact published unsigned NSIS preview package\./,
);
assert.match(
  support,
  /\| Windows 11 25H2 build 26200\.7462 x64\s+\| Verified public unsigned NSIS preview baseline; exact-artifact install, native launch, graphical\/accessibility, passive-behavior, removal, and post-removal acceptance passed\s+\|/,
);
assert.match(
  support,
  /\| Other Windows versions\/architectures\s+\| Not currently claimed\s+\|/,
);
assert.match(windowsChecklist, /npm run tauri build -- --bundles nsis/);
assert.match(
  windowsChecklist,
  /Do not execute, install, sign, upload, publish, or build MSI/,
);
assert.match(
  windowsChecklist,
  /Windows available-memory support has current-source verification for the exact artifact and tested baseline recorded in/,
);
const windowsAvailableMemoryStatus =
  "AI Engine Room's Windows available-memory observation has passed native compilation, current-source unsigned NSIS packaging, and developer package verification on Windows 11 25H2 build 26200.7462 x64.";
const windowsLinuxMemoryQualification =
  "The value is the operating system's reported available physical memory and is not claimed to be numerically equivalent to Linux `MemAvailable`.";
assert.ok(readme.includes(windowsAvailableMemoryStatus));
assert.ok(support.includes(windowsAvailableMemoryStatus));
assert.ok(readme.includes(windowsLinuxMemoryQualification));
assert.ok(support.includes(windowsLinuxMemoryQualification));
assert.match(
  windowsAvailableMemoryVerification,
  /Accepted source commit: `7e85354ea23f363eb1781c4fa342fa1e950da3be`/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /Windows 11 25H2, build 26200\.7462/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /B0B2F34A78483820A4281D7CB16F93D2092DD31C691FD0F761E9D991FFEF6A44/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /Filename: `AI Engine Room_0\.1\.0_x64-setup\.exe`/,
);
assert.match(windowsAvailableMemoryVerification, /Size: 2,487,355 bytes/);
assert.match(
  windowsAvailableMemoryVerification,
  /Authenticode status: `NotSigned`/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /`MEMORYSTATUSEX` binding compiled, `GlobalMemoryStatusEx` linked/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /does not establish Windows provider functionality or Windows\s+inference/,
);
assert.match(
  windowsAvailableMemoryVerification,
  /does not make AI Engine Room production-ready, a release\s+candidate, publicly released, or publicly available/,
);
assert.match(readme, /tested Windows 11 25H2 build 26200\.7462 x64 baseline/);
assert.match(readme, /tested Ollama version was 0\.32\.15/);
assert.match(
  readme,
  /does not establish broader Windows\/provider configuration compatibility or Windows inference/,
);
assert.match(support, /passive verification with Ollama 0\.32\.15 passed/);
assert.match(support, /passive native-v1 verification passed/);
assert.match(support, /Ollama \+ LM Studio coexistence/);
assert.match(support, /Windows inference was not verified/);
assert.match(windowsProviderVerification, /Ollama version: `0\.32\.15`/);
assert.match(windowsProviderVerification, /`127\.0\.0\.1:11434`/);
assert.match(windowsProviderVerification, /`127\.0\.0\.1:1234`/);
assert.match(
  windowsProviderVerification,
  /`api_scope = same_machine_loopback`/,
);
assert.match(
  windowsProviderVerification,
  /`compute_location = not_independently_verified`/,
);
assert.match(windowsProviderVerification, /No inference ran/);
assert.match(
  windowsProviderVerification,
  /No claim is made for other Windows versions/,
);
assert.match(readme, /LM Studio 0\.4\.0 or newer/);
assert.match(readme, /127\.0\.0\.1:1234/);
assert.match(support, /authenticated\/custom-port\/LAN LM Studio access/);
assert.match(lmStudioVerification, /liquid\/lfm2\.5-1\.2b/);
assert.match(
  lmStudioVerification,
  /live_lm_studio_native_v1_observation_is_bounded` passed/,
);
assert.match(lmStudioVerification, /`not_independently_verified`/);
const ollamaClient = sourceSection(
  ollamaSource,
  "impl ReqwestProbe {",
  "impl Default for ReqwestProbe",
);
const inventoryClient = sourceSection(
  inventorySource,
  "impl ReqwestModelsProbe {",
  "impl Default for ReqwestModelsProbe",
);
const loadedModelsClient = sourceSection(
  loadedModelsSource,
  "impl ReqwestRunningProbe {",
  "impl Default for ReqwestRunningProbe",
);
const inferenceClient = sourceSection(
  inferenceSource,
  "impl ReqwestGenerateProbe {",
  "impl Default for ReqwestGenerateProbe",
);
const lmStudioClient = sourceSection(
  lmStudioSource,
  "impl ReqwestLmStudioProbe {",
  "impl Default for ReqwestLmStudioProbe",
);
assertHardenedClient(ollamaClient, "Ollama runtime");
assertHardenedClient(inventoryClient, "Ollama inventory");
assertHardenedClient(loadedModelsClient, "Ollama loaded state");
assertHardenedClient(inferenceClient, "Ollama inference");
assertHardenedClient(lmStudioClient, "LM Studio shared");
assert.equal(
  lmStudioClient.match(/base\(\)/g)?.length,
  2,
  "both LM Studio clients must use the hardened shared builder",
);
assert.match(platformSource, /cfg\(target_os = "linux"\)[\s\S]*pub mod linux/);
assert.match(
  platformSource,
  /cfg\(any\(target_os = "windows", test\)\)[\s\S]*pub mod windows/,
);
assert.match(linuxMetricsSource, /\/proc\/meminfo/);
assert.match(linuxMetricsSource, /MemAvailable:/);
assert.match(linuxMetricsSource, /MemTotal:/);
assert.match(linuxMetricsSource, /os\.ram\.available/);
assert.match(linuxMetricsSource, /os\.ram\.total/);
assert.match(windowsMetricsSource, /GlobalMemoryStatusEx/);
assert.match(windowsMetricsSource, /MEMORYSTATUSEX/);
assert.match(
  windowsMetricsSource,
  /let mut status = MEMORYSTATUSEX \{[\s\S]*dwLength:\s*size_of::<MEMORYSTATUSEX>\(\) as u32,[\s\S]*\.\.Default::default\(\)[\s\S]*\};[\s\S]*query_global_memory_status\(&mut status\)/,
  "Windows memory status length must be initialized before the FFI call",
);
assert.match(windowsMetricsSource, /status\.ullAvailPhys/);
assert.match(windowsMetricsSource, /status\.ullTotalPhys/);
assert.match(
  windowsMetricsSource,
  /const RAM_AVAILABLE_ID: &str = "os\.ram\.available"/,
);
assert.doesNotMatch(
  windowsMetricsSource,
  /ullTotalPageFile|ullAvailPageFile|ullTotalVirtual|ullAvailVirtual|ullAvailExtendedVirtual|dwMemoryLoad/,
  "Windows provider must acquire only approved available and total physical memory",
);
assert.match(machineContextSource, /GetNativeSystemInfo/);
assert.doesNotMatch(machineContextSource, /GetSystemInfo\(/);
assert.match(machineContextSource, /fn uname\(/);
assert.match(machineContextSource, /"unknown"/);
assert.match(
  commandsSource,
  /target_os = "windows"[\s\S]*WindowsMetricsProvider::new\(\)/,
);
assert.match(
  commandsSource,
  /not\(target_os = "linux"\)[\s\S]*not\(target_os = "windows"\)[\s\S]*empty_os_snapshot\(\)/,
  "unsupported platforms retain an empty production fallback",
);
const reportAllowList = sourceSection(
  reportSource,
  "pub const REPORT_ALLOWED_METRICS",
  "const REDACTED",
);
assert.deepEqual(
  [...reportAllowList.matchAll(/"([a-z0-9.]+)"/g)].map((match) => match[1]),
  [
    "os.ram.available",
    "os.ram.used",
    "os.swap.used",
    "os.cpu.load",
    "os.disk.used",
  ],
  "Report metric allow-list must not expand for Windows support",
);
assert.match(
  availableMemoryHistorySource,
  /AVAILABLE_MEMORY_HISTORY_LIMIT = 12/,
  "Available-memory history must remain bounded to 12 session events",
);
assert.match(
  readme,
  /newest 12 Available-memory startup and explicit \*\*Refresh\*\* observations/,
);
assert.match(
  readme,
  /human-readable, allow-listed, sanitized plain-text report/,
);
assert.match(
  readme,
  /Available-memory session history adds no polling, persistence, trend, threshold, pressure, health, model-fit, or headroom judgement/,
);
assert.match(
  readme,
  /\*\*Diagnose\*\* workspace[\s\S]*newest 12 startup and explicit \*\*Refresh\*\* observation bundles/,
);
assert.match(readme, /Observation → Meaning → Safe next check/);
assert.match(
  readme,
  /Diagnose adds no acquisition, monitoring, persistence, root-cause claim, automated repair, or provider action/,
);
assert.match(
  availableMemoryHistorySource,
  /Number\.isSafeInteger\(value\)/,
  "Available-memory history must not plot unsafe numeric values",
);
assert.match(appSource, /appendAvailableMemoryObservation\(/);
assert.match(appSource, /if \(refreshPending\) return/);
assert.doesNotMatch(
  `${appSource}\n${availableMemoryHistorySource}`,
  /setInterval|setTimeout|localStorage|sessionStorage|indexedDB/,
  "Available-memory history must remain session-only without polling or persistence",
);
assert.doesNotMatch(
  inferenceHistorySource,
  /AvailableMemoryHistory|availableMemoryHistory/,
  "OS observations must remain separate from inference history",
);
assert.match(availableMemoryComponentSource, /<svg/);
assert.match(availableMemoryComponentSource, /aria-hidden="true"/);
assert.match(
  availableMemoryComponentSource,
  /Spacing shows observation\s+sequence, not time/,
);
assert.doesNotMatch(
  availableMemoryComponentSource,
  /<(?:line|polyline|path|polygon)\b|<canvas\b|<meter\b|<progress\b/,
  "Available-memory presentation must remain an unconnected inline-SVG sequence",
);
assert.doesNotMatch(
  resourceContextSource,
  /<svg\b|<canvas\b|<meter\b|<progress\b/,
  "Resource Context must remain non-graphical",
);
assert.match(resourceContextSource, /must not be added or subtracted/);
const reportRenderer = sourceSection(
  reportSource,
  "pub fn render_preview",
  "fn unit_label",
);
assert.match(reportRenderer, /AI Engine Room — Observation Report/);
assert.match(reportRenderer, /provenance_label/);
assert.match(reportRenderer, /limitation\.message\(\)/);
assert.match(reportRenderer, /Not representable exactly in this presentation/);
assert.doesNotMatch(
  reportRenderer,
  /snapshot\.at|millis_since_epoch/,
  "Report timestamp must remain outside rendered and copied text",
);
assert.match(reportWorkspaceSource, /copyText\(preview\)/);
assert.match(
  reportPreviewSource,
  /<pre class="report-preview-text">\{preview\}<\/pre>/,
);
assert.deepEqual(
  Object.keys({ ...pkg.dependencies, ...pkg.devDependencies }).filter((name) =>
    /(?:^|[-@/])(chart|charts|d3|plot|echarts|recharts)(?:$|[-/])/i.test(name),
  ),
  [],
  "Milestone 1R must not add a charting dependency",
);
assert.match(
  appShellSource,
  /\| "diagnose"[\s\S]*\{ id: "diagnose", label: "Diagnose" \}/,
  "the fifth Diagnose workspace must remain in the keyboard tab contract",
);
assert.match(
  diagnosticObservationSource,
  /DIAGNOSTIC_OBSERVATION_LIMIT = 12/,
  "diagnostic history must remain bounded to 12 session observations",
);
assert.match(appSource, /appendDiagnosticObservation\(/);
assert.match(appSource, /diagnosticHistory\.at\(-1\)/);
assert.doesNotMatch(
  `${appSource}\n${diagnosticObservationSource}`,
  /setInterval|setTimeout|localStorage|sessionStorage|indexedDB/,
  "diagnostic observations must remain session-only without polling or persistence",
);
assert.doesNotMatch(
  inferenceHistorySource,
  /DiagnosticObservation|diagnosticHistory/,
  "diagnostic observations must remain separate from inference history",
);
assert.match(diagnoseWorkspaceSource, /Changes since previous observation/);
assert.match(diagnoseWorkspaceSource, /Safe next check/);
assert.match(diagnoseWorkspaceSource, /not an atomic machine\s+snapshot/);
assert.doesNotMatch(
  diagnoseWorkspaceSource,
  /<svg\b|<canvas\b|<meter\b|<progress\b|health score|fit score|trend arrow/i,
  "Diagnose must remain text-first without scoring or monitoring graphics",
);
assert.match(diagnosisSource, /pub fn diagnose\(/);
assert.match(diagnosisSource, /fn valid_input\(/);
assert.doesNotMatch(
  diagnosisSource,
  /reqwest::|std::fs::|std::process::|Command::new|GlobalMemoryStatusEx|run_inference_observation_with_adapter|clipboard_manager/,
  "the deterministic diagnosis implementation must remain pure and no-I/O",
);
const diagnosisCommand = sourceSection(
  commandsSource,
  "pub fn diagnose_observation",
  "/// Run one observed inference",
);
assert.match(diagnosisCommand, /crate::diagnosis::diagnose\(&input\)/);
assert.doesNotMatch(
  diagnosisCommand,
  /build_snapshot|Adapter::new|run_inference|report_preview_of/,
  "the diagnosis command must use only its supplied bounded input",
);
assert.match(tauriLibSource, /commands::diagnose_observation/);
assert.doesNotMatch(
  reportSource,
  /DiagnosticObservation|DiagnosticFinding|diagnostic_history/,
  "diagnostic state must remain outside the Report model and renderer",
);
assert.doesNotMatch(
  reportWorkspaceSource,
  /Diagnose|diagnosticHistory|DiagnosticFinding/,
  "diagnostic state must remain outside visible Report and Copy",
);
assert.match(ollamaSource, /http:\/\/127\.0\.0\.1:11434\/api\/version/);
assert.match(inventorySource, /http:\/\/127\.0\.0\.1:11434\/api\/tags/);
assert.match(loadedModelsSource, /http:\/\/127\.0\.0\.1:11434\/api\/ps/);
assert.match(inferenceSource, /http:\/\/127\.0\.0\.1:11434\/api\/generate/);
assert.match(lmStudioSource, /http:\/\/127\.0\.0\.1:1234\/api\/v1\/models/);
assert.match(lmStudioSource, /http:\/\/127\.0\.0\.1:1234\/api\/v1\/chat/);
assert.match(
  readme,
  /AI Engine Room can passively detect and display a traditional single-model llama-server/,
);
assert.match(readme, /127\.0\.0\.1:8080/);
assert.match(readme, /tested Ubuntu 24\.04 LTS x86_64 baseline/);
assert.match(readme, /Snap `llama-cpp` label `b9969`, revision `307`/);
assert.match(readme, /server build commit `76f2798`/);
assert.match(support, /llama\.cpp is passive-only in Milestone 1O/);
assert.match(
  support,
  /verification passed on the tested Ubuntu 24\.04 LTS x86_64 baseline/i,
);
assert.match(support, /traditional single-model llama-server/);
assert.match(support, /validated provider-reported served-model ID/);
assert.match(
  support,
  /does not support inference, router mode, authentication, TLS, custom endpoints, LAN\/remote access, service control, or model management/,
);
assert.match(llamaCppSource, /http:\/\/127\.0\.0\.1:8080\/health/);
assert.match(llamaCppSource, /http:\/\/127\.0\.0\.1:8080\/v1\/models/);
assert.match(llamaCppSource, /Policy::none\(\)/);
assert.match(llamaCppSource, /\.no_proxy\(\)/);
assert.match(llamaCppSource, /Duration::from_millis\(500\)/);
assert.match(llamaCppSource, /Duration::from_secs\(2\)/);
assert.match(llamaCppSource, /MAX_RESPONSE_BYTES: usize = 16 \* 1024/);
assert.match(llamaCppSource, /response\.chunk\(\)\.await/);
assert.match(llamaCppSource, /pub model_id: String/);
assert.match(llamaCppSource, /owned_by: String/);
assert.match(llamaCppSource, /data\.owned_by != "llamacpp"/);
assert.doesNotMatch(llamaCppSource, /LlamaCppState::NotDetected/);
assert.doesNotMatch(llamaCppSource, /struct ModelWire/);
assert.doesNotMatch(llamaCppSource, /\.post\s*\(/);
assert.doesNotMatch(llamaCppSource, /chat\/completions|\/completion/);
assert.match(commandsSource, /current_llama_cpp_snapshot/);
assert.match(
  typesSource,
  /export type RuntimeProvider = "ollama" \| "lm_studio";/,
);
assert.doesNotMatch(reportSource, /llama_cpp|llama\.cpp/i);
assert.match(
  llamaCppVerification,
  /Aligned AI Engine Room source commit: `6fbd802c59d56ade4b37d73d8b6f50399ead92fe`/,
);
assert.match(llamaCppVerification, /Snap `llama-cpp`, `latest\/stable`/);
assert.match(llamaCppVerification, /Version\s+\| `b9969`/);
assert.match(llamaCppVerification, /Snap revision\s+\| `307`/);
assert.match(llamaCppVerification, /server build commit `76f2798`/);
assert.match(
  llamaCppVerification,
  /`GET \/health` returned HTTP 200 with `status = ok`/,
);
assert.match(
  llamaCppVerification,
  /`GET \/v1\/models` still reported exactly one `data` entry with `id = lfm2\.5-1\.2b` and `owned_by = llamacpp`/,
);
assert.match(
  llamaCppVerification,
  /No llama\.cpp inference or POST request ran/,
);
assert.match(
  llamaCppVerification,
  /Compute location\s+\| `not_independently_verified`/,
);
assert.match(
  llamaCppVerification,
  /One passive Refresh preserved the observed server\/model state/,
);
assert.match(
  llamaCppVerification,
  /Broader llama\.cpp versions, router\/multi-model mode, authentication, TLS, custom endpoints\/ports, LAN\/remote access, Windows llama\.cpp support, model compatibility, inference, and compute placement are not established/,
);
assert.match(security, /gweir@tartanleaf\.com/);
assert.match(security, /exact public unsigned `v0\.1\.0-preview\.1` artifacts/);
assert.match(conduct, /gweir@tartanleaf\.com/);
assert.match(deterministicWorkflow, /ubuntu-latest/);
assert.match(deterministicWorkflow, /windows-latest/);
assert.match(deterministicWorkflow, /fetch-depth: 0/);
assert.match(deterministicWorkflow, /RUST_TEST_THREADS: 1/);
assert.match(deterministicWorkflow, /cargo test --workspace/);
assert.match(
  deterministicWorkflow,
  /node scripts\/verify-target-license-coverage\.mjs/,
);
assert.match(deterministicWorkflow, /npm run licenses:generate/);
assert.match(deterministicWorkflow, /git diff --exit-code/);
assert.match(deterministicWorkflow, /Package licences/);
assert.match(deterministicWorkflow, /npm run tauri build -- --bundles/);
assert.match(
  deterministicWorkflow,
  /node scripts\/verify-package-license-payload\.mjs/,
);
assert.match(
  deterministicWorkflow,
  /cargo clippy --workspace --all-targets -- -D warnings/,
);
assert.match(deterministicWorkflow, /npm run test:run/);
assert.doesNotMatch(
  deterministicWorkflow,
  /upload-artifact|release|publish|sign/i,
);
assert.match(
  unsignedPreviewContract,
  /Status: approved planning and verification contract/,
);
assert.match(unsignedPreviewContract, /Unsigned preview/);
assert.match(unsignedPreviewContract, /Get-FileHash -Algorithm SHA256/);
assert.match(
  unsignedPreviewContract,
  /must not tell users to accept a warning blindly/,
);
assert.match(
  unsignedPreviewContract,
  /Publication remains a distinct\s+approval/,
);
assert.match(signpathAssessment, /not application-ready/i);
assert.match(signpathAssessment, /SignPath Foundation/);
assert.match(
  signpathAssessment,
  /public \[`v0\.1\.0-preview\.1` release\]/,
);
assert.match(signpathAssessment, /No application,\s+account, certificate/);
assert.match(
  signpathAssessment,
  /Greg Weir as the developer and Tartanleaf\.com Inc\. as the product\s+and package publisher/,
);
assert.match(
  signpathAssessment,
  /SignPath Foundation as its authenticated certificate publisher remains a\s+separate future decision/,
);
for (const text of [readme, support, signpathAssessment]) {
  assert.match(
    text,
    /https:\/\/github\.com\/gregweir\/ai-engine-room\/releases\/tag\/v0\.1\.0-preview\.1/,
  );
}
assert.equal(
  read("NOTICE").replace(/\r\n/g, "\n"),
  "AI Engine Room\nCopyright 2026 Tartanleaf.com Inc.\n\nThis product is licensed under the Apache License, Version 2.0.\n",
  "NOTICE must contain the approved product copyright attribution",
);
assert.ok(
  !("plugins" in tauri && "updater" in tauri.plugins),
  "updater is deferred",
);
console.log("Repository and packaging-readiness contract verified.");
