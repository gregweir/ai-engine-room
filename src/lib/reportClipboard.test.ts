import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { writeText } = vi.hoisted(() => ({
  writeText: vi.fn<(text: string) => Promise<void>>(),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText }));

import { createNativeReportClipboard } from "./reportClipboard";

const repositoryRoot = resolve(import.meta.dirname, "../..");

describe("native report clipboard boundary", () => {
  beforeEach(() => {
    writeText.mockReset();
    writeText.mockResolvedValue(undefined);
  });

  it("delegates the exact report text to the official plain-text writer", async () => {
    const clipboard = await createNativeReportClipboard();
    const preview =
      "AI Engine Room — Observation Report\n\nObservation\nAvailable memory\nValue: 6.00 GB\n";

    await clipboard.copyText(preview);

    expect(writeText).toHaveBeenCalledTimes(1);
    expect(writeText).toHaveBeenCalledWith(preview);
    expect(writeText.mock.calls[0][0]).toMatch(/\n$/);
  });

  it("contains no browser fallback or broader clipboard operation", () => {
    const source = readFileSync(
      resolve(repositoryRoot, "src/lib/reportClipboard.ts"),
      "utf8",
    );

    expect(source).not.toContain("navigator.clipboard");
    expect(source).not.toMatch(/\breadText\b/);
    expect(source).not.toMatch(/\bclear\b/);
    expect(source).not.toMatch(/\bwriteHtml\b|\bwriteImage\b|\bwriteFiles\b/);
  });
});

describe("native clipboard capability contract", () => {
  it("initializes the official plugin exactly once", () => {
    const source = readFileSync(
      resolve(repositoryRoot, "src-tauri/src/lib.rs"),
      "utf8",
    );

    expect(
      source.match(/tauri_plugin_clipboard_manager::init\(\)/g) ?? [],
    ).toHaveLength(1);
    expect(source.match(/\.plugin\(/g) ?? []).toHaveLength(1);
  });

  it("adds no unrelated Tauri plugin package", () => {
    const cargoManifest = readFileSync(
      resolve(repositoryRoot, "src-tauri/Cargo.toml"),
      "utf8",
    );
    const packageManifest = JSON.parse(
      readFileSync(resolve(repositoryRoot, "package.json"), "utf8"),
    ) as { dependencies: Record<string, string> };

    expect(cargoManifest.match(/^tauri-plugin-[\w-]+\s*=.*$/gm) ?? []).toEqual([
      'tauri-plugin-clipboard-manager = "2"',
    ]);
    expect(
      Object.keys(packageManifest.dependencies).filter((name) =>
        name.startsWith("@tauri-apps/plugin-"),
      ),
    ).toEqual(["@tauri-apps/plugin-clipboard-manager"]);
  });

  it("grants only local plain-text clipboard writes", () => {
    const capability = JSON.parse(
      readFileSync(
        resolve(repositoryRoot, "src-tauri/capabilities/default.json"),
        "utf8",
      ),
    ) as { permissions: string[]; remote?: unknown };

    expect(capability.permissions).toEqual([
      "core:default",
      "clipboard-manager:allow-write-text",
    ]);
    expect(capability).not.toHaveProperty("remote");

    const serialized = JSON.stringify(capability.permissions);
    expect(serialized).not.toMatch(
      /clipboard-manager:(?:default|allow-read|allow-clear|allow-write-html|allow-write-image)/,
    );
    expect(serialized).not.toMatch(
      /(?:dialog|fs|filesystem|shell|opener|http|network):/,
    );
  });
});
