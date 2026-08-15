import "./styles/base.css";
import "./styles/layout.css";
import "./styles/playground.css";
import "./styles/sections.css";
import "./styles/responsive.css";

import { errorMessage } from "./playground-helpers";

async function mountInteractivePlayground(): Promise<void> {
  const playground = document.querySelector<HTMLElement>("[data-playground]");
  if (playground === null) {
    return;
  }
  try {
    const playgroundModule = await import("./playground");
    await playgroundModule.mountPlayground(playground);
  } catch (error: unknown) {
    const message = `Compiler failed to load: ${errorMessage(error)}`;
    const status = playground.querySelector<HTMLElement>("[data-status]");
    const statusText = playground.querySelector<HTMLElement>("[data-status-text]");
    const output = playground.querySelector<HTMLElement>("[data-source-output]");
    if (status !== null) {
      status.dataset.state = "error";
    }
    if (statusText !== null) {
      statusText.textContent = message;
    }
    if (output !== null) {
      output.textContent = message;
    }
  }
}

void mountInteractivePlayground();
