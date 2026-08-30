import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";

const read = (path) => readFileSync(path);
const text = (path) => read(path).toString("utf8");
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const normalizedFileSha256 = (value) =>
  sha256(Buffer.from(value.toString("utf8").replace(/\r\n/g, "\n")));
const manifestPath = "third-party/generated-manifest.json";
const generatedPaths = [
  "THIRD-PARTY-LICENSES.txt",
  "THIRD-PARTY-SOURCES.txt",
  "packaging/debian/copyright",
];
for (const path of [manifestPath, ...generatedPaths]) {
  assert.ok(existsSync(path), `missing generated licence material: ${path}`);
}

const manifest = JSON.parse(text(manifestPath));
assert.equal(manifest.schemaVersion, 1);
assert.equal(
  manifest.generatorSha256,
  normalizedFileSha256(read("scripts/generate-third-party-licenses.mjs")),
  "generated licence material is stale for its generator",
);
assert.equal(manifest.spdxLicenseListRevision, "a3cbf2e897d54bccec0c35469c691521d089ef53");
assert.equal(
  manifest.cargoLockSha256,
  normalizedFileSha256(read("Cargo.lock")),
  "third-party material is stale for Cargo.lock",
);
assert.equal(
  manifest.packageLockSha256,
  normalizedFileSha256(read("package-lock.json")),
  "third-party material is stale for package-lock.json",
);
assert.ok(manifest.rustRegistryComponentCount > 200);
assert.ok(manifest.frontendComponentCount > 3);
assert.equal(manifest.installerComponentCount, 2);
assert.deepEqual(manifest.installerPayloads, {
  "$PLUGINSDIR\\System.dll": "NSIS@3.11",
  "$PLUGINSDIR\\modern-wizard.bmp": "NSIS@3.11",
  "$PLUGINSDIR\\nsDialogs.dll": "NSIS@3.11",
  "$PLUGINSDIR\\nsis_tauri_utils.dll": "nsis_tauri_utils@0.5.3",
  "$PLUGINSDIR\\StartMenu.dll": "NSIS@3.11",
  "$PLUGINSDIR\\NSISdl.dll": "NSIS@3.11",
  "aiengineroom.exe": "AI Engine Room@0.1.0",
});
assert.match(manifest.installerPayloadSha256["$PLUGINSDIR\\modern-wizard.bmp"], /^[a-f0-9]{64}$/);
assert.deepEqual(manifest.installerSourceArchives, {
  "NSIS@3.11": {
    url: "https://downloads.sourceforge.net/project/nsis/NSIS%203/3.11/nsis-3.11-src.tar.bz2",
    sha256: "19e72062676ebdc67c11dc032ba80b979cdbffd3886c60b04bb442cdd401ff4b",
  },
  "nsis_tauri_utils@0.5.3": {
    url: "https://codeload.github.com/tauri-apps/nsis-tauri-utils/tar.gz/13d9edd27b69310e108d6fbd49f90992f8a05390",
    sha256: "9b277f7cdec4277b48d319c32e10b5dc4303e9efce8c3b44f1db77f86a8131c7",
  },
});
assert.ok(manifest.uniqueLicenseDocumentCount > 100);
for (const path of generatedPaths) {
  assert.equal(
    manifest.outputs[path],
    normalizedFileSha256(read(path)),
    `generated licence material has changed: ${path}`,
  );
}

const components = new Map(
  manifest.components.map((component) => [`${component.name}@${component.version}`, component]),
);
const packageJson = JSON.parse(text("package.json"));
const packageLock = JSON.parse(text("package-lock.json"));
const resolveLockedDependency = (parentPath, name) => {
  const candidates = [];
  let scope = parentPath;
  while (scope.startsWith("node_modules/")) {
    candidates.push(`${scope}/node_modules/${name}`);
    const nestedAt = scope.lastIndexOf("/node_modules/");
    if (nestedAt < 0) break;
    scope = scope.slice(0, nestedAt);
  }
  candidates.push(`node_modules/${name}`);
  return candidates.find((path) => packageLock.packages[path]);
};
const expectedFrontendByPath = new Map();
const visitFrontend = (name, path) => {
  if (expectedFrontendByPath.has(path)) return;
  const entry = packageLock.packages[path];
  assert.ok(entry, `missing locked frontend package: ${path}`);
  expectedFrontendByPath.set(path, `${name}@${entry.version}`);
  for (const dependency of Object.keys(entry.dependencies ?? {})) {
    const dependencyPath = resolveLockedDependency(path, dependency);
    assert.ok(dependencyPath, `cannot resolve locked dependency ${dependency} from ${path}`);
    visitFrontend(dependency, dependencyPath);
  }
};
for (const name of [...Object.keys(packageJson.dependencies ?? {}), "svelte"]) {
  visitFrontend(name, `node_modules/${name}`);
}
const expectedFrontend = [...expectedFrontendByPath.values()].sort();
const actualFrontend = manifest.components
  .slice(manifest.rustRegistryComponentCount, manifest.rustRegistryComponentCount + manifest.frontendComponentCount)
  .map((component) => `${component.name}@${component.version}`)
  .sort();
assert.deepEqual(actualFrontend, expectedFrontend, "frontend publication inventory is stale");
for (const expected of [
  "@tauri-apps/api@2.11.1",
  "@tauri-apps/plugin-clipboard-manager@2.3.2",
  "svelte@5.56.9",
  "clsx@2.1.1",
  "esm-env@1.2.2",
  "NSIS@3.11",
  "nsis_tauri_utils@0.5.3",
  "cssparser@0.36.0",
  "cssparser-macros@0.6.1",
  "dtoa-short@0.3.5",
  "option-ext@0.2.0",
  "selectors@0.36.1",
]) {
  assert.ok(components.has(expected), `missing required publication component: ${expected}`);
}

const sources = text("THIRD-PARTY-SOURCES.txt");
assert.match(sources, /corresponding source locations/);
assert.match(sources, /MPL-2\.0 target-graph candidates/);
assert.match(sources, /NSIS\s+source archive covers the LZMA module/);
const debianCopyright = text("packaging/debian/copyright");
assert.match(debianCopyright, /^AI Engine Room\r?\nCopyright 2026 Tartanleaf\.com Inc\./);
assert.match(debianCopyright, /APPLICATION LICENCE/);
assert.match(debianCopyright, /THIRD-PARTY LICENCES AND NOTICES/);

const aerCoreCargo = text("crates/aer-core/Cargo.toml");
assert.match(aerCoreCargo, /^license = "Apache-2\.0"$/m);
const tauri = JSON.parse(text("src-tauri/tauri.conf.json"));
assert.deepEqual(tauri.bundle.resources, {
  "../LICENSE": "licenses/LICENSE",
  "../NOTICE": "licenses/NOTICE",
  "../THIRD-PARTY-LICENSES.txt": "licenses/THIRD-PARTY-LICENSES.txt",
  "../THIRD-PARTY-SOURCES.txt": "licenses/THIRD-PARTY-SOURCES.txt",
});
assert.deepEqual(tauri.bundle.linux.deb.files, {
  "/usr/share/doc/ai-engine-room/NOTICE": "../NOTICE",
  "/usr/share/doc/ai-engine-room/THIRD-PARTY-SOURCES.txt": "../THIRD-PARTY-SOURCES.txt",
  "/usr/share/doc/ai-engine-room/copyright": "../packaging/debian/copyright",
});

console.log("Third-party licence materials verified.");
