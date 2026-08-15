import { highlightText, languages, tokenize } from "prism-code-editor/prism";
import "prism-code-editor/prism/languages/dart";
import "prism-code-editor/prism/languages/handlebars";
import { embeddedIn } from "prism-code-editor/prism/utils";
import "prism-code-editor/languages/handlebars";

export type HighlightLanguage = "dart" | "mustache";

languages["dart-mustache"] = {
  ...languages.handlebars,
  [tokenize]: embeddedIn("dart"),
};

/** Renders escaped, language-aware token spans into one code surface. */
export function highlightCode(
  element: HTMLElement,
  source: string,
  language: HighlightLanguage,
): void {
  const grammar = language === "mustache" ? "dart-mustache" : "dart";
  element.innerHTML = highlightText(source, grammar);
  element.className = `language-${grammar}`;

  for (const token of element.querySelectorAll<HTMLElement>("span.token")) {
    const tokenClass = Array.from(token.classList)
      .find((className) => className !== "token" && !className.startsWith("language-"));
    if (tokenClass !== undefined) {
      token.dataset.syntaxToken = tokenClass;
    }
  }
}

/** Keeps a highlighted input overlay aligned with its real textarea. */
export function syncHighlightScroll(input: HTMLTextAreaElement, code: HTMLElement): void {
  const scroller = code.parentElement;
  if (scroller instanceof HTMLElement) {
    scroller.scrollTop = input.scrollTop;
    scroller.scrollLeft = input.scrollLeft;
  }
}
