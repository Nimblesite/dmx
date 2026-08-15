export interface SourceMetrics {
  readonly lines: number;
  readonly characters: number;
}

export type GeneratorResponse = readonly [succeeded: boolean, result: string];

export function sourceMetrics(source: string): SourceMetrics {
  return {
    lines: source.length === 0 ? 0 : source.split("\n").length,
    characters: source.length,
  };
}

export function formatElapsed(milliseconds: number): string {
  if (!Number.isFinite(milliseconds) || milliseconds < 0) {
    return "0.0 ms";
  }
  return `${milliseconds.toFixed(1)} ms`;
}

export function errorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }
  if (typeof error === "string" && error.trim().length > 0) {
    return error;
  }
  return "The generator returned an unknown error.";
}

export interface SelectionRange {
  readonly start: number;
  readonly end: number;
}

/**
 * Maps a caret or selection from the text an edit produced onto the text the
 * generator spliced back, so typing above the dividers never moves the caret
 * [playground.interface.inline-mode].
 *
 * A selection inside the shared prefix keeps its offsets; one inside the
 * shared suffix shifts by the length difference; one inside the regenerated
 * region collapses to the end of the shared prefix.
 */
export function mappedSelection(
  previous: string,
  next: string,
  selection: SelectionRange,
): SelectionRange {
  const shortest = Math.min(previous.length, next.length);
  let prefix = 0;
  while (prefix < shortest && previous[prefix] === next[prefix]) {
    prefix += 1;
  }
  if (selection.end <= prefix) {
    return selection;
  }
  let suffix = 0;
  while (
    suffix < shortest - prefix &&
    previous[previous.length - 1 - suffix] === next[next.length - 1 - suffix]
  ) {
    suffix += 1;
  }
  const shift = next.length - previous.length;
  return selection.start >= previous.length - suffix
    ? { start: selection.start + shift, end: selection.end + shift }
    : { start: prefix, end: prefix };
}

export function generatorResponse(value: unknown): GeneratorResponse | null {
  if (!Array.isArray(value) || value.length < 2) {
    return null;
  }
  const succeeded: unknown = value[0];
  const result: unknown = value[1];
  return typeof succeeded === "boolean" && typeof result === "string"
    ? [succeeded, result]
    : null;
}
