import { useContributions } from "./contributions";
import type { Surface } from "./types";
import { useModuleStates, useModulesSync } from "./useModules";

export function Listeners({ surface }: { surface: Surface }) {
  useModulesSync();
  const { isPending } = useModuleStates();
  const listeners = useContributions((manifest) => manifest.listeners, []);
  if (isPending) return null;
  return (
    <>
      {listeners
        .filter((listener) => (listener.surface ?? surface) === surface)
        .map(({ id, Component }) => (
          <Component key={id} />
        ))}
    </>
  );
}
