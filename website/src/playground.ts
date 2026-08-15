import init, { generate_with_template } from "../pkg/dmx_wasm.js";

import {
  errorMessage,
  formatElapsed,
  generatorResponse,
  mappedSelection,
  sourceMetrics,
} from "./playground-helpers";
import { PLAYGROUND_SAMPLES, sampleById } from "./samples";
import { highlightCode, syncHighlightScroll } from "./syntax-highlight";

type StatusKind = "loading" | "ready" | "success" | "error";
type EditorKind = "dart" | "template";
const MAX_INPUT_LENGTH = 50_000;

interface PlaygroundElements {
  readonly macro: HTMLSelectElement;
  readonly input: HTMLTextAreaElement;
  readonly templateInput: HTMLTextAreaElement;
  readonly sourceHighlight: HTMLElement;
  readonly templateHighlight: HTMLElement;
  readonly output: HTMLElement;
  readonly resetButton: HTMLButtonElement;
  readonly copyButton: HTMLButtonElement;
  readonly status: HTMLElement;
  readonly statusText: HTMLElement;
  readonly sampleDescription: HTMLElement;
  readonly inputMetrics: HTMLElement;
  readonly outputMetrics: HTMLElement;
  readonly inputHint: HTMLElement;
  readonly inlineToggle: HTMLInputElement;
  readonly editorTabs: readonly HTMLButtonElement[];
  readonly editorPanels: readonly HTMLElement[];
}

function elements(root: HTMLElement): PlaygroundElements | null {
  const macro = root.querySelector<HTMLSelectElement>("[data-macro-select]");
  const input = root.querySelector<HTMLTextAreaElement>("[data-source-input]");
  const templateInput = root.querySelector<HTMLTextAreaElement>("[data-template-input]");
  const sourceHighlight = root.querySelector<HTMLElement>("[data-source-highlight]");
  const templateHighlight = root.querySelector<HTMLElement>("[data-template-highlight]");
  const output = root.querySelector<HTMLElement>("[data-source-output]");
  const resetButton = root.querySelector<HTMLButtonElement>("[data-reset]");
  const copyButton = root.querySelector<HTMLButtonElement>("[data-copy]");
  const status = root.querySelector<HTMLElement>("[data-status]");
  const statusText = root.querySelector<HTMLElement>("[data-status-text]");
  const sampleDescription = root.querySelector<HTMLElement>("[data-sample-description]");
  const inputMetrics = root.querySelector<HTMLElement>("[data-input-metrics]");
  const outputMetrics = root.querySelector<HTMLElement>("[data-output-metrics]");
  const inputHint = root.querySelector<HTMLElement>("[data-input-hint]");
  const inlineToggle = root.querySelector<HTMLInputElement>("[data-inline-toggle]");
  const editorTabs = Array.from(root.querySelectorAll<HTMLButtonElement>("[data-editor-tab]"));
  const editorPanels = Array.from(root.querySelectorAll<HTMLElement>("[data-editor-panel]"));
  if (
    macro === null || input === null || templateInput === null ||
    sourceHighlight === null || templateHighlight === null || output === null ||
    resetButton === null || copyButton === null ||
    status === null || statusText === null || sampleDescription === null ||
    inputMetrics === null || outputMetrics === null || inputHint === null ||
    inlineToggle === null ||
    editorTabs.length !== 2 || editorPanels.length !== 2
  ) {
    return null;
  }
  return {
    macro, input, templateInput, sourceHighlight, templateHighlight, output,
    resetButton, copyButton,
    status, statusText, sampleDescription, inputMetrics, outputMetrics, inputHint,
    inlineToggle, editorTabs, editorPanels,
  };
}

function setStatus(ui: PlaygroundElements, kind: StatusKind, message: string): void {
  ui.status.dataset.state = kind;
  ui.statusText.textContent = message;
}

function updateMetrics(elementToUpdate: HTMLElement, source: string): void {
  const metrics = sourceMetrics(source);
  elementToUpdate.textContent = `${metrics.lines} lines · ${metrics.characters} chars`;
}

function editorKind(value: string | undefined): EditorKind {
  return value === "template" ? "template" : "dart";
}

function editorValue(ui: PlaygroundElements, kind: EditorKind): string {
  return kind === "dart" ? ui.input.value : ui.templateInput.value;
}

function highlightInput(ui: PlaygroundElements, kind: EditorKind): void {
  if (kind === "dart") {
    highlightCode(ui.sourceHighlight, ui.input.value, "dart");
    syncHighlightScroll(ui.input, ui.sourceHighlight);
    return;
  }
  highlightCode(ui.templateHighlight, ui.templateInput.value, "mustache");
  syncHighlightScroll(ui.templateInput, ui.templateHighlight);
}

function updateInputMeta(ui: PlaygroundElements, kind: EditorKind): void {
  updateMetrics(ui.inputMetrics, editorValue(ui, kind));
  ui.inputHint.textContent = kind === "template"
    ? "Edit the real Mustache template · the output regenerates live"
    : ui.inlineToggle.checked
      ? "Generated members land between the //#region dividers as you type · exactly what dmx watch writes"
      : "Edit Dart freely · the output regenerates live";
}

function activateEditor(
  ui: PlaygroundElements,
  kind: EditorKind,
  moveFocus: boolean,
): void {
  for (const tab of ui.editorTabs) {
    const selected = editorKind(tab.dataset.editorTab) === kind;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    if (selected && moveFocus) {
      tab.focus();
    }
  }
  for (const panel of ui.editorPanels) {
    panel.hidden = editorKind(panel.dataset.editorPanel) !== kind;
  }
  updateInputMeta(ui, kind);
}

function inputLimitMessage(ui: PlaygroundElements): string | null {
  if (ui.input.value.length > MAX_INPUT_LENGTH) {
    return `Dart is too large · ${MAX_INPUT_LENGTH.toLocaleString()} characters maximum`;
  }
  if (ui.templateInput.value.length > MAX_INPUT_LENGTH) {
    return `Template is too large · ${MAX_INPUT_LENGTH.toLocaleString()} characters maximum`;
  }
  return null;
}

export async function mountPlayground(root: HTMLElement): Promise<void> {
  const ui = elements(root);
  if (ui === null) {
    return;
  }
  let wasmReady = false;
  let lastGenerationSucceeded = false;
  let activeEditor: EditorKind = "dart";

  const showFailure = (message: string): void => {
    highlightCode(ui.output, message, "dart");
    updateMetrics(ui.outputMetrics, message);
    lastGenerationSucceeded = false;
    ui.copyButton.disabled = true;
    setStatus(ui, "error", message);
  };

  const loadSample = (generateAfterLoad: boolean): void => {
    const sample = sampleById(ui.macro.value);
    ui.input.value = sample.source;
    ui.templateInput.value = sample.template;
    highlightInput(ui, "dart");
    highlightInput(ui, "template");
    ui.sampleDescription.textContent = sample.description;
    updateInputMeta(ui, activeEditor);
    if (generateAfterLoad && wasmReady) {
      runGenerator();
    }
  };

  // The browser twin of dmx watch: the regenerated file replaces the Dart
  // editor's content in place, and the caret survives because user edits and
  // generated members never overlap [playground.interface.inline-mode].
  const spliceInline = (result: string): void => {
    if (result !== ui.input.value) {
      const selection = mappedSelection(ui.input.value, result, {
        start: ui.input.selectionStart,
        end: ui.input.selectionEnd,
      });
      ui.input.value = result;
      ui.input.setSelectionRange(selection.start, selection.end);
      highlightInput(ui, "dart");
    }
    updateInputMeta(ui, activeEditor);
  };

  const runGenerator = (): void => {
    if (!wasmReady) {
      setStatus(ui, "loading", "The compiler is still loading…");
      return;
    }

    const limitMessage = inputLimitMessage(ui);
    if (limitMessage !== null) {
      showFailure(limitMessage);
      return;
    }

    const startedAt = performance.now();
    try {
      const response = generatorResponse(generate_with_template(
        ui.input.value,
        ui.templateInput.value,
      ));
      if (response === null) {
        showFailure("The compiler returned an invalid response.");
        return;
      }
      const [succeeded, result] = response;
      const inline = ui.inlineToggle.checked;
      if (succeeded && inline) {
        spliceInline(result);
      } else {
        highlightCode(ui.output, result, "dart");
        updateMetrics(ui.outputMetrics, result);
      }
      lastGenerationSucceeded = succeeded;
      ui.copyButton.disabled = !succeeded || result.length === 0;
      const elapsed = formatElapsed(performance.now() - startedAt);
      setStatus(
        ui,
        succeeded ? "success" : "error",
        succeeded
          ? inline
            ? `Spliced inline in ${elapsed} · exactly what dmx watch writes`
            : `Rendered your template locally in ${elapsed}`
          : result,
      );
    } catch (error: unknown) {
      showFailure(errorMessage(error));
    }
  };

  ui.macro.replaceChildren(
    ...PLAYGROUND_SAMPLES.map((sample) => {
      const option = document.createElement("option");
      option.value = sample.id;
      option.textContent = `${sample.name} · @dmx('${sample.id}')`;
      return option;
    }),
  );
  loadSample(false);

  ui.macro.addEventListener("change", () => loadSample(true));
  const regenerateFromEdit = (kind: EditorKind): void => {
    activeEditor = kind;
    highlightInput(ui, kind);
    updateInputMeta(ui, kind);
    runGenerator();
  };
  ui.input.addEventListener("input", () => regenerateFromEdit("dart"));
  ui.templateInput.addEventListener("input", () => regenerateFromEdit("template"));
  ui.input.addEventListener("scroll", () => syncHighlightScroll(ui.input, ui.sourceHighlight));
  ui.templateInput.addEventListener("scroll", () =>
    syncHighlightScroll(ui.templateInput, ui.templateHighlight)
  );

  for (const tab of ui.editorTabs) {
    tab.addEventListener("click", () => {
      activeEditor = editorKind(tab.dataset.editorTab);
      activateEditor(ui, activeEditor, false);
    });
    tab.addEventListener("keydown", (event) => {
      const destination = event.key === "ArrowRight" || event.key === "End"
        ? "template"
        : event.key === "ArrowLeft" || event.key === "Home"
          ? "dart"
          : null;
      if (destination !== null) {
        event.preventDefault();
        activeEditor = destination;
        activateEditor(ui, destination, true);
      }
    });
  }
  ui.inlineToggle.addEventListener("change", () => {
    root.dataset.mode = ui.inlineToggle.checked ? "inline" : "split";
    updateInputMeta(ui, activeEditor);
    if (wasmReady) {
      runGenerator();
    }
  });
  ui.resetButton.addEventListener("click", () => loadSample(true));
  ui.copyButton.addEventListener("click", async () => {
    if (!lastGenerationSucceeded) {
      return;
    }
    try {
      await navigator.clipboard.writeText(
        ui.inlineToggle.checked ? ui.input.value : ui.output.textContent ?? "",
      );
      setStatus(ui, "success", "Generated Dart copied to clipboard");
    } catch (error: unknown) {
      setStatus(ui, "error", `Could not copy: ${errorMessage(error)}`);
    }
  });

  try {
    setStatus(ui, "loading", "Loading the Rust compiler…");
    await init();
    wasmReady = true;
    setStatus(ui, "ready", "Compiler ready · everything stays on this device");
    runGenerator();
  } catch (error: unknown) {
    ui.resetButton.disabled = true;
    showFailure(`Compiler failed to load: ${errorMessage(error)}`);
  }
}
