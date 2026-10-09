import { Link } from "@tanstack/react-router";
import { useGoals, useGoalsSync } from "@/shared/hooks/useGoals";
import { GoalCard } from "../goals/GoalCard";
import { NewGoalForm } from "../goals/NewGoalForm";
import { SkillsPanel } from "../goals/SkillsPanel";

export function Goals() {
  useGoalsSync();
  const { data: goals = [] } = useGoals();

  return (
    <main className="mx-auto grid max-w-5xl gap-10 p-10 md:grid-cols-[1fr_18rem]">
      <div className="flex flex-col gap-6">
        <header className="flex items-baseline justify-between">
          <h1 className="font-heading text-3xl font-bold">Goals</h1>
          <Link to="/" className="text-text-secondary text-sm underline">
            Back
          </Link>
        </header>
        <NewGoalForm />
        {goals.length === 0 ? (
          <p className="text-caption text-sm">
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
