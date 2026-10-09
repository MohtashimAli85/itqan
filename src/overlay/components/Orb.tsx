import { useRef, type Ref } from "react";
import type { OrbState } from "@/shared/bindings/bindings";
import { ORB_SIZE } from "../follower";
import { useOrbAnimations } from "../hooks/useOrbAnimations";
import { Eyes } from "./Eyes";
import { ProgressRing } from "./ProgressRing";

type OrbProps = {
  state: OrbState;
  progress: number | null;
  bodyRef: Ref<HTMLDivElement>;
  pupilRef: (index: number) => (element: HTMLSpanElement | null) => void;
  onClick: () => void;
};

export function Orb({ state, progress, bodyRef, pupilRef, onClick }: OrbProps) {
  const surface = useRef<HTMLButtonElement>(null);
  const eyes = useRef<HTMLSpanElement>(null);
  const z = useRef<HTMLSpanElement>(null);
  useOrbAnimations(state, { body: surface, eyes, z });

  return (
    <div
      ref={bodyRef}
      className="orb relative will-change-transform"
      data-state={state}
      style={{ width: ORB_SIZE, height: ORB_SIZE }}
    >
      {progress !== null && <ProgressRing progress={progress} />}
      {state === "listening" && (
        <span
          aria-hidden
          className="orb-surface absolute inset-0 animate-ping rounded-full opacity-40 motion-reduce:animate-none"
        />
      )}
      <button
        ref={surface}
        type="button"
        data-hit-area
        aria-label={`Itqan, ${state}`}
        onClick={onClick}
        className="orb-surface focus-visible:ring-ring relative flex size-full items-center justify-center rounded-full focus-visible:ring-2 focus-visible:outline-none"
      >
        <span ref={eyes} className="flex items-center gap-2">
          <Eyes state={state} pupilRef={pupilRef} />
        </span>
      </button>
      {state === "resting" && (
        <span
          ref={z}
          aria-hidden
          className="font-heading text-orb-resting pointer-events-none absolute -top-2 right-0 text-sm font-bold opacity-0"
        >
          z
        </span>
      )}
    </div>
  );
}
