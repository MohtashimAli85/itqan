import { useEffect, useRef } from "react";

type Listenable<T> = {
  listen: (handler: (event: { payload: T }) => void) => Promise<() => void>;
};

export function useTauriEvent<T>(
  event: Listenable<T>,
  handler: (payload: T) => void,
) {
  const latest = useRef(handler);
  useEffect(() => {
    latest.current = handler;
  });

  useEffect(() => {
    let active = true;
    let stop: (() => void) | undefined;
    void event
      .listen((message) => latest.current(message.payload))
      .then((unlisten) => {
        if (active) {
          stop = unlisten;
        } else {
          unlisten();
        }
      });
    return () => {
      active = false;
      stop?.();
    };
  }, [event]);
}
