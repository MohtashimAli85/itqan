import { allTasks, useTasks } from "@/shared/hooks/useTasks";
import { dayjs } from "@/shared/lib/dayjs";
import { doneToday, todaySections } from "@/shared/lib/taskGroups";
import { greeting } from "@/shared/lib/today";
import { TaskScreen } from "../layout/TaskScreen";
import { QuickAddBar } from "../tasks/QuickAddBar";
import { TaskSection } from "../tasks/TaskSection";

export function Today() {
  const { data: tasks = [] } = useTasks(allTasks);
  const sections = todaySections(tasks);
  const done = doneToday(tasks);

  return (
    <TaskScreen title={greeting()} subtitle={dayjs().format("dddd, D MMMM")}>
      <QuickAddBar />
      <TaskSection
        title="Top 3"
        tasks={sections.topThree}
        empty="Star up to three tasks that would make today a win."
      />
      <TaskSection title="Today" tasks={sections.today} />
      <TaskSection title="Anytime" tasks={sections.anytime} />
      <TaskSection title="Done today" tasks={done} />
    </TaskScreen>
  );
}
