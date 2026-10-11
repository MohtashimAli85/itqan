import type { ReactNode } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/shared/components/ui/card";

type SectionProps = {
  id: string;
  title: string;
  description?: string;
  children: ReactNode;
};

export function Section({ id, title, description, children }: SectionProps) {
  return (
    <Card
      aria-labelledby={id}
      className="gap-4 rounded-2xl ring-border [--card-spacing:--spacing(6)]"
    >
      <CardHeader>
        <CardTitle id={id} className="font-heading text-lg font-semibold">
          {title}
        </CardTitle>
        {description && (
          <CardDescription className="text-caption">
            {description}
          </CardDescription>
        )}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}
