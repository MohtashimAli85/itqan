import { useEffect, useRef, useState } from "react";
import { commands, events } from "@/shared/bindings/bindings";
import { createFollower, ORB_SIZE, type Point } from "./follower";

const reducedMotionQuery = window.matchMedia(
  "(prefers-reduced-motion: reduce)",
);

export function Orb() {
  const orbRef = useRef<HTMLButtonElement>(null);
  const pupilsRef = useRef<HTMLSpanElement[]>([]);
  const [happy, setHappy] = useState(false);

  useEffect(() => {
    const render = (position: Point, pupils: Point) => {
      if (orbRef.current) {
        orbRef.current.style.transform = `translate3d(${position.x}px, ${position.y}px, 0)`;
      }
      for (const pupil of pupilsRef.current) {
        pupil.style.transform = `translate(${pupils.x}px, ${pupils.y}px)`;
      }
    };
    const follower = createFollower({
      render,
      onMove: () => void commands.setOverlayHitAreas([]),
      onSettle: (rect) => void commands.setOverlayHitAreas([rect]),
      reducedMotion: () => reducedMotionQuery.matches,
    });
    const unlisten = events.overlayCursor.listen((event) =>
      follower.setCursor(event.payload),
    );
    return () => {
      follower.stop();
      void unlisten.then((stop) => stop());
    };
  }, []);

  return (
    <button
      ref={orbRef}
      type="button"
      aria-label="Itqan"
      aria-pressed={happy}
      onClick={() => setHappy((value) => !value)}
      className="bg-brand focus-visible:ring-ring fixed top-0 left-0 flex items-center justify-center gap-2 rounded-full shadow-lg will-change-transform focus-visible:ring-2 focus-visible:outline-none"
      style={{ width: ORB_SIZE, height: ORB_SIZE }}
    >
      {[0, 1].map((index) => (
        <span
          key={index}
          className={
            happy
              ? "border-orb-eye h-2 w-3.5 rounded-t-full border-t-[3px]"
              : "bg-orb-eye flex h-4 w-3.5 items-center justify-center rounded-full"
          }
        >
          {!happy && (
            <span
              ref={(element) => {
                if (element) pupilsRef.current[index] = element;
              }}
              className="bg-orb-pupil size-2 rounded-full"
            />
          )}
        </span>
      ))}
    </button>
  );
}
