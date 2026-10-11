import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TimePicker } from "./time-picker";

describe("TimePicker", () => {
  it("emits an HH:mm value when the hour changes", () => {
    const onChange = vi.fn<(value: string) => void>();
    render(<TimePicker value="09:00" onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("Time hour"));
    fireEvent.click(screen.getByRole("option", { name: "10" }));

    expect(onChange).toHaveBeenCalledWith("10:00");
  });
});
