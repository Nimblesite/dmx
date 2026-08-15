import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";
import MarkdownIt from "markdown-it";

/** [semantic-expansion.architecture]: the intended architecture is valid Mermaid. */
test("parses the intended architecture diagram", async ({ page }) => {
  const markdown = readFileSync(
    new URL("../../docs/plans/semantic-metaprogramming.md", import.meta.url),
    "utf8",
  );
  const diagrams = new MarkdownIt()
    .parse(markdown, {})
    .filter((token) => token.type === "fence" && token.info.trim() === "mermaid")
    .map((token) => token.content);

  expect(diagrams).toHaveLength(1);
  await page.goto("/docs/pipeline/");
  const result = await page.evaluate(async (diagram) => {
    const mermaidWindow = window as typeof window & {
      mermaid: { parse(source: string): Promise<unknown> };
    };
    return mermaidWindow.mermaid.parse(diagram);
  }, diagrams[0] ?? "");

  expect(result).toBeTruthy();
});
