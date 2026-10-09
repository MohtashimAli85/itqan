import type { ReactNode } from "react";

type SectionProps = {
  id: string;
  title: string;
  description?: string;
  children: ReactNode;
};

export function Section({ id, title, description, children }: SectionProps) {
  return (
    <section
      aria-labelledby={id}
      className="bg-card ring-border flex flex-col gap-4 rounded-2xl p-6 ring-1"
    >
      <header className="flex flex-col gap-1">
        <h2 id={id} className="font-heading text-lg font-semibold">
          {title}
        </h2>
        {description && <p className="text-caption text-sm">{description}</p>}
      </header>
      {children}
    </section>
  );
}
