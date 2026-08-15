'use strict';

// Which binary the extension runs, over real directory trees
// [editor.extension.binary].
//
// The bug these hold shut: the crate moved to `src/dmx`, so a dmx checkout's
// own build stopped being found and working on dmx itself silently ran a
// different binary than the one just built.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { test } = require('node:test');
const { BINARY, binaryCandidates } = require('../binary.js');

/// A throwaway workspace with a `dmx` binary at each of `builds`.
function workspace(builds) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'dmx-binary-'));
  for (const relative of builds) {
    const directory = path.join(root, relative);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, BINARY), '', { mode: 0o755 });
  }
  return root;
}

const bundled = path.join('extension', 'bin', BINARY);

test('a dmx checkout is offered its own release build under src/dmx', () => {
  const root = workspace([path.join('src', 'dmx', 'target', 'release')]);

  assert.ok(
    binaryCandidates(root, bundled, '').includes(
      path.join(root, 'src', 'dmx', 'target', 'release', BINARY),
    ),
    'the crate lives at src/dmx, so that is where this repository builds its binary',
  );
});

test('a dmx checkout is offered its own debug build under src/dmx', () => {
  const root = workspace([path.join('src', 'dmx', 'target', 'debug')]);

  assert.ok(
    binaryCandidates(root, bundled, '').includes(
      path.join(root, 'src', 'dmx', 'target', 'debug', BINARY),
    ),
    'a debug build is what `cargo run` leaves behind while developing dmx',
  );
});

test('an explicit setting outranks every build and the bundled copy', () => {
  const root = workspace([path.join('target', 'release')]);

  assert.deepEqual(binaryCandidates(root, bundled, path.join('build', BINARY))[0], path.join(root, 'build', BINARY));
});

test('an absolute setting is taken as it is, not joined to the workspace', () => {
  const root = workspace([]);
  const absolute = path.join(os.tmpdir(), 'elsewhere', BINARY);

  assert.deepEqual(binaryCandidates(root, bundled, absolute)[0], absolute);
});

test('the bundled copy outranks any build in the workspace', () => {
  const root = workspace([path.join('target', 'release'), path.join('src', 'dmx', 'target', 'release')]);
  const order = binaryCandidates(root, bundled, '');

  assert.deepEqual(order[0], bundled, 'a consumer runs the binary their VSIX shipped');
});

test('a workspace-root build still outranks the crate subdirectory', () => {
  const root = workspace([path.join('target', 'release'), path.join('src', 'dmx', 'target', 'release')]);
  const order = binaryCandidates(root, bundled, '');

  assert.ok(
    order.indexOf(path.join(root, 'target', 'release', BINARY)) <
      order.indexOf(path.join(root, 'src', 'dmx', 'target', 'release', BINARY)),
    'the root build is the one a consumer means; src/dmx is dmx developing itself',
  );
});

test('release is preferred to debug at both locations', () => {
  const root = workspace([]);
  const order = binaryCandidates(root, bundled, '');

  assert.ok(order.indexOf(path.join(root, 'target', 'release', BINARY)) < order.indexOf(path.join(root, 'target', 'debug', BINARY)));
  assert.ok(
    order.indexOf(path.join(root, 'src', 'dmx', 'target', 'release', BINARY)) <
      order.indexOf(path.join(root, 'src', 'dmx', 'target', 'debug', BINARY)),
  );
});

test('no setting means no candidate joined from an empty string', () => {
  const root = workspace([]);

  assert.ok(!binaryCandidates(root, bundled, '').includes(root));
});
