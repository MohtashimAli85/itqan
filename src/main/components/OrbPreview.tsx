import {
  commands,
  type FollowMode,
  type OrbState,
} from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";

const states: OrbState[] = [
  "idle",
  "happy",
  "alert",
  "focus",
  "resting",
  "critical",
  "listening",
  "evening",
];
const modes: FollowMode[] = ["follow", "corner", "hidden"];
const progressSteps = [null, 0.25, 0.6, 1] as const;

function showSampleBubble() {
  void commands.showBubble(
    "You planned to ship the parser tonight. Start a 25 minute focus?",
    [
      { id: "start-focus", label: "Start focus" },
      { id: "snooze", label: "In 30 min" },
      { id: "skip", label: "Not today" },
    ],
  );
}

export function OrbPreview() {
  return (
    <section aria-labelledby="orb" className="flex flex-col gap-4">
      <h2 id="orb" className="font-heading text-xl font-semibold">
        Orb preview
      </h2>
      <div className="flex flex-wrap gap-2">
        {states.map((state) => (
          <Button
            key={state}
            size="sm"
            variant="outline"
            onClick={() => void commands.setOrbState(state)}
          >
            {state}
          </Button>
        ))}
      </div>
      <div className="flex flex-wrap gap-2">
        {modes.map((mode) => (
          <Button
            key={mode}
            size="sm"
            variant="secondary"
            onClick={() => void commands.setFollowMode(mode)}
          >
            {mode}
          </Button>
        ))}
        {progressSteps.map((step) => (
          <Button
            key={String(step)}
            size="sm"
            variant="secondary"
            onClick={() => void commands.setOrbProgress(step)}
          >
            {step === null ? "no ring" : `ring ${step * 100}%`}
          </Button>
        ))}
        <Button size="sm" onClick={showSampleBubble}>
          Show bubble
        </Button>
      </div>
    </section>
  );
}
