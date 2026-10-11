import { Award, Lock } from "lucide-react";
import type { Badge } from "@/shared/bindings/bindings";
import { Card, CardContent } from "@/shared/components/ui/card";

export function BadgeGrid({ badges }: { badges: Badge[] }) {
  return (
    <section aria-labelledby="badges" className="flex flex-col gap-3">
      <h2 id="badges" className="font-heading text-lg font-semibold">
        Badges
      </h2>
      <div className="grid grid-cols-2 gap-2 lg:grid-cols-4">
        {badges.map((badge) => (
          <Card
            key={badge.id}
            className={
              badge.earned
                ? "rounded-xl ring-border [--card-spacing:--spacing(3)]"
                : "rounded-xl bg-muted text-caption ring-0 [--card-spacing:--spacing(3)]"
            }
          >
            <CardContent className="flex items-start gap-2">
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
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  );
}
