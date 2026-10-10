import { useEffect, useRef } from "react";
import { contributions } from "@/core/registry/contributions";
import type { PanelTab } from "@/core/registry/types";
import { openTasks, useTasks } from "@/shared/hooks/useTasks";
import { greeting, todayTasks } from "@/shared/lib/today";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/shared/components/ui/tabs";
import { HealthTab } from "./HealthTab";
import { ProgressRow } from "./ProgressRow";
import { TodayTab } from "./TodayTab";

const tabs = contributions((manifest) => manifest.panelTabs, [
  { id: "today", label: "Today", order: 10, Component: TodayTab },
  { id: "health", label: "Health", order: 20, Component: HealthTab },
] satisfies PanelTab[]);

type CompactPanelProps = {
  onClose: () => void;
  onResize: () => void;
};

export function CompactPanel({ onClose, onResize }: CompactPanelProps) {
  const panel = useRef<HTMLDivElement>(null);
  const { data: tasks = [] } = useTasks(openTasks);
  const { topThree, due } = todayTasks(tasks);

  useEffect(() => {
    const element = panel.current;
    if (!element) return;
    const observer = new ResizeObserver(onResize);
    observer.observe(element);
    return () => observer.disconnect();
  }, [onResize]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div
      ref={panel}
      data-hit-area
      role="dialog"
      aria-label="Itqan"
      className="orb-popover absolute flex w-80 flex-col gap-3 rounded-2xl bg-card p-4 text-card-foreground shadow-xl ring-1 ring-border"
    >
      <header>
        <p className="font-heading text-lg font-semibold">{greeting()}</p>
        <p className="text-xs text-caption">
          {topThree.length + due.length === 0
            ? "Nothing due today. Add your top 3."
            : `${topThree.length + due.length} left for today`}
        </p>
      </header>

      <ProgressRow />

      <Tabs defaultValue="today">
        <TabsList className="w-full">
          {tabs.map(({ id, label }) => (
            <TabsTrigger key={id} value={id}>
              {label}
            </TabsTrigger>
          ))}
        </TabsList>
        {tabs.map(({ id, Component }) => (
          <TabsContent key={id} value={id} className="pt-2">
            <Component />
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}
