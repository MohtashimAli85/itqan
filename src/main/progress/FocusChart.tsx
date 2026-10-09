import { Bar, BarChart, XAxis, YAxis } from "recharts";
import type { DaySummary } from "@/shared/bindings/bindings";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/shared/components/ui/chart";
import { dayjs } from "@/shared/lib/dayjs";

const config = {
  hours: { label: "Focus hours", color: "var(--personal)" },
} satisfies ChartConfig;

export function FocusChart({ days }: { days: DaySummary[] }) {
  const data = days.slice(-7).map((day) => ({
    day: dayjs(day.date).format("ddd"),
    hours: Math.round((day.focusMinutes / 60) * 10) / 10,
  }));
  const total = data.reduce((sum, day) => sum + day.hours, 0);

  return (
    <figure className="flex flex-col gap-3">
      <figcaption className="flex items-baseline justify-between">
        <span className="font-heading text-lg font-semibold">
          Focus this week
        </span>
        <span className="text-caption font-mono text-sm">
          {total.toFixed(1)} h
        </span>
      </figcaption>
      <ChartContainer config={config} className="h-44 w-full">
        <BarChart data={data} accessibilityLayer>
          <XAxis dataKey="day" tickLine={false} axisLine={false} />
          <YAxis
            width={28}
            tickLine={false}
            axisLine={false}
            allowDecimals={false}
          />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Bar dataKey="hours" fill="var(--color-hours)" radius={6} />
        </BarChart>
      </ChartContainer>
    </figure>
  );
}
