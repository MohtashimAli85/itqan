import { Link } from "@tanstack/react-router";
import {
  CalendarDays,
  Settings,
  Sun,
  Target,
  TrendingUp,
  type LucideIcon,
} from "lucide-react";
import { openTasks, useCategories, useTasks } from "@/shared/hooks/useTasks";
import { categoryDot, todaySections } from "@/shared/lib/taskGroups";
import { useContributions } from "@/core/registry/contributions";
import type { Contribution } from "@/core/registry/types";
import { NextPrayerCard } from "./NextPrayerCard";

const linkClass =
  "flex items-center gap-2.5 rounded-lg px-3 py-1.5 text-sm text-text-secondary hover:bg-muted hover:text-foreground data-[status=active]:bg-muted data-[status=active]:font-medium data-[status=active]:text-foreground";

function NavLink({
  to,
  icon: Icon,
  label,
  count,
}: {
  to: "/" | "/upcoming" | "/goals" | "/progress" | "/settings";
  icon: LucideIcon;
  label: string;
  count?: number;
}) {
  return (
    <Link to={to} className={linkClass} activeOptions={{ exact: true }}>
      <Icon aria-hidden className="size-4" />
      <span className="flex-1">{label}</span>
      {count !== undefined && count > 0 && (
        <span className="font-mono text-xs text-caption">{count}</span>
      )}
    </Link>
  );
}

const coreWidgets: Contribution[] = [
  { id: "next-prayer", order: 10, Component: NextPrayerCard },
];

export function Sidebar() {
  const widgets = useContributions(
    (manifest) => manifest.sidebarWidgets,
    coreWidgets,
  );
  const { data: tasks = [] } = useTasks(openTasks);
  const { data: categories = [] } = useCategories();
  const sections = todaySections(tasks);
  const todayCount = sections.topThree.length + sections.today.length;

  return (
    <nav
      aria-label="Main"
      className="flex w-56 shrink-0 flex-col gap-6 border-r border-sidebar-border bg-sidebar p-4"
    >
      <p className="px-3 font-heading text-lg font-bold">Itqan</p>
      <div className="flex flex-col gap-0.5">
        <NavLink to="/" icon={Sun} label="Today" count={todayCount} />
        <NavLink to="/upcoming" icon={CalendarDays} label="Upcoming" />
      </div>
      <div className="flex flex-col gap-0.5">
        <p className="px-3 pb-1 text-xs font-medium text-caption">Areas</p>
        {categories.map((category) => (
          <Link
            key={category.id}
            to="/category/$categoryId"
            params={{ categoryId: String(category.id) }}
            className={linkClass}
          >
            <span
              aria-hidden
              className={`size-2 rounded-full ${categoryDot(category.colour)}`}
            />
            {category.name}
          </Link>
        ))}
      </div>
      <div className="flex flex-col gap-0.5">
        <NavLink to="/goals" icon={Target} label="Goals" />
        <NavLink to="/progress" icon={TrendingUp} label="Progress" />
        <NavLink to="/settings" icon={Settings} label="Settings" />
      </div>
      <div className="mt-auto flex flex-col gap-3">
        {widgets.map(({ id, Component }) => (
          <Component key={id} />
        ))}
      </div>
    </nav>
  );
}
