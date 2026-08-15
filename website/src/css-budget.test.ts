import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("authored CSS budget", () => {
  /** [playground.interface]: all hand-authored website CSS stays within 2,000 physical lines. */
  it("uses no more than 2,000 lines", () => {
    const directories = [new URL("./styles/", import.meta.url), new URL("./assets/css/", import.meta.url)];
    const files = directories.flatMap((directory) =>
      readdirSync(directory, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith(".css"))
        .map((entry) => new URL(entry.name, directory)),
    );
    const lineCount = files.reduce(
      (total, file) => total + readFileSync(file, "utf8").split("\n").length,
      0,
    );

    expect(lineCount).toBeLessThanOrEqual(2_000);
  });
});
