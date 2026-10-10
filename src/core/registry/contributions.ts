import { modules } from "@/modules";
import type { Contribution, ModuleManifest } from "./types";

export function contributions<T extends Contribution>(
  pick: (manifest: ModuleManifest) => T[] | undefined,
  core: T[],
  manifests: ModuleManifest[],
): T[] {
  const merged = [
    ...core,
    ...manifests.flatMap((manifest) => pick(manifest) ?? []),
  ].sort((a, b) => a.order - b.order);
  const ids = new Set(merged.map((item) => item.id));
  if (ids.size !== merged.length) {
    throw new Error(
      `Duplicate contribution ids: ${merged.map((item) => item.id).join(", ")}`,
    );
  }
  return merged;
}

export function useContributions<T extends Contribution>(
  pick: (manifest: ModuleManifest) => T[] | undefined,
  core: T[],
): T[] {
  return contributions(pick, core, modules);
}
