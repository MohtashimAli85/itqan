import { Flame } from "lucide-react";
import { Progress } from "@/shared/components/ui/progress";
import { useProgress } from "../hooks/useProgress";

export function ProgressRow() {
  const { data: progress } = useProgress();
  if (!progress) return null;
  const { level, streak } = progress;

  return (
    <div className="flex items-center gap-3">
      <span className="font-mono text-xs text-caption">L{level.level}</span>
      <Progress
        value={level.xpIntoLevel}
        max={level.xpForNext}
        aria-label="Experience to the next level"
        className="h-1.5 flex-1 [&_[data-slot=progress-indicator]]:bg-brand [&_[data-slot=progress-indicator]]:motion-reduce:transition-none"
      />
      <span
        className="flex items-center gap-1 font-mono text-xs text-caption"
        aria-label={`${streak.current} day streak`}
      >
        <Flame aria-hidden className="size-3.5 text-amber" />
        {streak.current}
      </span>
    </div>
  );
}
