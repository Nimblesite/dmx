import { expect, test } from "@playwright/test";

/** [playground.hosting]: every generated surface reuses byte-identical navigation markup. */
test("reuses one navigation across every product page", async ({ page }) => {
  const routes = [
    "/",
    "/playground.html",
    "/docs/",
    "/docs/dart-custom-macros/",
    "/docs/macros/",
    "/blog/",
    "/blog/introducing-dmx/",
  ];
  const navigationMarkup: string[] = [];
  const navigationPresentation: string[] = [];

  for (const route of routes) {
    await page.goto(route);
    const navigation = page.locator("[data-shared-navigation]");
    await expect(navigation).toHaveCount(1);
    navigationMarkup.push(await navigation.evaluate((element) => element.outerHTML));
    navigationPresentation.push(await navigation.evaluate((element) => {
      const styles = [
        element,
        element.querySelector(".nav-shell"),
        element.querySelector(".brand"),
        element.querySelector(".nav-menu"),
        element.querySelector(".nav-menu > a"),
        element.querySelector(".nav-github"),
      ].map((target) => {
        const style = getComputedStyle(target ?? element);
        return [
          style.backgroundColor,
          style.borderColor,
          style.color,
          style.display,
          style.fontFamily,
          style.fontSize,
          style.fontWeight,
          style.gap,
          style.height,
          style.padding,
        ];
      });
      return JSON.stringify(styles);
    }));
  }

  expect(new Set(navigationMarkup).size).toBe(1);
  expect(new Set(navigationPresentation).size).toBe(1);
});

/** [playground.interface]: the homepage shows current generator output, not placeholder Dart. */
test("shows an actual generated Dart region in the homepage hero", async ({ page }) => {
  await page.goto("/");

  const code = page.locator(".hero-code");
  await expect(code).toContainText("//#region");
  await expect(code).toContainText(
    "static Result<Profile, DecodeError> fromJson(Object? json, [String path = 'Profile']) =>",
  );
  await expect(code).toContainText("Profile copyWith({");
  await expect(code).toContainText("//#endregion");
  await expect(code).not.toContainText("dmx:generated");
  await expect(code).not.toContainText("…");
});

/** [playground.hosting]: the product header exposes the two primary documentation routes. */
test("consolidates documentation under one desktop menu", async ({ page }) => {
  await page.goto("/");

  const navigation = page.locator("#site-navigation");
  await expect(navigation.locator(":scope > a")).toHaveText(["Playground", "Blog", "GitHub"]);
  await expect(navigation.getByRole("link", { name: "GitHub" })).toHaveAttribute(
    "href",
    "https://github.com/Nimblesite/dmx",
  );

  const docs = navigation.locator(".nav-docs");
  await docs.locator("summary").click();
  await expect(docs.locator(".nav-submenu").getByRole("link")).toHaveText([
    "Getting started",
    "Dart (Custom) Macros",
    "Macro catalogue",
  ]);
});

/** [playground.hosting]: Docs is a real route, not only a dropdown toggle. */
test("links Docs directly to the getting started guide", async ({ page }) => {
  await page.goto("/");

  await page.locator("#site-navigation").getByRole("link", { name: "Docs", exact: true }).click();

  await expect(page).toHaveURL(/\/docs\/$/);
  await expect(
    page.getByRole("heading", { level: 1, name: "Getting started with dmx" }),
  ).toBeVisible();
});

/** [playground.hosting]: documentation and articles share the same prose surface. */
test("uses the shared prose classes for docs and blog articles", async ({ page }) => {
  await page.goto("/docs/");
  await expect(page.locator("main article.prose.prose-content")).toBeVisible();

  await page.goto("/blog/introducing-dmx/");
  await expect(page.locator("main article.prose.prose-content")).toBeVisible();
});

/** [playground.hosting]: the introductory article states what dmx does under one title. */
test("introduces dmx as a code generation tool under one plain heading", async ({ page }) => {
  await page.goto("/blog/introducing-dmx/");

  const heading = page.getByRole("heading", { level: 1 });
  await expect(heading).toHaveCount(1);
  await expect(heading).toHaveText(
    "Dart model code generation without part files",
  );
  await expect(page.locator(".blog-post-content > p").first()).toContainText(
    "dmx is a Dart code generation tool.",
  );
  await expect(page.getByRole("link", { name: "JSON guide", exact: true })).toHaveAttribute(
    "href",
    "https://docs.flutter.dev/data-and-backend/serialization/json",
  );
  await expect(page.getByRole("link", { name: "stopped work on macros" })).toHaveAttribute(
    "href",
    "https://dart.dev/blog/an-update-on-dart-macros-data-serialization",
  );
  await expect(
    page.getByRole("link", { name: "Code generation experience needs improvements" }),
  ).toHaveAttribute("href", "https://github.com/flutter/flutter/issues/63323");
});

/** [playground.hosting]: the article shows generated Dart and carries no research notes. */
test("shows real generated Dart and carries no research notes", async ({ page }) => {
  await page.goto("/blog/introducing-dmx/");

  const article = page.locator("main article.prose.prose-content");
  await expect(article.locator("pre code").first()).toContainText("@dmx('model')");
  await expect(article.locator("pre code")).toContainText([
    /@dmx\('model'\)/u,
    /\/\/#region[\s\S]*copyWith/u,
    /copyParam/u,
  ]);
  await expect(article).toContainText("No part directive");

  const body = (await article.textContent()) ?? "";
  for (const researchPhrase of [
    "Google Trends",
    "Playwright",
    "marketing comparison",
    "editorial conclusion",
    "Relative average",
  ]) {
    expect(body).not.toContain(researchPhrase);
  }
});

/** [playground.hosting]: each post image appears on both the article and its listing card. */
test("shows the blog post image on the article and blog listing", async ({ page }) => {
  const alt = "Dart code generation adding members inside one source file";

  await page.goto("/blog/introducing-dmx/");
  const articleImage = page.locator("main article").getByRole("img", { name: alt, exact: true });
  await expect(articleImage).toBeVisible();
  await expect(articleImage).toHaveAttribute("src", /dmx-code-generation\.webp$/);
  expect(await articleImage.evaluate((image) => image instanceof HTMLImageElement
    ? image.naturalWidth
    : 0)).toBeGreaterThan(0);

  await page.goto("/docs/models-in-markdown/");
  await expect(page.getByRole("heading", { level: 1, name: "Models in Markdown" })).toBeVisible();
  await expect(
    page.getByText("A template belongs to the typeDiagram fence", { exact: false }),
  ).toBeVisible();

  await page.goto("/blog/");
  const cardImage = page.locator("main .post-list article").getByRole("img", {
    name: alt,
    exact: true,
  });
  await expect(cardImage).toBeVisible();
  await expect(cardImage).toHaveAttribute("src", /dmx-code-generation\.webp$/);
  expect(await cardImage.evaluate((image) => image instanceof HTMLImageElement
    ? image.naturalWidth
    : 0)).toBeGreaterThan(0);
});

/** [playground.hosting]: documentation diagrams render as Mermaid SVG, not ASCII code blocks. */
test("renders documentation diagrams with Mermaid", async ({ page }) => {
  for (const route of ["/docs/dart-custom-macros/", "/docs/pipeline/"]) {
    await page.goto(route);
    const diagram = page.locator(".mermaid[data-processed='true'] svg");
    await expect(diagram).toHaveCount(1);
    await expect(diagram).toBeVisible();
  }
});

/** [playground.hosting]: Dart samples use distinct Prism token colors. */
test("syntax highlights Dart documentation samples", async ({ page }) => {
  await page.goto("/docs/dart-custom-macros/");

  const sample = page.locator("pre.language-dart").filter({ hasText: "AuditMacro" }).first();
  await expect(sample.locator(".token.keyword").first()).toBeVisible();
  await expect(sample.locator(".token.string").first()).toBeVisible();
  await expect(sample.locator(".token.class-name").first()).toBeVisible();

  const colors = await sample.locator(".token.keyword, .token.string, .token.class-name")
    .evaluateAll((tokens) => new Set(tokens.map((token) => getComputedStyle(token).color)).size);
  expect(colors).toBeGreaterThanOrEqual(3);
});

/** [playground.hosting]: Eleventy TechDoc emits navigable docs and blog routes into the site. */
test("serves the TechDoc documentation and blog structure", async ({ page }) => {
  await page.goto("/docs/");
  await expect(
    page.getByRole("heading", { level: 1, name: "Getting started with dmx" }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "What is a macro?" })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("link")).toHaveText([
    "Getting started",
    "Dart (Custom) Macros",
    "Macro catalogue",
    "Models in Markdown",
  ]);

  await page.goto("/docs/dart-custom-macros/");
  await expect(page.getByRole("heading", { level: 1, name: "Dart (Custom) Macros" })).toBeVisible();
  await expect(page.getByText("It does not need a Mustache template.")).toBeVisible();

  await page.goto("/blog/");
  await expect(page.getByRole("heading", { level: 1, name: "Blog" })).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Dart model code generation without part files",
    }),
  ).toBeVisible();
});

/** [playground.interface]: the shared navigation remains usable without horizontal overflow on mobile. */
test("keeps the shared navigation usable on a phone", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  await page.getByRole("button", { name: "Toggle navigation" }).click();
  const docs = page.locator(".nav-docs");
  await docs.locator("summary").click();
  await expect(docs.getByRole("link", { name: "Getting started" })).toBeVisible();

  await docs.getByRole("link", { name: "Getting started" }).click();
  await expect(page).toHaveURL(/\/docs\/$/);
  await expect(page.locator(".sidebar")).toBeVisible();
  await expect(page.locator("[data-nav-toggle]")).toBeVisible();

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(0);
});
