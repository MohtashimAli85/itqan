import { openTasks, useTasks } from "@/modules/tasks/hooks/useTasks";
import { upcomingGroups } from "@/modules/tasks/lib/taskGroups";
import { TaskScreen } from "@/modules/tasks/components/TaskScreen";
import { TaskSection } from "@/modules/tasks/components/TaskSection";

export function Upcoming() {
  const { data: tasks = [] } = useTasks(openTasks);
  const groups = upcomingGroups(tasks);

  return (
    <TaskScreen title="Upcoming" subtitle="The next two weeks">
      {groups.length === 0 ? (
        <p className="text-sm text-caption">Nothing scheduled yet.</p>
      ) : (
        groups.map((group) => (
          <TaskSection
            key={group.key}
            title={group.label}
            tasks={group.tasks}
          />
        ))
      )}
    </TaskScreen>
  );
}
