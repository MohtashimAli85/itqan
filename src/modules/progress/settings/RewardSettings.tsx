import { commands } from "@/shared/bindings/bindings";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { useToggle } from "@/shared/hooks/useToggle";
import { unwrap } from "@/shared/lib/result";
import { Section } from "@/core/ui/Section";

export function RewardSettings() {
  const sound = useToggle(
    "reward-sound",
    async () => unwrap(await commands.getRewardSound()),
    async (value) => unwrap(await commands.setRewardSound(value)),
  );

  return (
    <Section id="rewards" title="Rewards">
      <div className="flex items-center justify-between">
        <Label htmlFor="sound">Reward sounds</Label>
        <Switch
          id="sound"
          checked={sound.value ?? true}
          onCheckedChange={sound.set}
        />
      </div>
    </Section>
  );
}
