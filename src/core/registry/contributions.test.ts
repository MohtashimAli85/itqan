import { describe, expect, it } from "vitest";
import { contributions } from "./contributions";
import type { Contribution, ModuleManifest } from "./types";

const section = (id: string, order: number): Contribution => ({
  id,
  order,
  Component: () => null,
});

describe("contributions", () => {
  it("merges core and module items by order, keeping ties stable", () => {
    const manifests: ModuleManifest[] = [
      { id: "health", settingsSections: [section("health", 30)] },
      { id: "salah", settingsSections: [section("salah", 20)] },
      { id: "tasks" },
    ];

    const merged = contributions(
      (manifest) => manifest.settingsSections,
      [section("orb", 10), section("ai", 40), section("nudges", 20)],
      manifests,
    );

    expect(merged.map((item) => item.id)).toEqual([
      "orb",
      "nudges",
      "salah",
      "health",
      "ai",
    ]);
  });

  it("rejects duplicate ids", () => {
    expect(() =>
      contributions(
        (manifest) => manifest.settingsSections,
        [section("health", 30)],
        [{ id: "health", settingsSections: [section("health", 30)] }],
      ),
    ).toThrow(/Duplicate/);
  });
});
