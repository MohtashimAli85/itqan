import { useContributions } from "./contributions";
import type { Surface } from "./types";

export function Listeners({ surface }: { surface: Surface }) {
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
