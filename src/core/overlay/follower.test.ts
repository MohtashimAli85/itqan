import { describe, expect, it } from "vitest";
import {
  cornerTarget,
  followTarget,
  isInside,
  orbRect,
  pupilOffset,
  tiltFor,
} from "./follower";

describe("follower", () => {
  it("keeps the orb offset from the cursor", () => {
    expect(followTarget({ x: 100, y: 200 })).toEqual({ x: 120, y: 220 });
  });

  it("parks the orb in the bottom right corner", () => {
    expect(cornerTarget({ width: 1440, height: 900 })).toEqual({
      x: 1356,
      y: 816,
    });
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

  it("limits the tilt while moving", () => {
    expect(tiltFor(100)).toBe(12);
    expect(tiltFor(-100)).toBe(-12);
    expect(tiltFor(5)).toBeCloseTo(3);
  });
});
