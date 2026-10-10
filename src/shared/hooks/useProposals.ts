import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands, events, type Proposal } from "@/shared/bindings/bindings";
import { unwrap } from "@/shared/lib/result";
import { useTauriEvent } from "./useTauriEvent";

const proposalsKey = ["proposals"] as const;

export function useProposalsSync() {
  const queryClient = useQueryClient();
  useTauriEvent(events.proposalsChanged, () => {
    void queryClient.invalidateQueries({ queryKey: proposalsKey });
  });
}

export function useProposals() {
  return useQuery({
    queryKey: proposalsKey,
    queryFn: async () => unwrap(await commands.listProposals()),
  });
}

export function useDecideProposal() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ id, accept }: { id: number; accept: boolean }) =>
      unwrap(
        await (accept
          ? commands.acceptProposal(id)
          : commands.rejectProposal(id)),
      ),
    onSuccess: (next: Proposal[]) =>
      queryClient.setQueryData(proposalsKey, next),
  });
}
