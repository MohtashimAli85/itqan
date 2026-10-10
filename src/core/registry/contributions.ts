import { modules } from "@/modules";
import type { ModuleManifest } from "./types";

type Ordered = { order: number };

export function contributions<T extends Ordered>(
  pick: (manifest: ModuleManifest) => T[] | undefined,
  core: T[] = [],
  manifests: ModuleManifest[] = modules,
): T[] {
  return [...core, ...manifests.flatMap((manifest) => pick(manifest) ?? [])]
    .map((item, index) => ({ item, index }))
    .sort((a, b) => a.item.order - b.item.order || a.index - b.index)
    .map(({ item }) => item);
}
