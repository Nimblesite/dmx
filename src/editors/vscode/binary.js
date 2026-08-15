'use strict';

// Which `dmx` the extension runs [editor.extension.binary].
//
// Split out from the extension itself so it can be tested over real
// directories: `extension.js` requires `vscode`, which exists only inside the
// editor host, and a resolution order nothing can exercise is a resolution
// order that drifts from the spec without anything noticing.

const path = require('node:path');

const BINARY = process.platform === 'win32' ? 'dmx.exe' : 'dmx';

/// Where cargo leaves a build, relative to the workspace root. The root is the
/// answer for a crate opened on its own; `src/dmx` is where this repository
/// keeps the dmx crate, so that working on dmx itself picks up the local build
/// with no configuration [editor.extension.binary].
const BUILD_ROOTS = ['.', path.join('src', 'dmx')];

/// The paths a `dmx` binary might be at, in the order they are preferred
/// [editor.extension.binary]: an explicit setting, the copy bundled in this
/// VSIX, a build inside the opened workspace, then — left to the caller — PATH.
///
/// `bundled` is passed in rather than read here because only the extension host
/// knows where its own VSIX was unpacked.
function binaryCandidates(root, bundled, override) {
  const found = [];
  if (override) {
    found.push(path.isAbsolute(override) ? override : path.join(root, override));
  }
  found.push(bundled);
  for (const build of BUILD_ROOTS) {
    found.push(path.join(root, build, 'target', 'release', BINARY));
    found.push(path.join(root, build, 'target', 'debug', BINARY));
  }
  return found;
}

module.exports = { BINARY, binaryCandidates };
