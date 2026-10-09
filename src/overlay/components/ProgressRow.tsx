import { Flame } from "lucide-react";
import { useProgress } from "@/shared/hooks/useProgress";

export function ProgressRow() {
  const { data: progress } = useProgress();
  if (!progress) return null;
  const { level, streak } = progress;

  return (
    <div className="flex items-center gap-3">
      <span className="text-caption font-mono text-xs">L{level.level}</span>
      <div
        role="progressbar"
        aria-label="Experience to the next level"
        aria-valuemin={0}
        aria-valuemax={level.xpForNext}
        aria-valuenow={level.xpIntoLevel}
        className="bg-muted h-1.5 flex-1 overflow-hidden rounded-full"
      >
        <div
          className="bg-brand h-full origin-left rounded-full transition-transform duration-500 motion-reduce:transition-none"
          style={{
            transform: `scaleX(${level.xpIntoLevel / level.xpForNext})`,
          }}
        />
      </div>
      <span
        className="text-caption flex items-center gap-1 font-mono text-xs"
        aria-label={`${streak.current} day streak`}
      >
        <Flame aria-hidden className="text-amber size-3.5" />
        {streak.current}
      </span>
    </div>
  );
}
