import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { unwrap } from "@/shared/lib/result";
import { Section } from "@/core/ui/Section";

const soundKey = ["reward-sound"] as const;

export function RewardSettings() {
  const queryClient = useQueryClient();
  const { data: sound } = useQuery({
    queryKey: soundKey,
    queryFn: async () => unwrap(await commands.getRewardSound()),
  });
  const setSound = useMutation({
    mutationFn: async (value: boolean) =>
      unwrap(await commands.setRewardSound(value)),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: soundKey }),
  });

  return (
    <Section id="rewards" title="Rewards">
      <div className="flex items-center justify-between">
        <Label htmlFor="sound">Reward sounds</Label>
        <Switch
          id="sound"
          checked={sound ?? true}
          onCheckedChange={(value) => setSound.mutate(value)}
        />
      </div>
    </Section>
  );
}
