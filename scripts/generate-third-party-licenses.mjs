import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
  mkdirSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { gunzipSync } from "node:zlib";

const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const read = (path) => readFileSync(path);
const text = (path) => read(path).toString("utf8");
const normalizeText = (value) =>
  value.replace(/\r\n/g, "\n").replace(/[ \t]+$/gm, "").trimEnd() + "\n";

const parseCargoLock = (source) =>
  source
    .split(/\r?\n(?=\[\[package\]\])/)
    .filter((block) => block.startsWith("[[package]]"))
    .map((block) => {
      const field = (name) => {
        const value = block.match(new RegExp(`^${name} = ("(?:[^"\\\\]|\\\\.)*")$`, "m"));
        return value ? JSON.parse(value[1]) : undefined;
      };
      return {
        name: field("name"),
        version: field("version"),
        source: field("source"),
        checksum: field("checksum"),
      };
    })
    .filter((item) => item.source?.startsWith("registry+") && item.checksum)
    .sort((a, b) => `${a.name}@${a.version}`.localeCompare(`${b.name}@${b.version}`));

const parseTar = (archive) => {
  const files = new Map();
  let offset = 0;
  while (offset + 512 <= archive.length) {
    const header = archive.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) break;
    const stringAt = (start, length) =>
      header.subarray(start, start + length).toString("utf8").replace(/\0.*$/s, "");
    const name = stringAt(0, 100);
    const prefix = stringAt(345, 155);
    const size = Number.parseInt(stringAt(124, 12).trim() || "0", 8);
    const type = String.fromCharCode(header[156] || 48);
    const path = prefix ? `${prefix}/${name}` : name;
    offset += 512;
    if (type === "0" || type === "\0") {
      files.set(path.replace(/^[^/]+\//, ""), archive.subarray(offset, offset + size));
    }
    offset += Math.ceil(size / 512) * 512;
  }
  return files;
};

const licenseLike = (path) => {
  const parts = path.split("/");
  const file = parts.at(-1);
  return (
    parts.length <= 3 &&
    /^(?:license|licence|copying|copyright|notice|unlicense)(?:[._-].*)?$/i.test(file)
  );
};

const packageLicense = (files) => {
  const manifest = (files.get("Cargo.toml") ?? files.get("Cargo.toml.orig"))?.toString("utf8") ?? "";
  const declared = manifest.match(/^license\s*=\s*"([^"]+)"/m)?.[1];
  const declaredFile = manifest.match(/^license[-_]file\s*=\s*"([^"]+)"/m)?.[1];
  const selected = [...files]
    .filter(([path]) => licenseLike(path) || path === declaredFile)
    .sort(([a], [b]) => a.localeCompare(b));
  return { declared: declared ?? (declaredFile ? `file: ${declaredFile}` : "not declared"), selected };
};

const fetchBytes = async (url) => {
  const response = await fetch(url, { redirect: "follow" });
  assert.ok(response.ok, `download failed (${response.status}): ${url}`);
  return Buffer.from(await response.arrayBuffer());
};

const spdxRevision = "a3cbf2e897d54bccec0c35469c691521d089ef53";
const spdxCache = new Map();
const spdxFiles = async (expression) => {
  const identifiers = [...new Set(expression.match(/[A-Za-z0-9][A-Za-z0-9.-]*/g) ?? [])]
    .filter((value) => !["AND", "OR", "WITH", "file", "not", "declared"].includes(value));
  assert.ok(identifiers.length > 0, `cannot derive SPDX fallback from: ${expression}`);
  return Promise.all(
    identifiers.map(async (identifier) => {
      if (!spdxCache.has(identifier)) {
        const url = `https://raw.githubusercontent.com/spdx/license-list-data/${spdxRevision}/text/${identifier}.txt`;
        spdxCache.set(identifier, fetchBytes(url));
      }
      return [`SPDX-${identifier}.txt`, await spdxCache.get(identifier)];
    }),
  );
};

const mapConcurrent = async (items, limit, operation) => {
  const results = new Array(items.length);
  let next = 0;
  await Promise.all(
    Array.from({ length: Math.min(limit, items.length) }, async () => {
      while (next < items.length) {
        const index = next++;
        results[index] = await operation(items[index], index);
      }
    }),
  );
  return results;
};

const renderComponent = ({ name, version, declared, sourceUrl, documentRefs }) => {
  return [
    `${name} ${version}`,
    `Declared licence: ${declared}`,
    `Exact source: ${sourceUrl}`,
    `Included documents: ${documentRefs.map(({ id, path }) => `${id} (${path})`).join(", ")}`,
  ].join("\n");
};

const cargoLock = read("Cargo.lock");
const packageLock = read("package-lock.json");
const packageJson = JSON.parse(text("package.json"));
const cargoPackages = parseCargoLock(cargoLock.toString("utf8"));
assert.ok(cargoPackages.length > 200, "unexpectedly small Cargo registry inventory");

const rustComponents = await mapConcurrent(cargoPackages, 8, async (item) => {
  const sourceUrl = `https://static.crates.io/crates/${encodeURIComponent(item.name)}/${encodeURIComponent(item.name)}-${encodeURIComponent(item.version)}.crate`;
  const archive = await fetchBytes(sourceUrl);
  assert.equal(sha256(archive), item.checksum, `Cargo checksum mismatch: ${item.name} ${item.version}`);
  const files = parseTar(gunzipSync(archive));
  const { declared, selected } = packageLicense(files);
  if (selected.length === 0) {
    assert.notEqual(declared, "not declared", `crate has neither licence metadata nor files: ${item.name} ${item.version}`);
    selected.push(...(await spdxFiles(declared)));
    for (const path of ["README.md", "README", "Cargo.toml.orig", "Cargo.toml"]) {
      if (files.has(path)) selected.push([path, files.get(path)]);
    }
  }
  return {
    name: item.name,
    version: item.version,
    declared,
    sourceUrl: `https://crates.io/api/v1/crates/${encodeURIComponent(item.name)}/${encodeURIComponent(item.version)}/download`,
    licenses: selected,
  };
});

const npmLock = JSON.parse(packageLock.toString("utf8"));
assert.ok(packageJson.devDependencies?.svelte, "Svelte compiler must remain an explicit reviewed frontend input");
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
  return candidates.find((path) => npmLock.packages[path]);
};
const frontendByPath = new Map();
const visitFrontend = (name, path) => {
  if (frontendByPath.has(path)) return;
  const entry = npmLock.packages[path];
  assert.ok(entry, `missing locked npm package path: ${path}`);
  frontendByPath.set(path, { name, path });
  for (const dependency of Object.keys(entry.dependencies ?? {}).sort()) {
    const dependencyPath = resolveLockedDependency(path, dependency);
    assert.ok(dependencyPath, `cannot resolve locked dependency ${dependency} from ${path}`);
    visitFrontend(dependency, dependencyPath);
  }
};
for (const name of [...Object.keys(packageJson.dependencies ?? {}), "svelte"].sort()) {
  visitFrontend(name, `node_modules/${name}`);
}
const frontendPackages = [...frontendByPath.values()].sort((a, b) =>
  `${a.name}@${a.path}`.localeCompare(`${b.name}@${b.path}`),
);
const frontendComponents = await mapConcurrent(frontendPackages, 4, async ({ name, path }) => {
  const entry = npmLock.packages[path];
  assert.ok(entry?.version && entry?.resolved && entry?.integrity, `missing locked npm package: ${name}`);
  const archive = await fetchBytes(entry.resolved);
  const [algorithm, encoded] = entry.integrity.split("-");
  const actual = createHash(algorithm).update(archive).digest("base64");
  assert.equal(actual, encoded, `npm integrity mismatch: ${name}`);
  const files = parseTar(gunzipSync(archive));
  const selected = [...files]
    .filter(([path]) => licenseLike(path))
    .sort(([a], [b]) => a.localeCompare(b));
  const declared = entry.license ?? "not declared";
  if (selected.length === 0) {
    assert.notEqual(declared, "not declared", `npm package has neither licence metadata nor files: ${name}`);
    selected.push(...(await spdxFiles(declared)));
    for (const contextPath of ["package.json", "README.md", "README"]) {
      if (files.has(contextPath)) selected.push([contextPath, files.get(contextPath)]);
    }
  }
  return {
    name,
    version: entry.version,
    declared,
    sourceUrl: entry.resolved,
    licenses: selected,
  };
});

const nsisUtilsCommit = "13d9edd27b69310e108d6fbd49f90992f8a05390";
const nsisUtilsSourceUrl =
  `https://codeload.github.com/tauri-apps/nsis-tauri-utils/tar.gz/${nsisUtilsCommit}`;
const nsisUtilsArchiveHash = "9b277f7cdec4277b48d319c32e10b5dc4303e9efce8c3b44f1db77f86a8131c7";
const nsisUtilsArchive = await fetchBytes(nsisUtilsSourceUrl);
assert.equal(sha256(nsisUtilsArchive), nsisUtilsArchiveHash, "installer helper source hash mismatch");
const nsisUtilsFiles = parseTar(gunzipSync(nsisUtilsArchive));
const nsisUtilsLicenses = ["LICENSE_APACHE-2.0", "LICENSE_MIT"].map((path) => {
  assert.ok(nsisUtilsFiles.has(path), `installer helper source lacks ${path}`);
  return [path, nsisUtilsFiles.get(path)];
});
const nsisUtils = {
  name: "nsis_tauri_utils",
  version: "0.5.3",
  declared: "Apache-2.0 OR MIT",
  sourceUrl: nsisUtilsSourceUrl,
  licenses: nsisUtilsLicenses,
};

const work = mkdtempSync(join(tmpdir(), "aer-nsis-license-"));
let nsisCopying;
let nsisWizardBitmap;
try {
  const nsisSourceUrl =
    "https://downloads.sourceforge.net/project/nsis/NSIS%203/3.11/nsis-3.11-src.tar.bz2";
  const sourceArchive = join(work, "nsis-3.11-src.tar.bz2");
  const sourceBytes = await fetchBytes(nsisSourceUrl);
  assert.equal(
    sha256(sourceBytes),
    "19e72062676ebdc67c11dc032ba80b979cdbffd3886c60b04bb442cdd401ff4b",
    "NSIS 3.11 source archive hash mismatch",
  );
  writeFileSync(sourceArchive, sourceBytes);
  nsisCopying = execFileSync("tar", ["-xOf", sourceArchive, "nsis-3.11-src/COPYING"]);
  nsisWizardBitmap = execFileSync("tar", [
    "-xOf",
    sourceArchive,
    "nsis-3.11-src/Contrib/Graphics/Wizard/win.bmp",
  ]);
} finally {
  rmSync(work, { recursive: true, force: true });
}
const nsis = {
  name: "NSIS",
  version: "3.11",
  declared: "zlib/libpng; LZMA module under CPL-1.0 with linking exception",
  sourceUrl:
    "https://downloads.sourceforge.net/project/nsis/NSIS%203/3.11/nsis-3.11-src.tar.bz2",
  licenses: [["COPYING", nsisCopying]],
};

const components = [...rustComponents, ...frontendComponents, nsis, nsisUtils];
const documentsByHash = new Map();
for (const component of components) {
  component.documentRefs = component.licenses.map(([path, contents]) => {
    const normalized = normalizeText(contents.toString("utf8"));
    const hash = sha256(Buffer.from(normalized));
    if (!documentsByHash.has(hash)) {
      documentsByHash.set(hash, {
        id: `L${String(documentsByHash.size + 1).padStart(4, "0")}`,
        hash,
        text: normalized,
      });
    }
    return { id: documentsByHash.get(hash).id, path };
  });
}
const documents = [...documentsByHash.values()];
const header = normalizeText(`AI Engine Room third-party licence and notice material

Generated from the exact Cargo.lock and package-lock.json inputs. The Rust
section intentionally includes every registry package in Cargo.lock. This is a
conservative superset of the code shipped on either supported build target and
must not be read as proof that every listed package is present in every binary.

The frontend section contains the locked packages compiled into the production
frontend even where the package manager classifies a compiler as a development
dependency. The installer section covers the exact NSIS helper baseline used by
the accepted Windows packaging toolchain.

Each component entry reproduces the licence and notice files included in its
verified upstream archive, normalized only to LF line endings with trailing
horizontal whitespace removed. Exact source retrieval locations are also
collected in THIRD-PARTY-SOURCES.txt.
`);
const thirdParty = normalizeText(`${header}
COMPONENT INDEX
===============

${components.map(renderComponent).join("\n\n")}

LICENCE AND NOTICE DOCUMENTS
============================

${documents
  .map(
    ({ id, hash, text: documentText }) =>
      `${"=".repeat(78)}\n${id} SHA-256 ${hash}\n${"=".repeat(78)}\n\n${documentText}`,
  )
  .join("\n")}
`);
const sources = normalizeText(`AI Engine Room corresponding third-party source locations

Recipients of an AI Engine Room binary can obtain the exact third-party source
used for the locked build from the URLs below. The listed crate archive URLs are
also the corresponding source locations for MPL-2.0 target-graph candidates;
the conservative lock inventory may include build-only components. The NSIS
source archive covers the LZMA module carried by the Windows installer.

${components.map((item) => `${item.name} ${item.version}\n${item.sourceUrl}`).join("\n\n")}
`);
const applicationLicense = text("LICENSE");
const applicationNotice = normalizeText(text("NOTICE"));
const debianCopyright = normalizeText(`AI Engine Room
Copyright 2026 Tartanleaf.com Inc.
Source: https://github.com/gregweir/ai-engine-room

APPLICATION LICENCE

${applicationLicense}

APPLICATION NOTICE

${applicationNotice}

THIRD-PARTY LICENCES AND NOTICES

${thirdParty}
`);

mkdirSync("third-party", { recursive: true });
mkdirSync("packaging/debian", { recursive: true });
writeFileSync("THIRD-PARTY-LICENSES.txt", thirdParty);
writeFileSync("THIRD-PARTY-SOURCES.txt", sources);
writeFileSync("packaging/debian/copyright", debianCopyright);
const manifest = {
  schemaVersion: 1,
  generatorSha256: sha256(read("scripts/generate-third-party-licenses.mjs")),
  spdxLicenseListRevision: spdxRevision,
  cargoLockSha256: sha256(cargoLock),
  packageLockSha256: sha256(packageLock),
  rustRegistryComponentCount: rustComponents.length,
  frontendComponentCount: frontendComponents.length,
  installerComponentCount: 2,
  installerPayloads: {
    "$PLUGINSDIR\\System.dll": "NSIS@3.11",
    "$PLUGINSDIR\\modern-wizard.bmp": "NSIS@3.11",
    "$PLUGINSDIR\\nsDialogs.dll": "NSIS@3.11",
    "$PLUGINSDIR\\nsis_tauri_utils.dll": "nsis_tauri_utils@0.5.3",
    "$PLUGINSDIR\\StartMenu.dll": "NSIS@3.11",
    "$PLUGINSDIR\\NSISdl.dll": "NSIS@3.11",
    "aiengineroom.exe": "AI Engine Room@0.1.0",
  },
  installerPayloadSha256: {
    "$PLUGINSDIR\\modern-wizard.bmp": sha256(nsisWizardBitmap),
  },
  installerSourceArchives: {
    "NSIS@3.11": {
      url: nsis.sourceUrl,
      sha256: "19e72062676ebdc67c11dc032ba80b979cdbffd3886c60b04bb442cdd401ff4b",
    },
    "nsis_tauri_utils@0.5.3": {
      url: nsisUtilsSourceUrl,
      sha256: nsisUtilsArchiveHash,
    },
  },
  uniqueLicenseDocumentCount: documents.length,
  outputs: {
    "THIRD-PARTY-LICENSES.txt": sha256(Buffer.from(thirdParty)),
    "THIRD-PARTY-SOURCES.txt": sha256(Buffer.from(sources)),
    "packaging/debian/copyright": sha256(Buffer.from(debianCopyright)),
  },
  components: components.map(({ name, version, declared, sourceUrl, licenses }) => ({
    name,
    version,
    declared,
    sourceUrl,
    files: licenses.map(([path]) => path),
  })),
};
writeFileSync("third-party/generated-manifest.json", `${JSON.stringify(manifest, null, 2)}\n`);
console.log(
  `Generated ${components.length} component records (${rustComponents.length} Rust, ${frontendComponents.length} frontend, 2 installer).`,
);
