import { commands, type Rect } from "@/shared/bindings/bindings";

export function reportHitAreas(root: ParentNode | null) {
  const rects: Rect[] = root
    ? Array.from(root.querySelectorAll("[data-hit-area]"), (element) => {
        const { x, y, width, height } = element.getBoundingClientRect();
        return { x, y, width, height };
      })
    : [];
  void commands.setOverlayHitAreas(rects);
}
