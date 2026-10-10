import { useMutation } from "@tanstack/react-query";
import {
  commands,
  type Goal,
  type PlanProposal,
} from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { dayjs } from "@/shared/lib/dayjs";
import { unwrap } from "@/shared/lib/result";

type PlanProposalViewProps = {
  goal: Goal;
  proposal: PlanProposal;
  onDone: () => void;
};

async function accept(goal: Goal, proposal: PlanProposal, addTask: boolean) {
  unwrap(await commands.addMilestones(goal.id, proposal.milestones));
  if (!addTask) return;
  unwrap(
    await commands.createTask({
      title: proposal.firstTask,
      notes: null,
      categoryId: null,
      kind: "deepWork",
      priority: 0,
      dueAt: dayjs().endOf("day").toISOString(),
      parentId: null,
      goalId: goal.id,
      skillId: null,
    }),
  );
}

export function PlanProposalView({
  goal,
  proposal,
  onDone,
}: PlanProposalViewProps) {
  const save = useMutation({
    mutationFn: (addTask: boolean) => accept(goal, proposal, addTask),
    onSuccess: onDone,
  });

  return (
    <div className="flex flex-col gap-3 rounded-xl bg-muted p-4">
      <p className="text-sm font-medium">
        {proposal.source === "ai" ? "Suggested plan (AI)" : "Suggested plan"}
      </p>
      {proposal.notice && (
        <p role="status" className="text-xs text-caption">
          {proposal.notice}
        </p>
      )}
      <ol className="flex flex-col gap-1.5">
        {proposal.milestones.map((milestone) => (
          <li
            key={`${milestone.weekStart}-${milestone.title}`}
            className="flex gap-3 text-sm"
          >
            <span className="w-20 shrink-0 font-mono text-xs text-caption">
              {dayjs(milestone.weekStart).format("D MMM")}
            </span>
            {milestone.title}
          </li>
        ))}
      </ol>
      <p className="text-xs text-caption">
        About {proposal.sessionsPerWeek} sessions of {proposal.sessionMinutes}{" "}
        minutes a week fit your free time.
      </p>
      {save.error && (
        <p role="alert" className="text-sm text-critical">
          {save.error.message}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          disabled={save.isPending}
          onClick={() => save.mutate(true)}
        >
          Accept and add today's task
        </Button>
        <Button
          size="sm"
          variant="secondary"
          disabled={save.isPending}
          onClick={() => save.mutate(false)}
        >
          Accept milestones only
        </Button>
        <Button size="sm" variant="ghost" onClick={onDone}>
          Not now
        </Button>
      </div>
    </div>
  );
}
