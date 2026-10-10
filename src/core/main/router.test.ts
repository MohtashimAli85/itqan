import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it } from "vitest";
import { createAppRouter } from "./router";

describe("app router", () => {
  it("registers module routes next to the core pages", () => {
    const router = createAppRouter(new QueryClient());
    expect(Object.keys(router.routesByPath).sort()).toEqual(
      [
        "/",
        "/upcoming",
        "/category/$categoryId",
        "/goals",
        "/progress",
        "/settings",
        "/onboarding",
      ].sort(),
    );
  });
});
