export type Point = { x: number; y: number };
export type Rect = Point & { width: number; height: number };

export const ORB_SIZE = 56;
const CURSOR_OFFSET = 20;
const HOVER_MARGIN = 12;
const EASING = 0.18;
const SETTLE_DISTANCE = 0.5;
const PUPIL_TRAVEL = 4;

export function orbRect(position: Point): Rect {
  return { ...position, width: ORB_SIZE, height: ORB_SIZE };
}

export function isInside(point: Point, rect: Rect, margin = 0): boolean {
  return (
    point.x >= rect.x - margin &&
    point.x < rect.x + rect.width + margin &&
    point.y >= rect.y - margin &&
    point.y < rect.y + rect.height + margin
  );
}

export function followTarget(cursor: Point): Point {
  return { x: cursor.x + CURSOR_OFFSET, y: cursor.y + CURSOR_OFFSET };
}

export function pupilOffset(position: Point, cursor: Point): Point {
  const dx = cursor.x - (position.x + ORB_SIZE / 2);
  const dy = cursor.y - (position.y + ORB_SIZE / 2);
  const distance = Math.hypot(dx, dy);
  if (distance === 0) return { x: 0, y: 0 };
  const travel = Math.min(PUPIL_TRAVEL, distance / 20);
  return { x: (dx / distance) * travel, y: (dy / distance) * travel };
}

type FollowerOptions = {
  render: (position: Point, pupils: Point) => void;
  onMove: () => void;
  onSettle: (rect: Rect) => void;
  reducedMotion: () => boolean;
};

export function createFollower(options: FollowerOptions) {
  let position: Point = { x: -ORB_SIZE, y: -ORB_SIZE };
  let target = position;
  let cursor = position;
  let hovering = false;
  let frame: number | null = null;

  function step() {
    const dx = target.x - position.x;
    const dy = target.y - position.y;
    const settled =
      options.reducedMotion() || Math.hypot(dx, dy) < SETTLE_DISTANCE;
    position = settled
      ? target
      : { x: position.x + dx * EASING, y: position.y + dy * EASING };
    options.render(position, pupilOffset(position, cursor));
    if (settled) {
      frame = null;
      options.onSettle(orbRect(position));
    } else {
      frame = requestAnimationFrame(step);
    }
  }

  function setCursor(next: Point) {
    cursor = next;
    hovering = isInside(cursor, orbRect(position), hovering ? HOVER_MARGIN : 0);
    if (hovering) {
      options.render(position, pupilOffset(position, cursor));
      return;
    }
    target = followTarget(cursor);
    if (frame === null) {
      options.onMove();
      frame = requestAnimationFrame(step);
    }
  }

  function stop() {
    if (frame !== null) cancelAnimationFrame(frame);
    frame = null;
  }

  return { setCursor, stop };
}
