import { Lightbulb } from "lucide-react";
import { commands } from "@/shared/bindings/bindings";
import { useProposals } from "@/shared/hooks/useProposals";

export function ProposalsRow({ onOpen }: { onOpen: () => void }) {
  const { data: proposals = [] } = useProposals();
  if (proposals.length === 0) return null;

  return (
    <button
      type="button"
      onClick={() => {
        void commands.openMainWindow("/proposals");
        onOpen();
      }}
      className="flex items-center gap-2 rounded-lg bg-muted px-3 py-2 text-left text-sm hover:bg-muted/80"
    >
      <Lightbulb aria-hidden className="size-4 text-amber" />
      {proposals.length === 1
        ? "1 suggestion to review"
        : `${proposals.length} suggestions to review`}
    </button>
  );
}
