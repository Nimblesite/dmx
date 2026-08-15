'use strict';

// What dmx adds to a Dart file, made visible [editor.dart-highlighting].
//
// The Dart grammar itself is the Dart extension's and is not installed here, so
// what these tests observe is exactly dmx's own contribution: the divider and
// the annotations. That is the half that has to be right — the Dart half is
// right already, and stays right as Dart changes.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');
const { EXTENSION, REPO, scopesAt, tokenizeLines } = require('./grammar-harness.js');

/// Every scope covering `needle` in a line of Dart.
const scopes = (line, needle) => scopesAt('source.dart', line, needle);

/// The scope prefix every annotation dmx owns is tokenized under.
const DMX = '.dmx';

test('the divider dmx owns reads as a divider, not as one more comment', async () => {
  const begin = await scopes('  //#region', '//#region');
  assert.ok(begin.includes('keyword.control.region.begin.dmx'), begin.join(' '));

  const end = await scopes('  //#endregion', '//#endregion');
  assert.ok(end.includes('keyword.control.region.end.dmx'), end.join(' '));
});

/// [emission.inline-backend.region-location]: a labelled fold is the author's
/// own and dmx never writes into it. Colouring it as dmx's would say otherwise.
test("a labelled fold is the author's, and is left alone", async () => {
  const found = await scopes('  //#region Helpers', '//#region');
  assert.ok(
    !found.some((scope) => scope.endsWith(DMX)),
    `dmx claimed the author's fold: ${found.join(' ')}`,
  );
});

test('a trailing comment that merely mentions a region is not a divider', async () => {
  const found = await scopes('  final int x = 1; //#region', '//#region');
  assert.ok(
    !found.some((scope) => scope.endsWith(DMX)),
    `a divider must own its line: ${found.join(' ')}`,
  );
});

/// The trigger and the macro it names are scoped apart: `@dmx` is the
/// annotation, and the string inside it is the thing that generates
/// [surface.annotations].
test('the trigger and its macro name are scoped apart', async () => {
  const trigger = await scopes("@dmx('model', {'fieldRename': 'snake'})", '@dmx');
  assert.ok(trigger.includes('entity.name.function.decorator.dmx'), trigger.join(' '));
  assert.ok(trigger.includes('punctuation.definition.annotation.dmx'), trigger.join(' '));

  const macro = await scopes("@dmx('model', {'fieldRename': 'snake'})", 'model');
  assert.ok(macro.includes('entity.name.function.macro.dmx'), macro.join(' '));
  assert.ok(!macro.includes('entity.name.function.decorator.dmx'), macro.join(' '));
});

test('a parameter annotation written mid-line is still an annotation', async () => {
  const found = await scopes(
    "  Future<Page> search({@dmx('query') int page = 1});",
    '@dmx',
  );
  assert.ok(found.includes('entity.name.function.decorator.dmx'), found.join(' '));
  const macro = await scopes(
    "  Future<Page> search({@dmx('query') int page = 1});",
    'query',
  );
  assert.ok(macro.includes('entity.name.function.macro.dmx'), macro.join(' '));
});

/// dmx colours what dmx owns. Dart's own annotations belong to the Dart
/// grammar, and an extension that repainted them would be lying about which
/// tool put them there.
test("Dart's own annotations are left to Dart", async () => {
  for (const annotation of ['@override', '@Deprecated', '@pragma']) {
    const found = await scopes(`  ${annotation}`, annotation);
    assert.ok(
      !found.some((scope) => scope.endsWith(DMX)),
      `dmx claimed ${annotation}: ${found.join(' ')}`,
    );
  }
});

/// A quoted annotation is a string, not an annotation — the one place `L:`
/// priority has to yield, and why the injection selector carries `-string`.
test('an annotation name inside a string literal is not an annotation', async () => {
  const grammar = path.join(EXTENSION, 'syntaxes/dmx-dart.injection.tmLanguage.json');
  const { injectionSelector } = JSON.parse(fs.readFileSync(grammar, 'utf8'));
  assert.match(injectionSelector, /-string/);
});

/// The drift gate. The examples are the macro catalogue's acceptance criterion
/// [catalogue.macros] and `tests/golden` is its corpus, so between them they
/// use every macro dmx ships — the built-ins and a user-defined one
/// [dartmacros], every one through the single `@dmx` trigger
/// [surface.annotations]. A trigger the grammar has not learned reads as
/// plain code, and the author only finds out by squinting.
test('every @dmx trigger the examples and the corpus use is highlighted', async () => {
  const sources = [
    path.join(REPO, 'examples/storefront/lib'),
    path.join(REPO, 'examples/dmx_sqlite_example/lib'),
    path.join(REPO, 'src/dmx/tests/golden'),
  ].flatMap(
    (dir) =>
      fs
        .readdirSync(dir)
        .filter((name) => name.endsWith('.dart'))
        .map((name) => path.join(dir, name)),
  );
  assert.ok(sources.length >= 20, `only ${sources.length} Dart sources found`);

  const seen = new Set();
  for (const file of sources) {
    const text = fs.readFileSync(file, 'utf8');
    for (const [number, line, tokens] of await tokenizeLines('source.dart', text)) {
      // Both the trigger and the macro name inside it must tokenize as dmx's:
      // the trigger as the annotation, the quoted name as the macro it names.
      for (const found of line.matchAll(/@dmx\(\s*'([A-Za-z.]+)'/g)) {
        const trigger = tokens.find(
          (candidate) =>
            candidate.startIndex <= found.index && candidate.endIndex > found.index,
        );
        assert.ok(
          trigger?.scopes.some((scope) => scope.endsWith(DMX)),
          `${path.basename(file)}:${number}: \`${found[0]}\` is not highlighted: ` +
            `${trigger?.scopes.join(' ')}`,
        );
        const nameIndex = found.index + found[0].indexOf(found[1]);
        const macro = tokens.find(
          (candidate) => candidate.startIndex <= nameIndex && candidate.endIndex > nameIndex,
        );
        assert.ok(
          macro?.scopes.includes('entity.name.function.macro.dmx'),
          `${path.basename(file)}:${number}: macro \`${found[1]}\` is not highlighted: ` +
            `${macro?.scopes.join(' ')}`,
        );
        seen.add(found[1]);
      }
    }
  }
  assert.ok(seen.size >= 30, `only ${seen.size} distinct macros exercised: ${[...seen]}`);
});
