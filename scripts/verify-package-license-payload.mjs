import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";

const kind = process.argv[2];
assert.ok(kind === "linux" || kind === "windows", "expected linux or windows package kind");
const walk = (root) =>
  readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  });
const normalized = (path) => path.replaceAll("\\", "/").toLowerCase();
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const findOne = (files, suffix) => {
  const matches = files.filter((path) => normalized(path).endsWith(suffix.toLowerCase()));
  assert.equal(matches.length, 1, `expected exactly one packaged ${suffix}, found ${matches.length}`);
  return matches[0];
};
const assertSame = (actualPath, expectedPath) =>
  assert.deepEqual(readFileSync(actualPath), readFileSync(expectedPath), `packaged file differs: ${actualPath}`);

const bundleRoot = join("target", "release", "bundle");
const artifactDirectory = join(bundleRoot, kind === "linux" ? "deb" : "nsis");
const artifactSuffix = kind === "linux" ? ".deb" : "-setup.exe";
const artifacts = walk(artifactDirectory).filter((path) => path.endsWith(artifactSuffix));
assert.equal(artifacts.length, 1, `expected exactly one ${kind} package artifact`);
const extraction = mkdtempSync(join(tmpdir(), `aer-${kind}-licences-`));
try {
  if (kind === "linux") {
    execFileSync("dpkg-deb", ["-x", artifacts[0], extraction], { stdio: "inherit" });
  } else {
    execFileSync("7z", ["x", "-y", `-o${extraction}`, artifacts[0]], { stdio: "inherit" });
  }
  const files = walk(extraction);
  assertSame(findOne(files, "/licenses/license"), "LICENSE");
  assertSame(findOne(files, "/licenses/notice"), "NOTICE");
  assertSame(
    findOne(files, "/licenses/third-party-licenses.txt"),
    "THIRD-PARTY-LICENSES.txt",
  );
  assertSame(
    findOne(files, "/licenses/third-party-sources.txt"),
    "THIRD-PARTY-SOURCES.txt",
  );
  if (kind === "linux") {
    assertSame(
      findOne(files, "/usr/share/doc/ai-engine-room/copyright"),
      "packaging/debian/copyright",
    );
    assertSame(findOne(files, "/usr/share/doc/ai-engine-room/notice"), "NOTICE");
    assertSame(
      findOne(files, "/usr/share/doc/ai-engine-room/third-party-sources.txt"),
      "THIRD-PARTY-SOURCES.txt",
    );
  } else {
    const manifest = JSON.parse(readFileSync("third-party/generated-manifest.json", "utf8"));
    const payloadFiles = new Map();
    for (const payload of Object.keys(manifest.installerPayloads)) {
      const file = findOne(files, `/${basename(payload.replaceAll("\\", "/"))}`);
      payloadFiles.set(payload, file);
    }
    for (const [payload, expectedHash] of Object.entries(manifest.installerPayloadSha256)) {
      assert.equal(
        sha256(readFileSync(payloadFiles.get(payload))),
        expectedHash,
        `packaged installer payload differs from pinned source: ${payload}`,
      );
    }
  }
} finally {
  rmSync(extraction, { recursive: true, force: true });
}
console.log(`${kind} package licence payload verified: ${artifacts[0]}`);
