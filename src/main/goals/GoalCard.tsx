import { useMutation } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import { commands, type Goal } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Checkbox } from "@/shared/components/ui/checkbox";
import { useMilestones } from "@/shared/hooks/useGoals";
import { dayjs } from "@/shared/lib/dayjs";
import { unwrap } from "@/shared/lib/result";
import { PlanProposalView } from "./PlanProposalView";

export function GoalCard({ goal }: { goal: Goal }) {
  const { data: milestones = [] } = useMilestones(goal.id);
  const propose = useMutation({
    mutationFn: async () => unwrap(await commands.proposePlan(goal.id)),
  });
  const toggle = useMutation({
    mutationFn: async ({ id, done }: { id: number; done: boolean }) =>
      unwrap(await commands.setMilestoneStatus(id, done ? "done" : "open")),
  });
  const remove = useMutation({
    mutationFn: async () => unwrap(await commands.deleteGoal(goal.id)),
  });
  const finished = milestones.filter((m) => m.status === "done").length;

  return (
    <article className="flex flex-col gap-4 rounded-2xl bg-card p-5 ring-1 ring-border">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h3 className="font-heading text-lg font-semibold">{goal.title}</h3>
          <p className="text-xs text-caption">
            {goal.targetDate
              ? `By ${dayjs(goal.targetDate).format("D MMMM YYYY")}`
              : "No target date"}
            {milestones.length > 0 &&
              ` · ${finished} of ${milestones.length} milestones`}
          </p>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={`Delete ${goal.title}`}
          onClick={() => remove.mutate()}
        >
          <Trash2 />
        </Button>
      </header>

      {milestones.length > 0 ? (
        <ul className="flex flex-col gap-1">
          {milestones.map((milestone) => (
            <li key={milestone.id} className="flex items-center gap-3">
              <Checkbox
                id={`milestone-${milestone.id}`}
                checked={milestone.status === "done"}
                onCheckedChange={(checked) =>
                  toggle.mutate({ id: milestone.id, done: checked === true })
                }
              />
              <label
                htmlFor={`milestone-${milestone.id}`}
                className="flex-1 text-sm"
              >
                {milestone.title}
              </label>
              <span className="font-mono text-xs text-caption">
                {dayjs(milestone.weekStart).format("D MMM")}
              </span>
            </li>
          ))}
        </ul>
      ) : propose.data ? (
        <PlanProposalView
          goal={goal}
          proposal={propose.data}
          onDone={() => propose.reset()}
        />
      ) : (
        <Button
          variant="secondary"
          size="sm"
          className="self-start"
          disabled={propose.isPending}
          onClick={() => propose.mutate()}
        >
          Plan it
        </Button>
      )}
    </article>
  );
}
