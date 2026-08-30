import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const manifest = JSON.parse(readFileSync("third-party/generated-manifest.json", "utf8"));
const covered = new Set(
  manifest.components.map((component) => `${component.name}@${component.version}`),
);
const target =
  process.platform === "win32"
    ? "x86_64-pc-windows-msvc"
    : process.platform === "linux"
      ? "x86_64-unknown-linux-gnu"
      : undefined;
assert.ok(target, `unsupported licence-verification host: ${process.platform}`);
const tree = execFileSync(
  "cargo",
  [
    "tree",
    "--workspace",
    "--locked",
    "--target",
    target,
    "--edges",
    "normal",
    "--prefix",
    "none",
    "--format",
    "{p}",
  ],
  { encoding: "utf8" },
);
const localPackages = new Set(["aer-core", "aiengineroom"]);
const targetPackages = new Set();
for (const line of tree.split(/\r?\n/)) {
  const match = line.replace(/ \(\*\)$/, "").match(/^([^ ]+) v([^ ]+)/);
  if (!match || localPackages.has(match[1])) continue;
  targetPackages.add(`${match[1]}@${match[2]}`);
}
assert.ok(targetPackages.size > 200, "unexpectedly small target dependency graph");
for (const component of targetPackages) {
  assert.ok(covered.has(component), `target dependency lacks publication material: ${component}`);
}
console.log(
  `Target licence coverage verified for ${targetPackages.size} registry components on ${target}.`,
);
