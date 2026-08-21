'use strict';

// Launches the real editor over the real artifact [editor.extension.e2e].
//
// `make vsix-e2e` rebuilds the engine binary, packages the VSIX, and hands the
// artifact's path over in DMX_VSIX. This launcher unpacks that artifact — the
// bytes a user installs, `.vscodeignore` filtering included — strips the
// execute bit the way a zip route does [editor.extension.binary], writes a
// Dart fixture workspace, and boots VS Code on it with the unpacked bundle as
// the extension under test. The suite itself lives in `suite/`.

const AdmZip = require('adm-zip');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { runTests } = require('@vscode/test-electron');
const { annotatedClass, definition, document, template } = require('./fixture.js');

const BINARY = process.platform === 'win32' ? 'dmx.exe' : 'dmx';

/// What the bundle cannot work without. `.vscodeignore` is one careless line
/// away from shipping a VSIX whose binary stayed on the build machine, and
/// that failure belongs here, not in a user's editor.
function missingFromBundle(extension) {
  return ['package.json', 'extension.js', path.join('bin', BINARY)].filter(
    (file) => !fs.existsSync(path.join(extension, file)),
  );
}

function writeWorkspace(workspace) {
  fs.mkdirSync(path.join(workspace, 'lib'), { recursive: true });
  fs.writeFileSync(
    path.join(workspace, 'pubspec.yaml'),
    'name: dmx_vsix_e2e\nenvironment:\n  sdk: ^3.0.0\n',
  );
  fs.writeFileSync(path.join(workspace, 'lib', 'profile.dart'), annotatedClass('Profile', 'handle'));
  fs.writeFileSync(path.join(workspace, 'lib', 'settings.dart'), annotatedClass('Settings', 'theme'));
  // A document with no Dart behind it [typediagram.documents]: the extension
  // has to find it, watch it, and generate the file it names.
  fs.mkdirSync(path.join(workspace, 'docs'), { recursive: true });
  fs.writeFileSync(path.join(workspace, 'docs', 'shipping.dmx.md'), document('Parcel', 'tracking'));
  // A standalone definition with the template beside it
  // [typediagram.standalone]: the extension has to find the `.td`, watch it,
  // and answer an edit to either file.
  fs.mkdirSync(path.join(workspace, 'models'), { recursive: true });
  fs.writeFileSync(path.join(workspace, 'models', 'crate.td'), definition('Crate', 'code'));
  fs.writeFileSync(path.join(workspace, 'models', 'crate.mustache'), template());
}

async function main() {
  // Running from a terminal inside VS Code inherits the extension host's
  // ELECTRON_RUN_AS_NODE, which turns the editor under test into a bare Node
  // that treats the workspace path as a script. Scrub it so the suite behaves
  // the same from any shell.
  delete process.env.ELECTRON_RUN_AS_NODE;
  const vsix = process.env.DMX_VSIX ?? '';
  if (vsix === '' || !fs.existsSync(vsix)) {
    console.error('DMX_VSIX does not name a packaged VSIX — run `make vsix-e2e`.');
    process.exitCode = 1;
    return;
  }
  // realpath, not the symlinked /var/folders macOS hands out: the editor's
  // file watcher must see the engine's writes, and watching through a symlink
  // is exactly the kind of flake an e2e suite exists to not have.
  const staging = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'dmx-vsix-e2e-'));
  // macOS caps a Unix socket path at 104 bytes (`sun_path`), and VS Code opens
  // its IPC socket INSIDE the user-data directory. The default lands under the
  // extension itself — `src/editors/vscode/.vscode-test/user-data` — which is
  // 107 bytes on a normal home directory and dies with `EINVAL` before the
  // editor starts. A short directory beside the staging one is 78, so the gate
  // does not depend on how deep the checkout happens to sit.
  const userData = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'dmx-ud-'));
  new AdmZip(path.resolve(vsix)).extractAllTo(staging, true);
  const extension = path.join(staging, 'extension');
  const missing = missingFromBundle(extension);
  if (missing.length > 0) {
    console.error(`the VSIX shipped without ${missing.join(', ')} — .vscodeignore ate the bundle`);
    process.exitCode = 1;
    return;
  }
  // A zip forgets the execute bit. Generation running at all proves the
  // extension restored it [editor.extension.binary].
  if (process.platform !== 'win32') {
    fs.chmodSync(path.join(extension, 'bin', BINARY), 0o644);
  }
  const workspace = path.join(staging, 'fixture');
  writeWorkspace(workspace);
  try {
    await runTests({
      extensionDevelopmentPath: extension,
      extensionTestsPath: path.join(__dirname, 'suite', 'index.js'),
      launchArgs: [
        workspace,
        '--user-data-dir',
        userData,
        '--disable-extensions',
        '--disable-workspace-trust',
        '--skip-welcome',
        '--skip-release-notes',
        '--disable-gpu',
      ],
      extensionTestsEnv: { DMX_E2E_WORKSPACE: workspace },
    });
  } catch {
    console.error('the VSIX e2e suite failed');
    process.exitCode = 1;
  } finally {
    fs.rmSync(staging, { recursive: true, force: true });
    fs.rmSync(userData, { recursive: true, force: true });
  }
}

main();
