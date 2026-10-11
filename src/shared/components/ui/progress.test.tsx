import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Progress } from "./progress";

describe("Progress", () => {
  it("fills proportionally to value over max", () => {
    render(<Progress value={50} max={200} aria-label="Experience" />);

    const bar = screen.getByRole("progressbar", { name: "Experience" });
    expect(bar).toHaveAttribute("aria-valuenow", "50");
    expect(bar).toHaveAttribute("aria-valuemax", "200");
    expect(bar.querySelector('[data-slot="progress-indicator"]')).toHaveStyle({
      transform: "translateX(-75%)",
    });
  });

  it("treats value as a percentage when max is omitted", () => {
    render(<Progress value={30} aria-label="Percent" />);

    const bar = screen.getByRole("progressbar", { name: "Percent" });
    expect(bar).toHaveAttribute("aria-valuemax", "100");
    expect(bar.querySelector('[data-slot="progress-indicator"]')).toHaveStyle({
      transform: "translateX(-70%)",
    });
  });
});
