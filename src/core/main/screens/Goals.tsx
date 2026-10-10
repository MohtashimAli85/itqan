import { useGoals } from "@/shared/hooks/useGoals";
import { GoalCard } from "../goals/GoalCard";
import { NewGoalForm } from "../goals/NewGoalForm";
import { SkillsPanel } from "../goals/SkillsPanel";

export function Goals() {
  const { data: goals = [] } = useGoals();

  return (
    <main className="grid min-w-0 flex-1 gap-10 overflow-y-auto p-8 md:grid-cols-[1fr_18rem]">
      <div className="flex flex-col gap-6">
        <header className="flex items-baseline justify-between">
          <h1 className="font-heading text-3xl font-bold">Goals</h1>
        </header>
        <NewGoalForm />
        {goals.length === 0 ? (
          <p className="text-sm text-caption">
            Add a goal and I'll break it into weekly milestones that end in
            something you ship.
          </p>
        ) : (
          goals.map((goal) => <GoalCard key={goal.id} goal={goal} />)
        )}
      </div>
      <SkillsPanel />
    </main>
  );
}
