import { useParams } from "@tanstack/react-router";
import { useCategories, useTasks } from "@/shared/hooks/useTasks";
import { todaySections, upcomingGroups } from "@/shared/lib/taskGroups";
import { TaskScreen } from "../layout/TaskScreen";
import { TaskSection } from "../tasks/TaskSection";

export function CategoryView() {
  const { categoryId } = useParams({ from: "/app/category/$categoryId" });
  const id = Number(categoryId);
  const { data: categories = [] } = useCategories();
  const { data: tasks = [] } = useTasks({
    status: "open",
    categoryId: id,
    dueBefore: null,
  });
  const category = categories.find((candidate) => candidate.id === id);
  const sections = todaySections(tasks);
  const later = upcomingGroups(tasks).flatMap((group) => group.tasks);

  return (
    <TaskScreen title={category?.name ?? "Area"}>
      <TaskSection title="Top 3" tasks={sections.topThree} />
      <TaskSection title="Today" tasks={sections.today} />
      <TaskSection title="Coming up" tasks={later} />
      <TaskSection
        title="Anytime"
        tasks={sections.anytime}
        empty={
          tasks.length === 0
            ? "No open tasks here. Add one with #name in quick add."
            : undefined
        }
      />
    </TaskScreen>
  );
}
