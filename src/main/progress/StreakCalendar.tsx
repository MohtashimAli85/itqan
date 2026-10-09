import type { DaySummary } from "@/shared/bindings/bindings";
import { dayjs } from "@/shared/lib/dayjs";

export function StreakCalendar({ days }: { days: DaySummary[] }) {
  const weeks: DaySummary[][] = [];
  for (let index = 0; index < days.length; index += 7) {
    weeks.push(days.slice(index, index + 7));
  }

  return (
    <figure className="flex flex-col gap-3">
      <figcaption className="font-heading text-lg font-semibold">
        Last five weeks
      </figcaption>
      <div
        role="grid"
        aria-label="Active days"
        className="flex flex-col gap-1.5"
      >
        {weeks.map((week) => (
          <div role="row" key={week[0]?.date} className="flex gap-1.5">
            {week.map((day) => (
              <div
                role="gridcell"
                key={day.date}
                aria-label={`${dayjs(day.date).format("D MMMM")}: ${day.active ? "active" : "no activity"}`}
                title={`${dayjs(day.date).format("ddd D MMM")} · ${day.xp} XP`}
                className={
                  day.active
                    ? "bg-brand size-7 rounded-md"
                    : "bg-muted size-7 rounded-md"
                }
              />
            ))}
          </div>
        ))}
      </div>
      <p className="text-caption text-xs">
        A missed day leaves a gap, never wipes your progress.
      </p>
    </figure>
  );
}
