import { expect, type Locator, type Page, test } from "@playwright/test";

async function openPlayground(page: Page): Promise<void> {
  await page.goto("/playground.html");
  await expect(page.locator("[data-status]")).toHaveAttribute("data-state", "success");
}

async function openSplitView(page: Page): Promise<void> {
  await openPlayground(page);
  await page.getByRole("checkbox", { name: "Inline watch mode" }).uncheck();
}

function typographyOf(locator: Locator) {
  return locator.evaluate((element) => {
    const style = getComputedStyle(element);
    return { fontFamily: style.fontFamily, fontSize: style.fontSize, lineHeight: style.lineHeight };
  });
}

const DART_SOURCE = `import 'package:dmx/dmx.dart';

@dmx('model')
class BrowserProfile {
  const BrowserProfile({required this.handle});

  final String handle;
}
`;

const INVALID_DART = "@dmx('model') class Broken { final int ===; }";
const CUSTOM_TEMPLATE_MEMBER = "  String get playwrightMarker => '{{className}}';";

/** [playground.interface.inline-mode]: generation splices into the Dart editor between the dividers, like dmx watch. */
test("splices generated members inline between the dividers by default", async ({ page }) => {
  await openPlayground(page);

  const status = page.locator("[data-status]");
  const sourceInput = page.getByLabel("Dart source to generate");
  await expect(page.getByRole("checkbox", { name: "Inline watch mode" })).toBeChecked();
  await expect(page.locator(".output-pane")).toBeHidden();

  await sourceInput.fill(DART_SOURCE);
  await expect(status).toHaveAttribute("data-state", "success");
  await expect(page.locator("[data-status-text]")).toContainText("dmx watch");
  await expect(sourceInput).toHaveValue(/\/\/#region/);
  await expect(sourceInput).toHaveValue(/\/\/#endregion/);
  await expect(sourceInput).toHaveValue(/BrowserProfile copyWith/);

  await sourceInput.fill(INVALID_DART);
  await expect(status).toHaveAttribute("data-state", "error");
  await expect(sourceInput).toHaveValue(INVALID_DART);
});

/** [playground.interface.inline-mode]: the checkbox flips between the inline file and the split output pane. */
test("toggles between inline watch mode and the split view", async ({ page }) => {
  await openPlayground(page);

  const status = page.locator("[data-status]");
  const outputPane = page.locator(".output-pane");
  const inlineToggle = page.getByRole("checkbox", { name: "Inline watch mode" });

  await inlineToggle.uncheck();
  await expect(outputPane).toBeVisible();
  await expect(page.locator("[data-source-output]")).toContainText("//#region");
  await expect(page.locator("[data-status-text]")).toContainText("Rendered your template locally");

  await inlineToggle.check();
  await expect(outputPane).toBeHidden();
  await expect(page.getByLabel("Dart source to generate")).toHaveValue(/\/\/#region/);
  await expect(status).toHaveAttribute("data-state", "success");
});

/** [playground.interface]: code and template edits regenerate live in the real browser WASM. */
test("regenerates as the Dart and Mustache template are edited", async ({ page }) => {
  const wasmResponsePromise = page.waitForResponse((response) =>
    new URL(response.url()).pathname.endsWith(".wasm")
  );

  await page.goto("/playground.html");

  const wasmResponse = await wasmResponsePromise;
  expect(await wasmResponse.headerValue("content-type")).toContain("application/wasm");

  const status = page.locator("[data-status]");
  const statusText = page.locator("[data-status-text]");
  const sourceInput = page.getByLabel("Dart source to generate");
  const templateInput = page.getByLabel("Mustache output template");
  const output = page.locator("[data-source-output]");

  await expect(status).toHaveAttribute("data-state", "success");
  await page.getByRole("checkbox", { name: "Inline watch mode" }).uncheck();
  await sourceInput.fill(DART_SOURCE);
  await expect(output).toContainText("BrowserProfile copyWith");

  await page.getByRole("tab", { name: "Mustache template" }).click();
  const repositoryTemplate = await templateInput.inputValue();
  await templateInput.fill(`${repositoryTemplate}\n\n${CUSTOM_TEMPLATE_MEMBER}\n`);

  await expect(status).toHaveAttribute("data-state", "success");
  await expect(statusText).toContainText("Rendered your template locally");
  await expect(output).toContainText("String get playwrightMarker => 'BrowserProfile';");

  await page.getByRole("tab", { name: "Dart source" }).click();
  await sourceInput.fill(INVALID_DART);

  await expect(status).toHaveAttribute("data-state", "error");
  await expect(statusText).toContainText("DMX4001");
});

/** [playground.wasm]: malformed user templates fail as data in the browser. */
test("reports an invalid supplied template without a JavaScript exception", async ({ page }) => {
  await openPlayground(page);

  const status = page.locator("[data-status]");
  await page.getByRole("tab", { name: "Mustache template" }).click();
  await page.getByLabel("Mustache output template").fill("{{> unavailable}}");

  await expect(status).toHaveAttribute("data-state", "error");
  await expect(page.locator("[data-status-text]")).toContainText("bad user template");
});

/** [playground.interface]: the dedicated page fills the viewport, edge to edge. */
test("fills the desktop viewport with no page scroll", async ({ page }) => {
  await openSplitView(page);

  const viewport = page.viewportSize();
  expect(viewport).not.toBeNull();
  const shell = await page.locator(".playground-shell").boundingBox();
  expect(shell).not.toBeNull();
  if (viewport === null || shell === null) {
    return;
  }
  expect(shell.width).toBeGreaterThanOrEqual(viewport.width - 1);
  expect(shell.y + shell.height).toBeGreaterThanOrEqual(viewport.height - 1);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollHeight - document.documentElement.clientHeight,
  );
  expect(overflow).toBeLessThanOrEqual(0);

  const outputScrolls = await page.locator(".output-pane pre").evaluate((pane) => {
    pane.scrollTop = pane.scrollHeight;
    return pane.scrollHeight > pane.clientHeight && pane.scrollTop > 0;
  });
  expect(outputScrolls, "generated output scrolls inside its pane").toBe(true);
});

/** [playground.interface]: the landing page links through to the playground page. */
test("navigates from the landing page to the playground", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Try it in the browser" }).click();
  await expect(page).toHaveURL(/playground\.html$/);
  await expect(page.locator("[data-status]")).toHaveAttribute("data-state", "success");
});

/** [playground.interface]: both input editors remain usable on a phone viewport. */
test("keeps the Dart and template inputs usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openPlayground(page);

  const templateTab = page.getByRole("tab", { name: "Mustache template" });
  await templateTab.click();

  await expect(templateTab).toHaveAttribute("aria-selected", "true");
  await expect(page.getByLabel("Mustache output template")).toBeVisible();
  await expect(page.getByRole("button", { name: "Reset" })).toBeVisible();
});

/** [playground.interface]: the editable and highlighted text layers stay aligned. */
test("keeps highlighted Dart aligned with the input while selecting text", async ({ page }) => {
  await openPlayground(page);

  const sourceInput = page.getByLabel("Dart source to generate");
  const sourceHighlight = page.locator("[data-source-highlight]");
  await sourceInput.selectText();

  expect(await typographyOf(sourceHighlight)).toEqual(await typographyOf(sourceInput));
});

/** [playground.interface]: every code surface renders language-aware tokens. */
test("syntax highlights both inputs and the generated Dart", async ({ page }) => {
  await openSplitView(page);
  await page.getByLabel("Dart source to generate").fill(DART_SOURCE);
  await expect(page.locator("[data-source-highlight]")).toContainText("BrowserProfile");
  await page.getByRole("tab", { name: "Mustache template" }).click();
  const templateInput = page.getByLabel("Mustache output template");
  await templateInput.fill(`${await templateInput.inputValue()}\n${CUSTOM_TEMPLATE_MEMBER}\n`);
  await expect(page.locator("[data-template-highlight]")).toContainText("playwrightMarker");
  await expect(page.locator("[data-status]")).toHaveAttribute("data-state", "success");

  const panes = [
    { language: "dart", pane: page.locator('[data-editor-panel="dart"]') },
    { language: "mustache", pane: page.locator('[data-editor-panel="template"]') },
    { language: "dart", pane: page.locator(".output-pane") },
  ];

  for (const { language, pane } of panes) {
    await expect(pane).toHaveAttribute("data-highlight-language", language);
    const tokens = pane.locator("[data-syntax-token]");
    expect(await tokens.count(), `${language} pane has token spans`).toBeGreaterThan(5);
    const colors = await tokens.evaluateAll((elements) =>
      elements.map((element) => getComputedStyle(element).color)
    );
    expect(new Set(colors).size, `${language} pane uses multiple token colors`).toBeGreaterThan(1);
  }

  await page.getByRole("tab", { name: "Dart source" }).click();
  await expect(
    page.locator('[data-source-highlight] [data-syntax-token="keyword"]').first(),
  ).toBeVisible();
  await page.getByRole("tab", { name: "Mustache template" }).click();
  await expect(
    page.locator('[data-template-highlight] [data-syntax-token="handlebars"]').first(),
  ).toBeVisible();
  await expect(
    page.locator('[data-template-highlight] [data-syntax-token="keyword"]').first(),
  ).toBeVisible();
  await expect(
    page.locator('[data-source-output] [data-syntax-token="keyword"]').first(),
  ).toBeVisible();
});
