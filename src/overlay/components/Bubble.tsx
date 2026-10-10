import { X } from "lucide-react";
import type { Bubble as BubbleData } from "@/shared/bindings/bindings";
import { Button } from "@/shared/components/ui/button";

type BubbleProps = {
  bubble: BubbleData;
  onResolve: (actionId: string | null) => void;
};

export function Bubble({ bubble, onResolve }: BubbleProps) {
  return (
    <div
      data-hit-area
      role="status"
      className="bubble absolute w-64 rounded-2xl bg-card p-3 pr-8 text-sm text-card-foreground shadow-xl ring-1 ring-border"
    >
      <p className="leading-snug">{bubble.text}</p>
      {bubble.actions.length > 0 && (
        <div className="mt-2.5 flex flex-wrap gap-1.5">
          {bubble.actions.map((action, index) => (
            <Button
              key={action.id}
              size="sm"
              variant={index === 0 ? "default" : "secondary"}
              onClick={() => onResolve(action.id)}
            >
              {action.label}
            </Button>
          ))}
        </div>
      )}
      <button
        type="button"
        aria-label="Dismiss"
        onClick={() => onResolve(null)}
        className="absolute top-2 right-2 rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      >
        <X className="size-3.5" />
      </button>
    </div>
  );
}
