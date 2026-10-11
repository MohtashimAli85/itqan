import { useMemo, useRef, useState, type KeyboardEvent } from "react";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { cn } from "cn";
import { dayjs, type Dayjs } from "@/shared/lib/dayjs";
import { Button } from "./button";

const weekdayLabels = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

type CalendarProps = {
  value?: string;
  onChange: (value: string) => void;
  min?: string;
  max?: string;
  className?: string;
  "aria-label"?: string;
};

export function Calendar({
  value,
  onChange,
  min,
  max,
  className,
  "aria-label": ariaLabel = "Calendar",
}: CalendarProps) {
  const grid = useRef<HTMLDivElement>(null);
  const selected = value ? dayjs(value) : null;
  const [month, setMonth] = useState<Dayjs>(() =>
    (selected ?? dayjs()).startOf("month"),
  );
  const [focus, setFocus] = useState<Dayjs>(() => selected ?? dayjs());

  const isOut = (day: Dayjs) =>
    (min !== undefined && day.isBefore(dayjs(min), "day")) ||
    (max !== undefined && day.isAfter(dayjs(max), "day"));

  const weeks = useMemo(() => {
    const start = month.startOf("month");
    const gridStart = start.subtract(start.day(), "day");
    const days = Array.from({ length: 42 }, (_, index) =>
      gridStart.add(index, "day"),
    );
    return Array.from({ length: 6 }, (_, week) =>
      days.slice(week * 7, week * 7 + 7),
    );
  }, [month]);

  function move(to: Dayjs) {
    setFocus(to);
    setMonth(to.startOf("month"));
    grid.current
      ?.querySelector<HTMLButtonElement>(
        `[data-day="${to.format("YYYY-MM-DD")}"]`,
      )
      ?.focus();
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const targets: Record<string, Dayjs> = {
      ArrowLeft: focus.subtract(1, "day"),
      ArrowRight: focus.add(1, "day"),
      ArrowUp: focus.subtract(7, "day"),
      ArrowDown: focus.add(7, "day"),
      Home: focus.subtract(focus.day(), "day"),
      End: focus.add(6 - focus.day(), "day"),
      PageUp: focus.subtract(1, "month"),
      PageDown: focus.add(1, "month"),
    };
    const to = targets[event.key];
    if (!to || isOut(to)) return;
    event.preventDefault();
    move(to);
  }

  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <div className="flex items-center justify-between">
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          aria-label="Previous month"
          onClick={() => setMonth(month.subtract(1, "month"))}
        >
          <ChevronLeftIcon />
        </Button>
        <span aria-live="polite" className="text-sm font-medium">
          {month.format("MMMM YYYY")}
        </span>
        <Button
          type="button"
          variant="ghost"
          size="icon-sm"
          aria-label="Next month"
          onClick={() => setMonth(month.add(1, "month"))}
        >
          <ChevronRightIcon />
        </Button>
      </div>
      <div
        role="grid"
        aria-label={ariaLabel}
        ref={grid}
        tabIndex={-1}
        onKeyDown={onKeyDown}
      >
        <div role="row" className="grid grid-cols-7">
          {weekdayLabels.map((label) => (
            <span
              key={label}
              role="columnheader"
              className="flex size-8 items-center justify-center text-xs text-muted-foreground"
            >
              {label}
            </span>
          ))}
        </div>
        {weeks.map((week) => (
          <div
            role="row"
            key={week[0]?.format("YYYY-MM-DD")}
            className="grid grid-cols-7"
          >
            {week.map((day) => {
              const iso = day.format("YYYY-MM-DD");
              const isSelected = selected?.isSame(day, "day") ?? false;
              const isToday = day.isSame(dayjs(), "day");
              const outside = !day.isSame(month, "month");
              return (
                <button
                  key={iso}
                  type="button"
                  role="gridcell"
                  data-day={iso}
                  aria-label={day.format("D MMMM YYYY")}
                  aria-selected={isSelected}
                  aria-current={isToday ? "date" : undefined}
                  disabled={isOut(day)}
                  tabIndex={focus.isSame(day, "day") ? 0 : -1}
                  onFocus={() => setFocus(day)}
                  onClick={() => onChange(iso)}
                  className={cn(
                    "flex size-8 items-center justify-center rounded-md text-sm transition-colors outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-40",
                    outside && "text-muted-foreground",
                    isToday && !isSelected && "ring-1 ring-ring",
                    isSelected &&
                      "bg-primary text-primary-foreground hover:bg-primary",
                  )}
                >
                  {day.date()}
                </button>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
