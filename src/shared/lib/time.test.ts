import { describe, expect, it } from "vitest";
import { minutesToTime, timeToMinutes } from "./time";

describe("time helpers", () => {
  it("round trips clock times", () => {
    expect(minutesToTime(540)).toBe("09:00");
    expect(timeToMinutes("17:30")).toBe(1050);
    expect(timeToMinutes(minutesToTime(1439))).toBe(1439);
  });

  it("rejects invalid input", () => {
    expect(timeToMinutes("24:00")).toBeNull();
    expect(timeToMinutes("9am")).toBeNull();
  });
});
