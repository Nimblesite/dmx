'use strict';

// The real tokenizer, wired the way the manifest wires it.
//
// `vscode-textmate` and `vscode-oniguruma` are the same tokenizer and the same
// regex engine VS Code itself runs, loaded here with the same `injectTo` wiring
// the manifest declares — so a tag that scopes correctly under these tests
// scopes correctly in the editor.
//
// The Dart grammar belongs to the Dart extension and is not installed here.
// That is the degraded case worth pinning rather than papering over: `source.dart`
// resolves to an empty grammar, so what these tests observe is exactly what dmx
// contributes and nothing the Dart extension would have contributed anyway.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const oniguruma = require('vscode-oniguruma');
const textmate = require('vscode-textmate');

const EXTENSION = path.join(__dirname, '..');
// The extension lives at src/editors/vscode, so the repository root is three
// directories above it.
const REPO = path.join(EXTENSION, '..', '..', '..');
const MANIFEST = JSON.parse(fs.readFileSync(path.join(EXTENSION, 'package.json'), 'utf8'));

/// The scope every host grammar stands in for when it is not installed.
const ABSENT = { scopeName: 'source.dart', patterns: [] };

/// The manifest is the single source of truth for which grammar owns which
/// scope, so these tests cannot drift from what ships.
const GRAMMARS = new Map(
  MANIFEST.contributes.grammars.map((grammar) => [
    grammar.scopeName,
    path.join(EXTENSION, grammar.path),
  ]),
);

const INJECTIONS = MANIFEST.contributes.grammars
  .filter((grammar) => Array.isArray(grammar.injectTo))
  .flatMap((grammar) => grammar.injectTo.map((target) => [target, grammar.scopeName]));

const onigLib = oniguruma
  .loadWASM(fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm')).buffer)
  .then(() => ({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (line) => new oniguruma.OnigString(line),
  }));

const registry = new textmate.Registry({
  onigLib,
  loadGrammar: async (scopeName) => {
    const file = GRAMMARS.get(scopeName);
    if (file) {
      return textmate.parseRawGrammar(fs.readFileSync(file, 'utf8'), file);
    }
    // A host grammar this repo does not ship — see ABSENT.
    return scopeName === ABSENT.scopeName ? ABSENT : null;
  },
  getInjections: (scopeName) =>
    INJECTIONS.filter(([target]) => target === scopeName).map(([, injection]) => injection),
});

/// Every scope covering `needle` in `line`, tokenized as `root`. A tag never
/// occupies one token: the braces, the sigil and the name are scoped separately
/// so a theme can colour punctuation apart from what it delimits.
async function scopesAt(root, line, needle) {
  const at = line.indexOf(needle);
  assert.notEqual(at, -1, `\`${needle}\` is not in \`${line}\``);
  const grammar = await registry.loadGrammar(root);
  return grammar
    .tokenizeLine(line, textmate.INITIAL)
    .tokens.filter((token) => token.startIndex < at + needle.length && token.endIndex > at)
    .flatMap((token) => token.scopes);
}

/// `text` tokenized as `root`, line by line, carrying the rule stack across
/// lines the way an editor does. Yields `[lineNumber, line, tokens]`.
async function tokenizeLines(root, text) {
  const grammar = await registry.loadGrammar(root);
  let rules = textmate.INITIAL;
  const out = [];
  for (const [number, line] of text.split('\n').entries()) {
    const result = grammar.tokenizeLine(line, rules);
    rules = result.ruleStack;
    out.push([number + 1, line, result.tokens]);
  }
  return out;
}

module.exports = { REPO, EXTENSION, MANIFEST, scopesAt, tokenizeLines };
