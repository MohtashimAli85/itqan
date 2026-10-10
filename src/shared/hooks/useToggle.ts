import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

export function useToggle(
  key: string,
  read: () => Promise<boolean>,
  write: (value: boolean) => Promise<unknown>,
) {
  const queryClient = useQueryClient();
  const query = useQuery({ queryKey: [key], queryFn: read });
  const mutation = useMutation({
    mutationFn: write,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [key] }),
  });
  return { value: query.data, set: (value: boolean) => mutation.mutate(value) };
}
