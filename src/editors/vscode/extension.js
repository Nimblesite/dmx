'use strict';

// The watcher, started for you [editor.extension.autostart].
//
// A correct `dmx watch` that nobody launches is indistinguishable, from the
// editor, from a broken one: you delete a generated member and it stays
// deleted. Opening a Dart workspace is the signal that someone is editing, so
// that is when the binary shipped inside this extension starts watching
// [editor.extension.binary].

const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const vscode = require('vscode');
const { watchTargets } = require('./paths.js');
const { BINARY, binaryCandidates } = require('./binary.js');

/** @type {vscode.OutputChannel} */
let channel;
/** @type {vscode.StatusBarItem} */
let status;
/** @type {Map<string, Watcher>} */
const watchers = new Map();

function log(message) {
  channel.appendLine(message);
}

function readSettings(folder) {
  const settings = vscode.workspace.getConfiguration('dmx', folder.uri);
  const inspected = settings.inspect('paths');
  return {
    autoStart: settings.get('autoStart', true),
    paths: settings.get('paths', ['lib']),
    // Whether somebody set `dmx.paths` themselves. Naming paths means naming
    // all of them; leaving it alone means "the packages in this folder"
    // [editor.extension.paths].
    explicitPaths:
      (inspected?.workspaceFolderValue ??
        inspected?.workspaceValue ??
        inspected?.globalValue) !== undefined,
    insertRegions: settings.get('insertRegions', true),
    binary: (settings.get('path', '') || '').trim(),
  };
}

/// Paths that are actually on disk, plus the packages a multi-package workspace
/// keeps below its root [editor.extension.paths]. `dmx watch` refuses a path it
/// cannot canonicalize, and a Flutter package with no `lib` yet is not an error
/// worth a popup — it is a folder nobody has written Dart in.
function existingPaths(folder, settings) {
  return watchTargets(folder.uri.fsPath, settings.paths, settings.explicitPaths);
}

function resolveBinary(context, folder, override) {
  const bundled = context.asAbsolutePath(path.join('bin', BINARY));
  const found = binaryCandidates(folder.uri.fsPath, bundled, override).find(isExecutableFile);
  if (found) {
    return found;
  }
  // The universal bundle carries no binary on purpose: it is what the
  // marketplace serves to a platform with no build of its own
  // [editor.extension.binary]. Reaching PATH is that bundle working as
  // intended, so this says where to get one rather than reporting a fault.
  log(
    `no bundled or built binary; using \`${BINARY}\` from PATH. ` +
      'Install one from https://github.com/Nimblesite/dmx/releases ' +
      'or point `dmx.path` at your own build.',
  );
  return BINARY;
}

function isExecutableFile(file) {
  try {
    if (!fs.statSync(file).isFile()) {
      return false;
    }
  } catch {
    return false;
  }
  return makeExecutable(file);
}

/// A VSIX is a zip, and the execute bit does not survive every route from a
/// build machine to an extensions folder. The binary is ours, so fixing it is
/// ours too.
function makeExecutable(file) {
  if (process.platform === 'win32') {
    return true;
  }
  try {
    fs.accessSync(file, fs.constants.X_OK);
    return true;
  } catch {
    try {
      fs.chmodSync(file, 0o755);
      return true;
    } catch (error) {
      log(`cannot make ${file} executable: ${error.message}`);
      return false;
    }
  }
}

/// Whole lines only: a chunk boundary in the middle of a diagnostic would
/// otherwise split one message across two log entries.
///
/// Nothing here touches an open buffer. Putting a document back in step with
/// the file behind it is VS Code's own job [editor.extension.refresh]: reading
/// the file ourselves and applying it as a workspace edit leaves VS Code
/// holding the version stamp from before dmx's atomic rename, and the save that
/// follows is refused as `File Modified Since` — which is the stale-buffer
/// conflict it was meant to prevent, now provoked deliberately.
function pipe(stream) {
  let pending = '';
  stream.setEncoding('utf8');
  stream.on('data', (chunk) => {
    const lines = (pending + chunk).split('\n');
    pending = lines.pop() ?? '';
    for (const line of lines) {
      log(line);
    }
  });
  stream.on('end', () => {
    if (pending.length > 0) {
      log(pending);
    }
  });
}

/// Runs `dmx` in `folder`, with everything it says going to the one channel a
/// reader can watch. Failure to spawn at all is reported through the same
/// channel by the caller that cares.
function spawnDmx(folder, binary, args) {
  log(`${folder.name}: ${binary} ${args.join(' ')}`);
  const child = spawn(binary, args, { cwd: folder.uri.fsPath });
  pipe(child.stdout);
  pipe(child.stderr);
  return child;
}

class Watcher {
  constructor(context, folder) {
    this.context = context;
    this.folder = folder;
    this.child = null;
    this.stopping = false;
    this.failed = false;
  }

  get running() {
    return this.child !== null;
  }

  start() {
    this.stop();
    this.stopping = false;
    this.failed = false;
    const settings = readSettings(this.folder);
    const targets = existingPaths(this.folder, settings);
    if (targets.length === 0) {
      log(`${this.folder.name}: no Dart package under it and no [${settings.paths.join(', ')}];`
        + ' nothing to watch');
      return;
    }
    const binary = resolveBinary(this.context, this.folder, settings.binary);
    if (!settings.insertRegions) {
      this.watch(binary, targets);
      return;
    }
    // `watch` deliberately refuses `--insert-regions`, so a class annotated
    // while the editor was closed gets its divider here — the same order
    // `make dev` uses, and for the same reason. A build that fails does not
    // stop the watcher: the file it choked on is the one being fixed.
    this.run(binary, ['build', ...targets, '--insert-regions']).on('close', () => {
      if (!this.stopping) {
        this.watch(binary, targets);
      }
    });
  }

  watch(binary, targets) {
    const child = this.run(binary, ['watch', ...targets]);
    this.child = child;
    child.on('exit', (code, signal) => {
      this.child = null;
      if (this.stopping) {
        return;
      }
      this.failed = true;
      log(`watcher exited (${signal ?? code}) — run "dmx: Restart Watcher" to bring it back`);
      refreshStatus();
    });
    refreshStatus();
  }

  run(binary, args) {
    const child = spawnDmx(this.folder, binary, args);
    child.on('error', (error) => {
      this.failed = true;
      log(`cannot run ${binary}: ${error.message}`);
      refreshStatus();
    });
    return child;
  }

  stop() {
    this.stopping = true;
    if (this.child !== null) {
      this.child.kill();
      this.child = null;
    }
  }
}

function refreshStatus() {
  const live = [...watchers.values()].filter((watcher) => watcher.running).length;
  const broken = [...watchers.values()].filter((watcher) => watcher.failed).length;
  if (broken > 0) {
    status.text = '$(error) dmx';
    status.tooltip = 'dmx is not watching — click for the log';
  } else if (live > 0) {
    status.text = '$(eye) dmx';
    status.tooltip = `dmx is watching ${live} folder(s) — click for the log`;
  } else {
    status.hide();
    return;
  }
  status.show();
}

function startAll(context, forced) {
  stopAll();
  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    if (!forced && !readSettings(folder).autoStart) {
      continue;
    }
    const watcher = new Watcher(context, folder);
    watchers.set(folder.uri.toString(), watcher);
    watcher.start();
  }
  refreshStatus();
}

function stopAll() {
  for (const watcher of watchers.values()) {
    watcher.stop();
  }
  watchers.clear();
  refreshStatus();
}

function buildAll(context) {
  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    const settings = readSettings(folder);
    const targets = existingPaths(folder, settings);
    if (targets.length === 0) {
      continue;
    }
    const binary = resolveBinary(context, folder, settings.binary);
    const child = spawnDmx(folder, binary, ['build', ...targets, '--insert-regions']);
    child.on('error', (error) => log(`cannot run ${binary}: ${error.message}`));
  }
  channel.show(true);
}

function activate(context) {
  channel = vscode.window.createOutputChannel('dmx');
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  status.command = 'dmx.showOutput';
  context.subscriptions.push(
    channel,
    status,
    vscode.commands.registerCommand('dmx.showOutput', () => channel.show(true)),
    vscode.commands.registerCommand('dmx.build', () => buildAll(context)),
    vscode.commands.registerCommand('dmx.restartWatcher', () => startAll(context, true)),
    vscode.commands.registerCommand('dmx.stopWatcher', () => stopAll()),
    vscode.workspace.onDidChangeWorkspaceFolders(() => startAll(context, false)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('dmx')) {
        startAll(context, false);
      }
    }),
  );
  startAll(context, false);
}

function deactivate() {
  stopAll();
}

module.exports = { activate, deactivate };
