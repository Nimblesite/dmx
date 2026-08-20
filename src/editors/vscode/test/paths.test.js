'use strict';

// What the extension watches, over real directory trees [editor.extension.paths].
//
// The bug these hold shut: a repo whose packages live under `examples/` has no
// `lib` at its root, the watcher was handed nothing, and every generated file
// in the workspace silently stopped being regenerated.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { test } = require('node:test');
const { documents, packageLibraries, watchTargets } = require('../paths.js');

/// A throwaway workspace holding `directories`, each made a package when its
/// entry says so.
function workspace(layout) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'dmx-paths-'));
  for (const [relative, isPackage] of Object.entries(layout)) {
    fs.mkdirSync(path.join(root, relative), { recursive: true });
    if (isPackage) {
      fs.mkdirSync(path.join(root, relative, 'lib'), { recursive: true });
      fs.writeFileSync(path.join(root, relative, 'pubspec.yaml'), 'name: fixture\n');
    }
  }
  return root;
}

test('a repo whose packages live under examples/ is watched at every one', () => {
  const root = workspace({
    'examples/storefront': true,
    'examples/dmx_sqlite_example': true,
    'src/dart_packages/dmx': true,
    src: false,
    docs: false,
  });

  assert.deepEqual(watchTargets(root, ['lib'], false), [
    path.join('examples', 'dmx_sqlite_example', 'lib'),
    path.join('examples', 'storefront', 'lib'),
    path.join('src', 'dart_packages', 'dmx', 'lib'),
  ]);
});

test('one package opened on its own is still watched at lib', () => {
  const root = workspace({ '.': true });

  assert.deepEqual(watchTargets(root, ['lib'], false), ['lib']);
});

test('a package and the packages beside it are watched together', () => {
  const root = workspace({ '.': true, 'packages/api': true });

  assert.deepEqual(watchTargets(root, ['lib'], false), ['lib', path.join('packages', 'api', 'lib')]);
});

test('naming paths yourself means naming all of them', () => {
  const root = workspace({ '.': true, 'packages/api': true });
  fs.mkdirSync(path.join(root, 'lib', 'models'), { recursive: true });

  assert.deepEqual(watchTargets(root, [path.join('lib', 'models')], true), [
    path.join('lib', 'models'),
  ]);
});

test('a path that was set but is not there is not handed to dmx', () => {
  const root = workspace({ src: false });

  assert.deepEqual(watchTargets(root, ['lib'], true), []);
});

test('build output and dependencies are never searched', () => {
  const root = workspace({
    'node_modules/some_package': true,
    'build/generated_package': true,
    '.dart_tool/cached_package': true,
  });

  assert.deepEqual(packageLibraries(root), []);
});

test('a package inside a package is left to the package that holds it', () => {
  const root = workspace({ 'packages/api': true, 'packages/api/example': true });

  assert.deepEqual(packageLibraries(root), [path.join('packages', 'api', 'lib')]);
});

test('a pubspec without lib is not a package to generate into', () => {
  const root = workspace({ tool: false });
  fs.writeFileSync(path.join(root, 'tool', 'pubspec.yaml'), 'name: tool\n');

  assert.deepEqual(packageLibraries(root), []);
});

/// A workspace holding `files`, each written with placeholder content.
function withFiles(layout, files) {
  const root = workspace(layout);
  for (const relative of files) {
    const target = path.join(root, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, '# a document\n');
  }
  return root;
}

test('every *.dmx.md document is watched, wherever it lives', () => {
  const root = withFiles({ 'packages/store': true, docs: false }, [
    'models.dmx.md',
    'docs/shipping.dmx.md',
    'packages/store/docs/store.dmx.md',
    'docs/README.md',
    'packages/store/lib/notes.md',
  ]);

  assert.deepEqual(documents(root), [
    path.join('docs', 'shipping.dmx.md'),
    'models.dmx.md',
    path.join('packages', 'store', 'docs', 'store.dmx.md'),
  ]);

  const targets = watchTargets(root, ['lib'], false);
  assert.ok(targets.includes(path.join('packages', 'store', 'lib')), targets.join(', '));
  assert.ok(targets.includes(path.join('docs', 'shipping.dmx.md')), targets.join(', '));
  assert.ok(!targets.includes(path.join('docs', 'README.md')), targets.join(', '));
});

test('build output and hidden directories hold no documents worth watching', () => {
  const root = withFiles({ build: false, node_modules: false, '.git': false }, [
    'build/generated.dmx.md',
    'node_modules/pkg/thing.dmx.md',
    '.git/stash.dmx.md',
    'kept.dmx.md',
  ]);
  assert.deepEqual(documents(root), ['kept.dmx.md']);
});

test('explicit paths are honoured exactly, documents included or not', () => {
  const root = withFiles({ docs: false }, ['docs/shipping.dmx.md']);
  assert.deepEqual(watchTargets(root, [path.join('docs', 'shipping.dmx.md')], true), [
    path.join('docs', 'shipping.dmx.md'),
  ]);
  assert.deepEqual(watchTargets(root, ['lib'], true), []);
});
