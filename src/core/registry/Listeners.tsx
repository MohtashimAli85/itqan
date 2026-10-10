import { useContributions } from "./contributions";

export function Listeners() {
  const listeners = useContributions((manifest) => manifest.listeners, []);
  return (
    <>
      {listeners.map(({ id, Component }) => (
        <Component key={id} />
      ))}
    </>
  );
}
