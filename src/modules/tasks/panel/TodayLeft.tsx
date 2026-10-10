import { useTodayTasks } from "@/modules/tasks/hooks/useTasks";

export function TodayLeft() {
  const { topThree, due } = useTodayTasks();
  const left = topThree.length + due.length;

  return (
    <p className="text-xs text-caption">
      {left === 0
        ? "Nothing due today. Add your top 3."
        : `${left} left for today`}
    </p>
  );
}
