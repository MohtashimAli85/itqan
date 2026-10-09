import { useState } from "react";
import { Moon, Sun } from "lucide-react";
import { Button } from "@/shared/components/ui/button";

const swatches = [
  { name: "ink", className: "bg-ink" },
  { name: "text-secondary", className: "bg-text-secondary" },
  { name: "caption", className: "bg-caption" },
  { name: "brand", className: "bg-brand" },
  { name: "amber", className: "bg-amber" },
  { name: "work", className: "bg-work" },
  { name: "personal", className: "bg-personal" },
  { name: "health", className: "bg-health" },
  { name: "critical", className: "bg-critical" },
];

export function App() {
  const [dark, setDark] = useState(false);

  function toggleTheme() {
    document.documentElement.classList.toggle("dark", !dark);
    setDark(!dark);
  }

  return (
    <main className="mx-auto flex max-w-3xl flex-col gap-10 p-10">
      <header className="flex items-center justify-between">
        <h1 className="font-heading text-4xl font-bold">Itqan tokens</h1>
        <Button
          variant="outline"
          size="icon"
          aria-label="Toggle theme"
          onClick={toggleTheme}
        >
          {dark ? <Sun /> : <Moon />}
        </Button>
      </header>

      <section aria-labelledby="colours" className="flex flex-col gap-4">
        <h2 id="colours" className="font-heading text-xl font-semibold">
          Colours
        </h2>
        <ul className="grid grid-cols-3 gap-4">
          {swatches.map(({ name, className }) => (
            <li key={name} className="flex flex-col gap-2">
              <div className={`h-14 rounded-lg ${className}`} />
              <span className="text-text-secondary font-mono text-xs">
                {name}
              </span>
            </li>
          ))}
        </ul>
      </section>

      <section aria-labelledby="type" className="flex flex-col gap-3">
        <h2 id="type" className="font-heading text-xl font-semibold">
          Type
        </h2>
        <p className="font-heading text-3xl font-bold">
          Bricolage Grotesque for headings
        </p>
        <p className="font-sans text-base">Geist for interface text</p>
        <p className="font-mono text-2xl">25:00 Geist Mono for timers</p>
        <p className="text-caption text-sm">Caption text</p>
      </section>

      <section aria-labelledby="buttons" className="flex flex-col gap-4">
        <h2 id="buttons" className="font-heading text-xl font-semibold">
          Buttons
        </h2>
        <div className="flex gap-3">
          <Button>Primary</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="outline">Outline</Button>
          <Button variant="ghost">Ghost</Button>
          <Button variant="destructive">Destructive</Button>
        </div>
      </section>
    </main>
  );
}
