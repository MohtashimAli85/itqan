import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDownIcon } from "lucide-react";
import { commands, type FollowMode } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/shared/components/ui/collapsible";
import { Label } from "@/shared/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/shared/components/ui/radio-group";
import { unwrap } from "@/shared/lib/result";
import { OrbPreview } from "../components/OrbPreview";
import { Section } from "@/core/ui/Section";

const modes: { id: FollowMode; label: string }[] = [
  { id: "follow", label: "Follow my cursor" },
  { id: "corner", label: "Sit in a corner" },
  { id: "hidden", label: "Hidden (notifications only)" },
];

export function OrbSettings() {
  const queryClient = useQueryClient();
  const { data: overlay } = useQuery({
    queryKey: ["overlay-settings"],
    queryFn: async () => unwrap(await commands.getOverlayState()),
  });
  const save = useMutation({
    mutationFn: async (mode: FollowMode) =>
      unwrap(await commands.setFollowMode(mode)),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["overlay-settings"] }),
  });

  return (
    <Section
      id="orb"
      title="Orb"
      description="Where the orb lives. Press ⌥ Space to open the panel any time."
    >
      {overlay && (
        <RadioGroup
          value={overlay.followMode}
          onValueChange={(value) => save.mutate(value as FollowMode)}
          className="flex flex-wrap gap-4"
        >
          {modes.map((mode) => (
            <Label
              key={mode.id}
              htmlFor={`follow-${mode.id}`}
              className="gap-2"
            >
              <RadioGroupItem id={`follow-${mode.id}`} value={mode.id} />
              {mode.label}
            </Label>
          ))}
        </RadioGroup>
      )}
      <Collapsible>
        <CollapsibleTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            className="group w-full justify-between"
          >
            Try orb states
            <ChevronDownIcon
              data-icon="inline-end"
              className="transition-transform group-data-[state=open]:rotate-180"
            />
          </Button>
        </CollapsibleTrigger>
        <CollapsibleContent className="pt-4">
          <OrbPreview />
        </CollapsibleContent>
      </Collapsible>
    </Section>
  );
}
