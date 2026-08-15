import { describe, expect, it } from "vitest";

import { generate_with_template } from "../../src/dmx/target/wasm-node/dmx_wasm.js";
import { PLAYGROUND_SAMPLES, sampleById } from "./samples";

describe("playground WASM samples [playground.wasm]", () => {
  it("generates an inline region for every sample shown in the interface", () => {
    expect(PLAYGROUND_SAMPLES).toHaveLength(5);

    for (const sample of PLAYGROUND_SAMPLES) {
      const [succeeded, output] = generate_with_template(sample.source, sample.template);

      expect(succeeded, sample.id).toBe(true);
      expect(typeof output, sample.id).toBe("string");
      if (succeeded === true && typeof output === "string") {
        expect(output, sample.id).toContain("//#region");
        expect(output, sample.id).toContain("//#endregion");
      }
    }
  });

  it("renders an edited template instead of the built-in output", () => {
    const sample = sampleById("model");
    const marker = "// customised in the playground";
    const [succeeded, output] = generate_with_template(
      sample.source,
      `${sample.template}\n${marker}`,
    );

    expect(succeeded).toBe(true);
    expect(output).toContain(marker);
  });
});
