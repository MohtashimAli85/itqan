import { openTasks, useTasks } from "@/shared/hooks/useTasks";
import { upcomingGroups } from "@/shared/lib/taskGroups";
import { TaskScreen } from "../layout/TaskScreen";
import { TaskSection } from "../tasks/TaskSection";

export function Upcoming() {
  const { data: tasks = [] } = useTasks(openTasks);
  const groups = upcomingGroups(tasks);

  return (
    <TaskScreen title="Upcoming" subtitle="The next two weeks">
      {groups.length === 0 ? (
        <p className="text-caption text-sm">Nothing scheduled yet.</p>
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
