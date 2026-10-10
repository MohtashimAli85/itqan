import { useMutation, useQueryClient } from "@tanstack/react-query";
import { commands } from "@/shared/bindings/bindings";
import { Label } from "@/shared/components/ui/label";
import { Switch } from "@/shared/components/ui/switch";
import { unwrap } from "@/shared/lib/result";
import { modules } from "@/modules";
import { modulesKey, useModuleStates } from "@/core/registry/useModules";
import { Section } from "@/core/ui/Section";

export function ModuleSettings() {
  const queryClient = useQueryClient();
  const { data: states = [] } = useModuleStates();
  const toggle = useMutation({
    mutationFn: async ({ id, enabled }: { id: string; enabled: boolean }) =>
      unwrap(await commands.setModuleEnabled(id, enabled)),
    onSuccess: (next) => queryClient.setQueryData(modulesKey, next),
  });

  return (
    <Section
      id="modules"
      title="Modules"
      description="Turn off what you don't use. Your data stays and comes back when you turn a module on again."
    >
      {modules.map((module) => {
        const enabled =
          states.find((state) => state.id === module.id)?.enabled ?? true;
        return (
          <div
            key={module.id}
            className="flex items-center justify-between gap-4"
          >
            <div className="flex flex-col gap-0.5">
              <Label htmlFor={`module-${module.id}`}>{module.name}</Label>
              <p className="text-xs text-caption">{module.description}</p>
            </div>
            <Switch
              id={`module-${module.id}`}
              checked={enabled}
              disabled={toggle.isPending}
              onCheckedChange={(value) =>
                toggle.mutate({ id: module.id, enabled: value })
              }
            />
          </div>
        );
      })}
    </Section>
  );
}
