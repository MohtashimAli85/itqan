import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DatePicker } from "./date-picker";

describe("DatePicker", () => {
  it("shows the formatted value", () => {
    render(<DatePicker value="2026-10-15" onChange={() => {}} />);

    expect(
      screen.getByRole("button", { name: "Oct 15, 2026" }),
    ).toBeInTheDocument();
  });

  it("shows a placeholder and clears when optional", () => {
    const onChange = vi.fn<(value: string) => void>();
    render(<DatePicker value="2026-10-15" onChange={onChange} optional />);

    fireEvent.click(screen.getByRole("button", { name: "Oct 15, 2026" }));
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(onChange).toHaveBeenCalledWith("");
  });
});
