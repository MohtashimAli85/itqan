import { useContributions } from "./contributions";
import type { Surface } from "./types";
import { useModulesSync } from "./useModules";

export function Listeners({ surface }: { surface: Surface }) {
  useModulesSync();
  const listeners = useContributions((manifest) => manifest.listeners, []);
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
