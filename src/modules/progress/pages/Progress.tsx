import { Flame, Snowflake } from "lucide-react";
import { SkillsPanel } from "@/core/ui/SkillsPanel";
import { Card, CardContent } from "@/shared/components/ui/card";
import { Progress as ProgressBar } from "@/shared/components/ui/progress";
import { useProgress } from "../hooks/useProgress";
import { BadgeGrid } from "../components/BadgeGrid";
import { FocusChart } from "../components/FocusChart";
import { StreakCalendar } from "../components/StreakCalendar";

export function Progress() {
  const { data: progress } = useProgress();
  if (!progress) return null;
  const { level, streak } = progress;

  return (
    <main className="flex min-w-0 flex-1 flex-col gap-8 overflow-y-auto p-8">
      <h1 className="font-heading text-3xl font-bold">Progress</h1>
      <div className="grid gap-4 md:grid-cols-2">
        <Card
          role="region"
          aria-label="Level"
          className="rounded-2xl ring-border [--card-spacing:--spacing(5)]"
        >
          <CardContent className="flex flex-col gap-3">
            <span className="text-sm text-caption">Level</span>
            <span className="font-heading text-5xl font-bold">
              {level.level}
            </span>
            <ProgressBar
              value={level.xpIntoLevel}
              max={level.xpForNext}
              aria-label="Experience to the next level"
              className="h-2 [&_[data-slot=progress-indicator]]:bg-brand"
            />
            <span className="font-mono text-xs text-caption">
              {level.xpIntoLevel} / {level.xpForNext} XP · {progress.totalXp}{" "}
              total · {progress.todayXp} today
            </span>
          </CardContent>
        </Card>
        <Card
          role="region"
          aria-label="Streak"
          className="rounded-2xl ring-border [--card-spacing:--spacing(5)]"
        >
          <CardContent className="flex flex-col gap-3">
            <span className="text-sm text-caption">Streak</span>
            <span className="flex items-center gap-2 font-heading text-5xl font-bold">
              <Flame aria-hidden className="size-9 text-amber" />
              {streak.current}
            </span>
            <span className="flex items-center gap-3 text-xs text-caption">
              <span>Best {streak.best}</span>
              <span className="flex items-center gap-1">
                <Snowflake aria-hidden className="size-3.5 text-work" />
                {streak.freezes} freeze{streak.freezes === 1 ? "" : "s"}
              </span>
            </span>
          </CardContent>
        </Card>
      </div>
      <div className="grid gap-8 md:grid-cols-2">
        <FocusChart days={progress.days} />
        <StreakCalendar days={progress.days} />
      </div>
      <BadgeGrid badges={progress.badges} />
      <div className="max-w-md">
        <SkillsPanel />
      </div>
    </main>
  );
}
