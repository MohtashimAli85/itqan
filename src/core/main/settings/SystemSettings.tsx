import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { commands } from "@/shared/bindings/bindings";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { unwrap } from "@/shared/lib/result";
import { useAppInfo } from "../hooks/useAppInfo";
import { Section } from "@/core/ui/Section";

function useToggle(
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

export function SystemSettings() {
  const { data: info } = useAppInfo();
  const autostart = useToggle(
    "autostart",
    async () => unwrap(await commands.getAutostart()),
    async (value) => unwrap(await commands.setAutostart(value)),
  );

  return (
    <>
      <Section
        id="profile"
        title="You"
        description="Motivation, coach style, work hours and prayer times."
      >
        <Link
          to="/onboarding"
          className="text-sm text-text-secondary underline"
        >
          Edit profile and schedule
        </Link>
      </Section>
      <Section id="system" title="System">
        <div className="flex items-center justify-between">
          <Label htmlFor="autostart">Launch Itqan when I log in</Label>
          <Switch
            id="autostart"
            checked={autostart.value ?? false}
            onCheckedChange={autostart.set}
          />
        </div>
      </Section>
      <Section id="privacy" title="Privacy">
        <p className="text-sm text-text-secondary">
          Everything stays on this computer in a local database. There are no
          accounts and no telemetry. Only the minimum text, with phone numbers,
          emails, card numbers, codes and keys removed, goes to the AI provider
          you choose. Keys live in your system keychain.
        </p>
        {info && (
          <p className="font-mono text-xs text-caption">
            {info.name} {info.version} · data schema {info.schemaVersion}
          </p>
        )}
      </Section>
    </>
  );
}
