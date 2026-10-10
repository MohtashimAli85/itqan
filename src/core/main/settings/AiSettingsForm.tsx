import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Controller, useForm } from "react-hook-form";
import {
  commands,
  type AiSettings,
  type AiStatus,
  type Preset,
} from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Label } from "@/shared/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/components/ui/select";
import { Switch } from "@/shared/components/ui/switch";
import { aiKey, useAiStatus } from "@/shared/hooks/useAi";
import { unwrap } from "@/shared/lib/result";
import { AiKeyField } from "./AiKeyField";

const presets: { id: Preset; label: string }[] = [
  { id: "groq", label: "Groq (fast, free tier)" },
  { id: "ollama", label: "Ollama on this computer" },
  { id: "llamaCpp", label: "llama.cpp on this computer" },
  { id: "openAi", label: "OpenAI" },
  { id: "openRouter", label: "OpenRouter" },
  { id: "custom", label: "Other OpenAI-compatible" },
];

export function AiSettingsForm() {
  const { data: status } = useAiStatus();
  if (!status) return null;
  return <Form key={status.settings.preset} status={status} />;
}

function Form({ status }: { status: AiStatus }) {
  const queryClient = useQueryClient();
  const form = useForm<AiSettings>({ defaultValues: status.settings });
  const save = useMutation({
    mutationFn: async (settings: AiSettings) =>
      unwrap(await commands.saveAiSettings(settings)),
    onSuccess: (next) => queryClient.setQueryData(aiKey, next),
  });
  const test = useMutation({
    mutationFn: async () => unwrap(await commands.testAiConnection()),
  });

  async function choosePreset(preset: Preset) {
    const defaults = await commands.getAiPresetDefaults(preset);
    save.mutate({ ...defaults, enabled: form.getValues("enabled") });
  }

  return (
    <div className="flex flex-col gap-5">
      <form
        onSubmit={form.handleSubmit((values) => save.mutate(values))}
        className="flex flex-col gap-4"
      >
        <div className="flex items-center justify-between">
          <Label htmlFor="ai-enabled">Use AI for planning and questions</Label>
          <Controller
            control={form.control}
            name="enabled"
            render={({ field }) => (
              <Switch
                id="ai-enabled"
                checked={field.value}
                onCheckedChange={field.onChange}
              />
            )}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="ai-preset">Provider</Label>
          <Select
            value={status.settings.preset}
            onValueChange={(value) => void choosePreset(value as Preset)}
          >
            <SelectTrigger id="ai-preset" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {presets.map((preset) => (
                <SelectItem key={preset.id} value={preset.id}>
                  {preset.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="ai-url">Address</Label>
          <Input id="ai-url" {...form.register("baseUrl")} />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-2">
            <Label htmlFor="ai-fast">Fast model</Label>
            <Input id="ai-fast" {...form.register("fastModel")} />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="ai-smart">Smart model</Label>
            <Input id="ai-smart" {...form.register("smartModel")} />
          </div>
        </div>
        <div className="flex items-center justify-between">
          <Label htmlFor="ai-fallback">
            Fall back to Ollama on this computer if the provider fails
          </Label>
          <Controller
            control={form.control}
            name="localFallback"
            render={({ field }) => (
              <Switch
                id="ai-fallback"
                checked={field.value}
                onCheckedChange={field.onChange}
              />
            )}
          />
        </div>
        <div className="flex items-center gap-3">
          <Button type="submit" disabled={save.isPending}>
            Save
          </Button>
          {save.isSuccess && (
            <span role="status" className="text-sm text-caption">
              Saved
            </span>
          )}
        </div>
        {save.error && (
          <p role="alert" className="text-sm text-critical">
            {save.error.message}
          </p>
        )}
      </form>

      <AiKeyField status={status} />

      <div className="flex items-center gap-3">
        <Button
          variant="secondary"
          disabled={test.isPending || !status.settings.enabled}
          onClick={() => test.mutate()}
        >
          {test.isPending ? "Testing" : "Test connection"}
        </Button>
        {test.isSuccess && (
          <span role="status" className="text-sm text-personal">
            Connected
          </span>
        )}
        {test.error && (
          <span role="alert" className="text-sm text-critical">
            {test.error.message}
          </span>
        )}
      </div>
    </div>
  );
}
