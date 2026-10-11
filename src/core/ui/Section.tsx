import type { ReactNode } from "react";
import { Card, CardContent, CardHeader } from "@/shared/components/ui/card";

type SectionProps = {
  id: string;
  title: string;
  description?: string;
  children: ReactNode;
};

export function Section({ id, title, description, children }: SectionProps) {
  return (
    <Card
      role="region"
      aria-labelledby={id}
      className="gap-4 rounded-2xl ring-border [--card-spacing:--spacing(6)]"
    >
      <CardHeader>
        <h2 id={id} className="font-heading text-lg font-semibold">
          {title}
        </h2>
        {description && <p className="text-sm text-caption">{description}</p>}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}
