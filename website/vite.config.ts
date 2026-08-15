import { readFileSync } from "node:fs";
import { defineConfig } from "vitest/config";

// The site is served from the root of its own domain [playground.hosting], so
// the navigation's root-relative links need no prefix rewriting here.
const sharedNavigation = readFileSync(
  new URL("./src/_includes/partials/site-navigation.njk", import.meta.url),
  "utf8",
);

export default defineConfig({
  base: "./",
  // The site is a set of pages, not a single-page app, and GitHub Pages serves
  // it as static files: a path with no file behind it is a 404 there. Vite's
  // default answers one with 200 and the homepage, so the preview server the
  // Playwright suite drives will happily serve index.html in place of a
  // stylesheet — which is exactly how a build whose every asset reference was
  // wrong passed this suite and went dark in production [playground.hosting].
  appType: "mpa",
  plugins: [
    {
      name: "shared-navigation",
      transformIndexHtml(html) {
        return html.replace("<!-- shared-navigation -->", sharedNavigation);
      },
    },
  ],
  build: {
    target: "es2022",
    sourcemap: true,
    rollupOptions: {
      input: {
        playground: "playground.html",
      },
    },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
