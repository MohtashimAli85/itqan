import { Flame, Snowflake } from "lucide-react";
import { useProgress } from "@/shared/hooks/useProgress";
import { SkillsPanel } from "../goals/SkillsPanel";
import { BadgeGrid } from "../progress/BadgeGrid";
import { FocusChart } from "../progress/FocusChart";
import { StreakCalendar } from "../progress/StreakCalendar";

export function Progress() {
  const { data: progress } = useProgress();
  if (!progress) return null;
  const { level, streak } = progress;

  return (
    <main className="flex min-w-0 flex-1 flex-col gap-8 overflow-y-auto p-8">
      <h1 className="font-heading text-3xl font-bold">Progress</h1>
      <div className="grid gap-4 md:grid-cols-2">
        <section
          aria-label="Level"
          className="flex flex-col gap-3 rounded-2xl bg-card p-5 ring-1 ring-border"
        >
          <span className="text-sm text-caption">Level</span>
          <span className="font-heading text-5xl font-bold">{level.level}</span>
          <div
            role="progressbar"
            aria-label="Experience to the next level"
            aria-valuemin={0}
            aria-valuemax={level.xpForNext}
            aria-valuenow={level.xpIntoLevel}
            className="h-2 overflow-hidden rounded-full bg-muted"
          >
            <div
              className="h-full origin-left rounded-full bg-brand"
              style={{
                transform: `scaleX(${level.xpIntoLevel / level.xpForNext})`,
              }}
            />
          </div>
          <span className="font-mono text-xs text-caption">
            {level.xpIntoLevel} / {level.xpForNext} XP · {progress.totalXp}{" "}
            total · {progress.todayXp} today
          </span>
        </section>
        <section
          aria-label="Streak"
          className="flex flex-col gap-3 rounded-2xl bg-card p-5 ring-1 ring-border"
        >
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
        </section>
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
