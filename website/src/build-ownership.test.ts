import { existsSync } from "node:fs";
import { describe, expect, it } from "vitest";
import viteConfig from "../vite.config";

describe("website build ownership", () => {
  /** [playground.hosting]: Eleventy owns the site while Vite owns only the interactive playground. */
  it("restricts Vite to the playground entry point", () => {
    expect(viteConfig.build?.rollupOptions?.input).toEqual({
      playground: "playground.html",
    });
    expect(existsSync(new URL("./index.njk", import.meta.url))).toBe(true);
  });
});
