import { Award, Lock } from "lucide-react";
import type { Badge } from "@/shared/bindings/bindings";

export function BadgeGrid({ badges }: { badges: Badge[] }) {
  return (
    <section aria-labelledby="badges" className="flex flex-col gap-3">
      <h2 id="badges" className="font-heading text-lg font-semibold">
        Badges
      </h2>
      <ul className="grid grid-cols-2 gap-2 lg:grid-cols-4">
        {badges.map((badge) => (
          <li
            key={badge.id}
            className={
              badge.earned
                ? "flex items-start gap-2 rounded-xl bg-card p-3 ring-1 ring-border"
                : "flex items-start gap-2 rounded-xl bg-muted p-3 text-caption"
            }
          >
            {badge.earned ? (
              <Award
                aria-hidden
                className="mt-0.5 size-4 shrink-0 text-amber"
              />
            ) : (
              <Lock aria-hidden className="mt-0.5 size-4 shrink-0" />
            )}
            <span className="flex flex-col">
              <span className="text-sm font-medium">{badge.title}</span>
              <span className="text-xs">{badge.description}</span>
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
