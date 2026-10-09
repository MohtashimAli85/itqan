import { describe, expect, it } from "vitest";
import { followTarget, isInside, orbRect, pupilOffset } from "./follower";

describe("follower", () => {
  it("keeps the orb offset from the cursor", () => {
    expect(followTarget({ x: 100, y: 200 })).toEqual({ x: 120, y: 220 });
  });

  it("treats the hover margin as part of the orb", () => {
    const rect = orbRect({ x: 100, y: 100 });
    expect(isInside({ x: 95, y: 120 }, rect)).toBe(false);
    expect(isInside({ x: 95, y: 120 }, rect, 12)).toBe(true);
  });

  it("points the pupils toward the cursor within their travel", () => {
    const offset = pupilOffset({ x: 0, y: 0 }, { x: 1028, y: 28 });
    expect(offset.x).toBeCloseTo(4);
    expect(offset.y).toBeCloseTo(0);
  });
});
