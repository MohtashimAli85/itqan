import { useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { commands, type AiStatus } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import { aiKey } from "@/shared/hooks/useAi";
import { unwrap } from "@/shared/lib/result";

export function AiKeyField({ status }: { status: AiStatus }) {
  const [key, setKey] = useState("");
  const queryClient = useQueryClient();
  const onSuccess = (next: AiStatus) => {
    queryClient.setQueryData(aiKey, next);
    setKey("");
  };
  const save = useMutation({
    mutationFn: async () => unwrap(await commands.setAiKey(key)),
    onSuccess,
  });
  const remove = useMutation({
    mutationFn: async () => unwrap(await commands.deleteAiKey()),
    onSuccess,
  });

  function submit(event: FormEvent) {
    event.preventDefault();
    if (key.trim()) save.mutate();
  }

  if (!status.needsKey) {
    return (
      <p className="text-caption text-sm">
        This provider runs on your computer and needs no key.
      </p>
    );
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-2">
      <Label htmlFor="ai-key">API key</Label>
      <div className="flex gap-2">
        <Input
          id="ai-key"
          type="password"
          autoComplete="off"
          placeholder={
            status.hasKey ? "Saved in your keychain" : "Paste your key"
          }
          value={key}
          onChange={(event) => setKey(event.target.value)}
        />
        <Button type="submit" variant="secondary" disabled={save.isPending}>
          Save key
        </Button>
        {status.hasKey && (
          <Button
            type="button"
            variant="ghost"
            disabled={remove.isPending}
            onClick={() => remove.mutate()}
          >
            Remove
          </Button>
        )}
      </div>
      <p className="text-caption text-xs">
        Stored only in your system keychain. Itqan never shows it again.
      </p>
      {(save.error ?? remove.error) && (
        <p role="alert" className="text-critical text-sm">
          {(save.error ?? remove.error)?.message}
        </p>
      )}
    </form>
  );
}
