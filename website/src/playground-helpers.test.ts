import { describe, expect, it } from "vitest";

import {
  errorMessage,
  formatElapsed,
  generatorResponse,
  mappedSelection,
  sourceMetrics,
} from "./playground-helpers";
import { PLAYGROUND_SAMPLES, sampleById } from "./samples";

describe("sourceMetrics", () => {
  it("counts empty and multiline sources exactly", () => {
    expect(sourceMetrics("")).toEqual({ lines: 0, characters: 0 });
    expect(sourceMetrics("one\ntwo\n")).toEqual({ lines: 3, characters: 8 });
  });
});

describe("status formatting", () => {
  it("formats measured durations and guards invalid values", () => {
    expect(formatElapsed(1.26)).toBe("1.3 ms");
    expect(formatElapsed(Number.NaN)).toBe("0.0 ms");
  });

  it("preserves useful errors and replaces opaque failures", () => {
    expect(errorMessage(new Error("DMX4001: invalid Dart"))).toBe("DMX4001: invalid Dart");
    expect(errorMessage("bad input")).toBe("bad input");
    expect(errorMessage(null)).toBe("The generator returned an unknown error.");
  });
});

describe("generatorResponse", () => {
  it("accepts the WASM pair and rejects malformed responses", () => {
    expect(generatorResponse([true, "generated"])).toEqual([true, "generated"]);
    expect(generatorResponse([false, "DMX4001"])).toEqual([false, "DMX4001"]);
    expect(generatorResponse([true])).toBeNull();
    expect(generatorResponse(["true", "generated"])).toBeNull();
    expect(generatorResponse(null)).toBeNull();
  });
});

describe("mappedSelection", () => {
  const before = "class A {}\n//#region dmx\nold body\n//#endregion\n";
  const after = "class A {}\n//#region dmx\nregenerated body\n//#endregion\n";

  it("keeps a caret that sits before the regenerated region", () => {
    const caret = { start: 8, end: 8 };
    expect(mappedSelection(before, after, caret)).toEqual(caret);
  });

  it("shifts a caret after the regenerated region by the length difference", () => {
    const caret = { start: before.length, end: before.length };
    expect(mappedSelection(before, after, caret)).toEqual({
      start: after.length,
      end: after.length,
    });
  });

  it("collapses a selection inside the regenerated region to its start", () => {
    const prefixLength = "class A {}\n//#region dmx\n".length;
    expect(mappedSelection(before, after, { start: prefixLength + 1, end: prefixLength + 3 }))
      .toEqual({ start: prefixLength, end: prefixLength });
  });

  it("returns identical selections for identical text", () => {
    const caret = { start: 4, end: 9 };
    expect(mappedSelection(before, before, caret)).toEqual(caret);
  });
});

describe("playground samples", () => {
  it("resolves every macro and falls back to Model", () => {
    for (const sample of PLAYGROUND_SAMPLES) {
      expect(sampleById(sample.id)).toBe(sample);
      expect(sample.template.trim().length).toBeGreaterThan(0);
      expect(sample.source, `example name ${sample.name} appears in its source`)
        .toContain(sample.name);
    }
    expect(sampleById("not-a-macro").id).toBe("model");
  });
});
