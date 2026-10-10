export type Point = { x: number; y: number };
export type Rect = Point & { width: number; height: number };
export type FollowMode = "follow" | "corner" | "hidden";

export const ORB_SIZE = 56;
const CURSOR_OFFSET = 20;
const CORNER_MARGIN = 28;
const HOVER_MARGIN = 12;
const EASING = 0.18;
const SETTLE_DISTANCE = 0.5;
const PUPIL_TRAVEL = 4;
const MAX_TILT = 12;

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

export function cornerTarget(viewport: { width: number; height: number }) {
  return {
    x: viewport.width - ORB_SIZE - CORNER_MARGIN,
    y: viewport.height - ORB_SIZE - CORNER_MARGIN,
  };
}

export function pupilOffset(position: Point, cursor: Point): Point {
  const dx = cursor.x - (position.x + ORB_SIZE / 2);
  const dy = cursor.y - (position.y + ORB_SIZE / 2);
  const distance = Math.hypot(dx, dy);
  if (distance === 0) return { x: 0, y: 0 };
  const travel = Math.min(PUPIL_TRAVEL, distance / 20);
  return { x: (dx / distance) * travel, y: (dy / distance) * travel };
}

export function tiltFor(velocityX: number): number {
  return Math.max(-MAX_TILT, Math.min(MAX_TILT, velocityX * 0.6));
}

export type Frame = { position: Point; pupils: Point; tilt: number };

type FollowerOptions = {
  render: (frame: Frame) => void;
  onMove: () => void;
  onSettle: () => void;
  reducedMotion: () => boolean;
  viewport: () => { width: number; height: number };
};

export function createFollower(options: FollowerOptions) {
  let position: Point = { x: -ORB_SIZE * 2, y: -ORB_SIZE * 2 };
  let target = position;
  let cursor = position;
  let mode: FollowMode = "follow";
  let pinned = false;
  let hovering = false;
  let frame: number | null = null;

  function draw(tilt: number) {
    options.render({
      position,
      pupils: pupilOffset(position, cursor),
      tilt,
    });
  }

  function step() {
    const dx = target.x - position.x;
    const dy = target.y - position.y;
    const settled =
      options.reducedMotion() || Math.hypot(dx, dy) < SETTLE_DISTANCE;
    if (settled) {
      position = target;
      frame = null;
      draw(0);
      options.onSettle();
      return;
    }
    position = { x: position.x + dx * EASING, y: position.y + dy * EASING };
    draw(tiltFor(dx * EASING));
    frame = requestAnimationFrame(step);
  }

  function moveTo(next: Point) {
    if (next.x === target.x && next.y === target.y) return;
    target = next;
    if (frame === null) {
      options.onMove();
      frame = requestAnimationFrame(step);
    }
  }

  function retarget() {
    if (mode === "corner") {
      moveTo(cornerTarget(options.viewport()));
    } else if (mode === "follow" && !pinned && !hovering) {
      moveTo(followTarget(cursor));
    }
  }

  function setCursor(next: Point) {
    cursor = next;
    hovering = isInside(cursor, orbRect(position), hovering ? HOVER_MARGIN : 0);
    if (frame === null) draw(0);
    retarget();
  }

  function setMode(next: FollowMode) {
    mode = next;
    if (frame === null) draw(0);
    retarget();
  }

  function setPinned(next: boolean) {
    pinned = next;
    retarget();
  }

  function isMoving() {
    return frame !== null;
  }

  function stop() {
    if (frame !== null) cancelAnimationFrame(frame);
    frame = null;
  }

  return { setCursor, setMode, setPinned, retarget, isMoving, stop };
}

export type Follower = ReturnType<typeof createFollower>;
