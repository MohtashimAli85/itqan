import type { Task } from "@/shared/bindings/bindings";
import { useCategories } from "@/modules/tasks/hooks/useTasks";
import { TaskItem } from "./TaskItem";

type TaskSectionProps = {
  title: string;
  tasks: Task[];
  empty?: string;
};

export function TaskSection({ title, tasks, empty }: TaskSectionProps) {
  const { data: categories = [] } = useCategories();
  const id = `section-${title.toLowerCase().replace(/\W+/g, "-")}`;
  if (tasks.length === 0 && !empty) return null;

  return (
    <section aria-labelledby={id} className="flex flex-col gap-1">
      <h2
        id={id}
        className="px-3 text-xs font-medium tracking-wide text-caption uppercase"
      >
        {title}
      </h2>
      {tasks.length === 0 ? (
        <p className="px-3 py-2 text-sm text-caption">{empty}</p>
      ) : (
        <ul>
          {tasks.map((task) => (
            <TaskItem
              key={task.id}
              task={task}
              category={categories.find((c) => c.id === task.categoryId)}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
