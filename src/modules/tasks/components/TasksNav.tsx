import { Link } from "@tanstack/react-router";
import { CalendarDays, Sun } from "lucide-react";
import { NavLink, navLinkClass } from "@/core/ui/NavLink";
import {
  openTasks,
  useCategories,
  useTasks,
} from "@/modules/tasks/hooks/useTasks";
import { categoryDot, todaySections } from "@/modules/tasks/lib/taskGroups";

export function TasksNav() {
  const { data: tasks = [] } = useTasks(openTasks);
  const sections = todaySections(tasks);
  const todayCount = sections.topThree.length + sections.today.length;

  return (
    <div className="flex flex-col gap-0.5">
      <NavLink to="/" icon={Sun} label="Today" count={todayCount} />
      <NavLink to="/upcoming" icon={CalendarDays} label="Upcoming" />
    </div>
  );
}

export function AreasNav() {
  const { data: categories = [] } = useCategories();

  return (
    <div className="flex flex-col gap-0.5">
      <p className="px-3 pb-1 text-xs font-medium text-caption">Areas</p>
      {categories.map((category) => (
        <Link
          key={category.id}
          to="/category/$categoryId"
          params={{ categoryId: String(category.id) }}
          className={navLinkClass}
        >
          <span
            aria-hidden
            className={`size-2 rounded-full ${categoryDot(category.colour)}`}
          />
          {category.name}
        </Link>
      ))}
    </div>
  );
}
