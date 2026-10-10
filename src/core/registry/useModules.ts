import { useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events } from "@/shared/bindings/bindings";
import { useTauriEvent } from "@/shared/hooks/useTauriEvent";
import { unwrap } from "@/shared/lib/result";
import { modules } from "@/modules";
import type { ModuleId, ModuleManifest } from "./types";

export const modulesKey = ["modules"] as const;

export function useModuleStates() {
  return useQuery({
    queryKey: modulesKey,
    queryFn: async () => unwrap(await commands.listModules()),
  });
}

export function useModulesSync() {
  const queryClient = useQueryClient();
  useTauriEvent(events.modulesChanged, () => {
    void queryClient.invalidateQueries({ queryKey: modulesKey });
  });
}

export function enabledModules(
  manifests: ModuleManifest[],
  states: { id: string; enabled: boolean }[] | undefined,
): ModuleManifest[] {
  if (!states) return manifests;
  return manifests.filter(
    (manifest) =>
      states.find((state) => state.id === manifest.id)?.enabled ?? true,
  );
}

export function useEnabledModules(): ModuleManifest[] {
  const { data } = useModuleStates();
  return enabledModules(modules, data);
}

export function useModuleEnabled(id: ModuleId): boolean {
  const { data } = useModuleStates();
  return data?.find((state) => state.id === id)?.enabled ?? true;
}
