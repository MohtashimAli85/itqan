import { describe, expect, it } from "vitest";
import { contributions } from "./contributions";
import { enabledModules } from "./useModules";
import type { Contribution, ModuleManifest } from "./types";

const section = (id: string, order: number): Contribution => ({
  id,
  order,
  Component: () => null,
});

describe("contributions", () => {
  it("merges core and module items by order, keeping ties stable", () => {
    const manifests: ModuleManifest[] = [
      {
        id: "health",
        name: "Health",
        description: "",
        settingsSections: [section("health", 30)],
      },
      {
        id: "salah",
        name: "Salah",
        description: "",
        settingsSections: [section("salah", 20)],
      },
      { id: "tasks", name: "Tasks", description: "" },
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
        [
          {
            id: "health",
            name: "Health",
            description: "",
            settingsSections: [section("health", 30)],
          },
        ],
      ),
    ).toThrow(/Duplicate/);
  });
});

describe("enabledModules", () => {
  const manifests: ModuleManifest[] = [
    { id: "tasks", name: "Tasks", description: "" },
    { id: "salah", name: "Salah", description: "" },
    { id: "health", name: "Health", description: "" },
  ];

  it("keeps every module until the states load", () => {
    expect(enabledModules(manifests, undefined)).toEqual(manifests);
  });

  it("drops disabled modules and keeps unknown ones", () => {
    const enabled = enabledModules(manifests, [
      { id: "tasks", enabled: true },
      { id: "salah", enabled: false },
    ]);
    expect(enabled.map((manifest) => manifest.id)).toEqual(["tasks", "health"]);
  });
});
