import { useEffect, useRef } from "react";
import { useContributions } from "@/core/registry/contributions";
import type { PanelTab } from "@/core/registry/types";
import { useTodayTasks } from "@/shared/hooks/useTasks";
import { greeting } from "@/shared/lib/today";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/shared/components/ui/tabs";
import { HealthTab } from "./HealthTab";
import { ProgressRow } from "./ProgressRow";
import { TodayTab } from "./TodayTab";

const coreTabs: PanelTab[] = [
  { id: "today", label: "Today", order: 10, Component: TodayTab },
  { id: "health", label: "Health", order: 20, Component: HealthTab },
];

type CompactPanelProps = {
  onClose: () => void;
  onResize: () => void;
};

export function CompactPanel({ onClose, onResize }: CompactPanelProps) {
  const panel = useRef<HTMLDivElement>(null);
  const { topThree, due } = useTodayTasks();
  const tabs = useContributions((manifest) => manifest.panelTabs, coreTabs);

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

      <Tabs defaultValue={tabs[0]?.id}>
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
