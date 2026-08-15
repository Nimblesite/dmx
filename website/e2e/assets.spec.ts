import { readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";

const distributionRoot = fileURLToPath(new URL("../dist", import.meta.url));
const indexFile = "index.html";

/**
 * Every page the build publishes, as the path a visitor types. Read from the
 * build rather than listed here: a hand-written list covers the pages somebody
 * remembered, and the page that breaks is the one nobody remembered.
 */
function publishedRoutes(): string[] {
  const walk = (directory: string): string[] =>
    readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      const child = join(directory, entry.name);
      if (entry.isDirectory()) return walk(child);
      return entry.name.endsWith(".html") ? [relative(distributionRoot, child)] : [];
    });

  return walk(distributionRoot)
    .map((file) => `/${file.endsWith(indexFile) ? file.slice(0, -indexFile.length) : file}`)
    .sort();
}

/**
 * [playground.hosting]: the site is served from the root of its own domain, so
 * every asset reference has to resolve there. Nothing else in this suite
 * notices when they do not — the markup is identical either way, and a page
 * whose every stylesheet 404s is uniformly unstyled, so the comparisons that
 * check one page against another still agree.
 */
test("resolves every asset the published pages reference", async ({ page }) => {
  const broken: string[] = [];
  page.on("response", (response) => {
    if (response.status() >= 400) broken.push(`${response.status()} ${response.url()}`);
  });

  const routes = publishedRoutes();
  expect(routes).toContain("/");
  expect(routes).toContain("/playground.html");

  for (const route of routes) {
    await page.goto(route, { waitUntil: "load" });
  }

  expect(broken).toEqual([]);
});

/**
 * [playground.hosting]: proof the stylesheets ARRIVED, which a link tag naming
 * them is not. Counting RULES rather than checking `link.sheet`, because the
 * browser hands a link whose href 404s a stylesheet object all the same — an
 * empty one. `sheet !== null` is true on a page with no styling whatsoever.
 */
test("applies every stylesheet the published pages link", async ({ page }) => {
  for (const route of publishedRoutes()) {
    await page.goto(route, { waitUntil: "load" });

    const sheets = await page.evaluate(() =>
      Array.from(document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')).map(
        (link) => ({ href: link.getAttribute("href") ?? "", rules: link.sheet?.cssRules.length ?? 0 }),
      ),
    );

    expect(sheets.length, `${route} links no stylesheet at all`).toBeGreaterThan(0);
    expect(
      sheets.filter((sheet) => sheet.rules === 0),
      `${route} links a stylesheet that arrived with no rules in it`,
    ).toEqual([]);
  }
});
