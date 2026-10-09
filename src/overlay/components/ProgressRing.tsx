import { ORB_SIZE } from "../follower";

const STROKE = 3;
const GAP = 4;
const SIZE = ORB_SIZE + (STROKE + GAP) * 2;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ProgressRing({ progress }: { progress: number }) {
  return (
    <svg
      aria-hidden
      width={SIZE}
      height={SIZE}
      className="pointer-events-none absolute -rotate-90"
      style={{ left: -(STROKE + GAP), top: -(STROKE + GAP) }}
    >
      <circle
        cx={SIZE / 2}
        cy={SIZE / 2}
        r={RADIUS}
        fill="none"
        strokeWidth={STROKE}
        className="orb-ring opacity-20"
      />
      <circle
        cx={SIZE / 2}
        cy={SIZE / 2}
        r={RADIUS}
        fill="none"
        strokeWidth={STROKE}
        strokeLinecap="round"
        strokeDasharray={CIRCUMFERENCE}
        strokeDashoffset={CIRCUMFERENCE * (1 - progress)}
        className="orb-ring transition-[stroke-dashoffset] duration-500 motion-reduce:transition-none"
      />
    </svg>
  );
}
