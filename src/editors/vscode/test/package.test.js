'use strict';

// What the marketplace and the packager see [editor.extension.bundle].
//
// The manifest is a set of promises about files: an entry point, an icon, a
// grammar per scope, a command per palette entry. Every one of them is kept by
// a path that has to exist, has to be readable, and has to survive
// `.vscodeignore` on the way into the VSIX. None of that fails at build time —
// it fails as a broken listing, a missing icon, or a command that reports
// "command not found" to whoever tried it.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { test } = require('node:test');
const { EXTENSION, MANIFEST } = require('./grammar-harness.js');

const at = (relative) => path.join(EXTENSION, relative);
const read = (relative) => fs.readFileSync(at(relative), 'utf8');

/// Everything the manifest points at, as paths relative to the extension root.
function promised() {
  const { contributes } = MANIFEST;
  return [
    MANIFEST.main,
    MANIFEST.icon,
    ...contributes.grammars.map((grammar) => grammar.path),
    ...contributes.languages.map((language) => language.configuration),
  ].map((file) => file.replace(/^\.\//, ''));
}

test('the marketplace has everything it refuses to list an extension without', () => {
  for (const field of [
    'name',
    'displayName',
    'description',
    'version',
    'publisher',
    'license',
    'icon',
    'repository',
    'categories',
  ]) {
    assert.ok(MANIFEST[field], `manifest has no \`${field}\``);
  }
  assert.match(MANIFEST.version, /^\d+\.\d+\.\d+$/, 'not a marketplace version');
  assert.ok(MANIFEST.engines?.vscode, 'no supported VS Code range');
  assert.ok(MANIFEST.categories.length > 0, 'no categories');
});

test('every file the manifest promises is there', () => {
  for (const file of promised()) {
    assert.ok(fs.existsSync(at(file)), `the manifest points at a missing \`${file}\``);
    assert.ok(fs.statSync(at(file)).size > 0, `\`${file}\` is empty`);
  }
});

/// `.vscodeignore` is one careless line away from shipping a bundle whose
/// grammar, icon or entry point was left on the build machine.
test('nothing the manifest promises is excluded from the bundle', () => {
  const excluded = read('.vscodeignore')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '' && !line.startsWith('#'))
    .map((pattern) => pattern.split('/')[0]);
  for (const file of promised()) {
    const root = file.split('/')[0];
    assert.ok(
      !excluded.includes(root),
      `.vscodeignore excludes \`${root}\`, which \`${file}\` needs`,
    );
  }
});

/// A module the entry point requires is as load-bearing as the entry point
/// itself: left out of the VSIX, the extension throws on activation and
/// nothing in this workspace is ever generated again.
test('every local module the entry point requires ships with it', () => {
  const entry = MANIFEST.main.replace(/^\.\//, '');
  const excluded = read('.vscodeignore')
    .split('\n')
    .map((line) => line.trim().split('/')[0])
    .filter((pattern) => pattern !== '' && !pattern.startsWith('#'));
  const required = [...read(entry).matchAll(/require\('(\.[^']+)'\)/g)].map(([, name]) => name);
  assert.ok(required.length > 0, 'the entry point requires no local module');
  for (const relative of required) {
    const file = path.normalize(path.join(path.dirname(entry), relative));
    assert.ok(fs.existsSync(at(file)), `\`${entry}\` requires \`${file}\`, which is not there`);
    assert.ok(
      !excluded.includes(file.split('/')[0]),
      `.vscodeignore excludes \`${file}\`, which \`${entry}\` requires`,
    );
  }
});

/// The bundled binary is staged by `make vsix` rather than committed, so the
/// one thing that can go wrong is `.vscodeignore` learning to skip it — which
/// would ship a watcher with nothing to run.
test('the staged binary directory is not excluded', () => {
  const excluded = read('.vscodeignore').split('\n').map((line) => line.trim());
  assert.ok(!excluded.some((line) => line.startsWith('bin')), '.vscodeignore drops bin/');
});

test('the icon is a square PNG the marketplace will accept', () => {
  const icon = fs.readFileSync(at(MANIFEST.icon));
  assert.deepEqual(
    [...icon.subarray(0, 8)],
    [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a],
    'the icon is not a PNG',
  );
  // Width and height live in the IHDR chunk, at a fixed offset in every PNG.
  const width = icon.readUInt32BE(16);
  const height = icon.readUInt32BE(20);
  assert.equal(width, height, `the icon is ${width}x${height}, not square`);
  assert.ok(width >= 128, `the icon is ${width}px; the marketplace wants at least 128`);
});

/// A command in the palette that no one registered reports "command not found"
/// to whoever ran it, and nothing catches that before a user does.
test('every command the palette offers is registered', () => {
  const source = read(MANIFEST.main.replace(/^\.\//, ''));
  for (const { command } of MANIFEST.contributes.commands) {
    assert.ok(
      source.includes(`registerCommand('${command}'`),
      `\`${command}\` is in the palette but nothing registers it`,
    );
  }
});

/// The other direction: a command registered but never contributed is dead
/// weight nobody can reach.
test('every registered command is offered', () => {
  const source = read(MANIFEST.main.replace(/^\.\//, ''));
  const offered = new Set(MANIFEST.contributes.commands.map(({ command }) => command));
  for (const [, command] of source.matchAll(/registerCommand\('([^']+)'/g)) {
    assert.ok(offered.has(command), `\`${command}\` is registered but not in the palette`);
  }
});

/// A setting the manifest documents and the extension never reads is a promise
/// to the user that nothing keeps.
test('every setting the manifest documents is read', () => {
  const source = read(MANIFEST.main.replace(/^\.\//, ''));
  for (const key of Object.keys(MANIFEST.contributes.configuration.properties)) {
    const name = key.replace(/^dmx\./, '');
    assert.ok(
      source.includes(`'${name}'`),
      `\`${key}\` is documented but never read`,
    );
  }
});

/// dmx runs a binary over the folder that is open, and `dmx.path` can name one
/// inside it. Declaring that is what stops VS Code auto-starting it in a folder
/// the user has not trusted.
test('the extension declines to run in an untrusted folder', () => {
  assert.equal(MANIFEST.capabilities?.untrustedWorkspaces?.supported, false);
  assert.ok(MANIFEST.capabilities.untrustedWorkspaces.description, 'no reason given');
});

test('every grammar file declares the scope the manifest binds it to', () => {
  for (const { scopeName, path: file } of MANIFEST.contributes.grammars) {
    const grammar = JSON.parse(read(file.replace(/^\.\//, '')));
    assert.equal(grammar.scopeName, scopeName, `${file} declares a different scope`);
  }
});

test('every injection names what it injects into', () => {
  const injections = MANIFEST.contributes.grammars.filter((grammar) => grammar.injectTo);
  assert.ok(injections.length >= 2, 'the template and Dart injections should both be here');
  for (const { path: file, injectTo } of injections) {
    const { injectionSelector } = JSON.parse(read(file.replace(/^\.\//, '')));
    assert.ok(injectionSelector, `${file} has no injectionSelector`);
    for (const scope of injectTo) {
      assert.ok(
        injectionSelector.includes(scope),
        `${file} is injected into ${scope} but selects \`${injectionSelector}\``,
      );
    }
  }
});
