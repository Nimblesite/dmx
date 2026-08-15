'use strict';

// The template grammar, run for real [editor.template-highlighting].
//
// The tokenizer, the regex engine and the `injectTo` wiring all come from
// grammar-harness.js, which loads them the way VS Code does.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');
const { REPO, scopesAt, tokenizeLines } = require('./grammar-harness.js');

/// Every scope covering `needle` in a template line.
const scopes = (line, needle) => scopesAt('source.mustache', line, needle);

test('a section opens, names itself, and closes', async () => {
  const open = await scopes('{{#fields}}', '{{#fields}}');
  assert.ok(open.includes('keyword.control.section.mustache'), open.join(' '));
  assert.ok(open.includes('entity.name.tag.mustache'), open.join(' '));

  const close = await scopes('{{/fields}}', '{{/fields}}');
  assert.ok(close.includes('keyword.control.section.mustache'), close.join(' '));

  const inverted = await scopes('{{^wantsJson}}', '{{^wantsJson}}');
  assert.ok(inverted.includes('keyword.control.section.mustache'), inverted.join(' '));
});

test('an interpolation is a variable', async () => {
  const found = await scopes('  final {{type}} {{name}};', '{{type}}');
  assert.ok(found.includes('variable.other.mustache'), found.join(' '));
});

test('a triple-brace tag is an unescaped variable, not a brace and a tag', async () => {
  const found = await scopes('{{{resultExpr}}},', '{{{resultExpr}}}');
  assert.ok(found.includes('variable.other.unescaped.mustache'), found.join(' '));
  assert.ok(!found.includes('variable.other.mustache'), found.join(' '));
});

test('a comment is a comment', async () => {
  const found = await scopes('{{! not emitted }}', '{{! not emitted }}');
  assert.ok(found.includes('comment.block.mustache'), found.join(' '));
});

test('a partial names the partial', async () => {
  const found = await scopes('{{> equality}}', '{{> equality}}');
  assert.ok(found.includes('keyword.control.partial.mustache'), found.join(' '));
  assert.ok(found.includes('entity.name.tag.mustache'), found.join(' '));
});

/// Templates put tags inside quotes constantly — `'{{className}}.{{name}}'`.
/// With the Dart grammar installed those quotes are a string literal, and it is
/// the injection's `L:` priority that keeps the tag from being swallowed by it;
/// here, with no Dart grammar, this pins the tag half of that pair.
test('a tag written inside quotes is still a tag', async () => {
  const found = await scopes(`      '{{className}}.{{name}}',`, '{{className}}');
  assert.ok(found.includes('variable.other.mustache'), found.join(' '));
});

/// The corpus test: every tag in every template that actually ships must be
/// recognised as one. A tag the grammar does not know reads as plain code, and
/// the author only finds out by squinting.
test('every tag in every shipped template is scoped as mustache', async () => {
  // The SHIPPED templates: the ones compiled into the binary, which live with
  // the crate. The catalogue previews under examples/ have no builder yet.
  const dir = path.join(REPO, 'src/dmx/templates');
  const templates = fs.readdirSync(dir).filter((name) => name.endsWith('.mustache'));
  assert.ok(templates.length >= 10, `only ${templates.length} templates found`);
  let checked = 0;
  for (const name of templates) {
    const source = fs.readFileSync(path.join(dir, name), 'utf8');
    for (const [number, line, tokens] of await tokenizeLines('source.mustache', source)) {
      for (const tag of line.matchAll(/\{\{+[^{}]*\}\}+/g)) {
        const token = tokens.find(
          (candidate) => candidate.startIndex <= tag.index && candidate.endIndex > tag.index,
        );
        assert.ok(
          token && token.scopes.some((scope) => scope.endsWith('.mustache')),
          `${name}:${number}: \`${tag[0]}\` is not a mustache tag: ${token?.scopes.join(' ')}`,
        );
        checked += 1;
      }
    }
  }
  assert.ok(checked > 100, `only ${checked} tags found across ${templates.length} templates`);
});
