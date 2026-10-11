import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Calendar } from "./calendar";

describe("Calendar", () => {
  it("selects a day and emits its ISO date", () => {
    const onChange = vi.fn<(value: string) => void>();
    render(<Calendar value="2026-10-15" onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("20 October 2026"));

    expect(onChange).toHaveBeenCalledWith("2026-10-20");
  });

  it("moves between months", () => {
    render(<Calendar value="2026-10-15" onChange={() => {}} />);

    expect(screen.getByText("October 2026")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Next month" }));
    expect(screen.getByText("November 2026")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Previous month" }));
    fireEvent.click(screen.getByRole("button", { name: "Previous month" }));
    expect(screen.getByText("September 2026")).toBeInTheDocument();
  });

  it("disables days before min", () => {
    render(
      <Calendar value="2026-10-15" min="2026-10-10" onChange={() => {}} />,
    );

    expect(screen.getByLabelText("5 October 2026")).toBeDisabled();
    expect(screen.getByLabelText("12 October 2026")).toBeEnabled();
  });
});
