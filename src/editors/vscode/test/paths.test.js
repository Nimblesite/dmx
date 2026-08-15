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
const { packageLibraries, watchTargets } = require('../paths.js');

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
