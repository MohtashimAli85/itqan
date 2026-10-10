import { Check, X } from "lucide-react";
import { Button } from "@/shared/components/ui/button";
import { useDecideProposal, useProposals } from "@/shared/hooks/useProposals";

export function Proposals() {
  const { data: proposals = [] } = useProposals();
  const decide = useDecideProposal();

  return (
    <main className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto p-8">
      <header className="flex flex-col gap-1">
        <h1 className="font-heading text-3xl font-bold">Suggestions</h1>
        <p className="text-sm text-caption">
          Changes Itqan wants to make from what it has learned. Nothing changes
          until you say yes.
        </p>
      </header>
      {proposals.length === 0 ? (
        <p className="text-sm text-caption">
          Nothing to review. When Itqan notices a pattern, it will ask here
          first.
        </p>
      ) : (
        <ul className="flex max-w-2xl flex-col gap-4">
          {proposals.map((proposal) => (
            <li
              key={proposal.id}
              className="flex flex-col gap-3 rounded-2xl bg-card p-5 ring-1 ring-border"
            >
              <h2 className="font-heading text-lg font-semibold">
                {proposal.title}
              </h2>
              <p className="text-sm text-text-secondary">{proposal.reason}</p>
              <p className="text-xs text-caption">{proposal.effect}</p>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  disabled={decide.isPending}
                  onClick={() =>
                    decide.mutate({ id: proposal.id, accept: true })
                  }
                >
                  <Check data-icon="inline-start" />
                  Yes, change it
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={decide.isPending}
                  onClick={() =>
                    decide.mutate({ id: proposal.id, accept: false })
                  }
                >
                  <X data-icon="inline-start" />
                  No
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </main>
  );
}
