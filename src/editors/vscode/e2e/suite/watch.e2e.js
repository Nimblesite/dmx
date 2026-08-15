'use strict';

// The packaged VSIX driven end to end [editor.extension.e2e]: the extension
// unpacked from the artifact, running the engine binary that artifact carries,
// inside a real VS Code, over a real Dart workspace.
//
// Nothing here reaches into internals. Every cause is a user action — opening
// the folder, typing, saving, running a palette command — and every effect is
// asserted where a user sees it: in the editor buffer and on disk.

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vscode = require('vscode');
const { annotatedClass } = require('../fixture.js');

const WORKSPACE = process.env.DMX_E2E_WORKSPACE ?? '';
const BINARY = process.platform === 'win32' ? 'dmx.exe' : 'dmx';
const EXTENSION_ID = 'nimblesite.dmx';
const MEMBERS = ['fromJson', 'toJson', 'operator ==', 'hashCode', 'toString', 'copyWith'];
const COMMANDS = ['dmx.build', 'dmx.restartWatcher', 'dmx.stopWatcher', 'dmx.showOutput'];

const at = (relative) => path.join(WORKSPACE, relative);
const read = (relative) => fs.readFileSync(at(relative), 'utf8');

/// Bounded wait on an asynchronous effect — the same shape, and for the same
/// reason, as the Rust watch suite's `wait_until_ready`: a watcher answers
/// when it answers, but the wait for it is bounded and named.
async function until(description, probe, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const value = probe();
    if (value) {
      return value;
    }
    assert.ok(Date.now() < deadline, `timed out waiting for ${description}`);
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
}

/// The generated region, or null while both dividers have not appeared.
function regionOf(source) {
  const start = source.indexOf('//#region');
  const end = source.indexOf('//#endregion');
  return start >= 0 && end > start ? source.slice(start, end) : null;
}

const generatedRegion = (relative) => regionOf(read(relative));

/// Every promise a `@dmx('model')` region makes, asserted in one sweep: the
/// dividers appear exactly once, all six members are present, every field is
/// carried by copyWith, equality, toJson, decode and toString, nothing listed
/// in `absent` survives, and the generated Dart obeys the same bans as
/// hand-written Dart — no `throw`, no ` as ` cast, no `.then(`.
function assertGeneratedModel(relative, className, fields, absent = []) {
  const source = read(relative);
  const generated = regionOf(source);
  assert.ok(generated, `${relative} has no generated region`);
  assert.equal(source.split('//#region').length, 2, `${relative}: not exactly one region start`);
  assert.equal(source.split('//#endregion').length, 2, `${relative}: not exactly one region end`);
  for (const member of MEMBERS) {
    assert.ok(generated.includes(member), `${relative}: the region lost \`${member}\``);
  }
  assert.ok(
    generated.includes(`fromJson`) && generated.includes(`Result<${className}, DecodeError>`),
    `${relative}: fromJson does not decode into Result<${className}, DecodeError>`,
  );
  assert.ok(generated.includes(`${className} copyWith`), `${relative}: copyWith lost its type`);
  for (const field of fields) {
    assert.ok(generated.includes(`${field} ?? this.${field}`), `${relative}: copyWith cannot replace \`${field}\``);
    assert.ok(generated.includes(`other.${field} == ${field}`), `${relative}: equality ignores \`${field}\``);
    assert.ok(generated.includes(`'${field}':`), `${relative}: toJson drops \`${field}\``);
    assert.ok(generated.includes(`'${field}': final`), `${relative}: fromJson never decodes \`${field}\``);
    assert.ok(generated.includes(`${field}: $${field}`), `${relative}: toString hides \`${field}\``);
  }
  for (const gone of absent) {
    assert.ok(!new RegExp(`\\b${gone}\\b`).test(generated), `${relative}: the region still carries removed \`${gone}\``);
  }
  assert.ok(!generated.includes('throw '), `${relative}: the generated region throws`);
  assert.ok(!generated.includes(' as '), `${relative}: the generated region casts`);
  assert.ok(!generated.includes('.then('), `${relative}: the generated region uses .then()`);
}

/// The user's own code, byte for byte, above the divider. Generation that eats
/// a constructor is worse than generation that never ran.
function assertUserCode(relative, markers) {
  const source = read(relative);
  const head = source.slice(0, source.indexOf('//#region'));
  for (const marker of markers) {
    assert.ok(head.includes(marker), `${relative}: generation lost the user's \`${marker}\``);
  }
}

async function openInEditor(relative) {
  const document = await vscode.workspace.openTextDocument(at(relative));
  const editor = await vscode.window.showTextDocument(document);
  return { document, editor };
}

/// One user edit: replace the single occurrence of `needle` in the buffer,
/// asserting the needle exists, the edit is applied, and the buffer is dirty —
/// three things a real keystroke also guarantees.
async function editOnce(editor, needle, replacement) {
  const text = editor.document.getText();
  const found = text.indexOf(needle);
  assert.ok(found >= 0, `the editor buffer has no \`${needle}\``);
  const range = new vscode.Range(
    editor.document.positionAt(found),
    editor.document.positionAt(found + needle.length),
  );
  const applied = await editor.edit((builder) => builder.replace(range, replacement));
  assert.ok(applied, `the edit replacing \`${needle}\` was refused`);
  assert.ok(editor.document.isDirty, `replacing \`${needle}\` left the buffer clean`);
}

/// The editor noticing the watcher's write on its own is part of what is being
/// proved — and it is also the synchronisation point: editing the stale buffer
/// before it reverts makes VS Code refuse the next save as a conflict.
async function awaitEditorCatchUp(document, relative) {
  await until(`the editor buffer to catch up with ${relative}`, () =>
    document.getText() === read(relative),
  );
}

/// Save the buffer, wait for the watcher's regeneration to land on disk, and
/// wait for the editor to pick the regenerated bytes up without any command.
async function saveAndAwaitRegeneration(document, relative, marker) {
  assert.ok(await document.save(), `${relative} did not save`);
  assert.ok(!document.isDirty, `${relative} is still dirty after saving`);
  await until(`${relative} to regenerate with \`${marker}\``, () => {
    const generated = generatedRegion(relative);
    return generated !== null && generated.includes(marker);
  });
  await awaitEditorCatchUp(document, relative);
}

/// Add `final String name;`-style field plus its `required this.` parameter —
/// the two edits a user makes to grow a model by one field.
async function addField(editor, lastParam, type, name) {
  await editOnce(editor, `required this.${lastParam}}`, `required this.${lastParam}, required this.${name}}`);
  const anchor = new RegExp(`  final [A-Za-z<, >?]+ ${lastParam};`).exec(editor.document.getText());
  assert.ok(anchor, `the buffer has no field declaration for \`${lastParam}\``);
  await editOnce(editor, anchor[0], `${anchor[0]}\n  final ${type} ${name};`);
}

describe('the packaged VSIX, running the engine it carries', () => {
  it('generates every annotated class on folder open, with no command at all', async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `${EXTENSION_ID} is not present in the test host`);
    await extension.activate();
    assert.ok(extension.isActive, 'the extension did not activate on a pubspec workspace');

    const commands = await vscode.commands.getCommands(true);
    for (const command of COMMANDS) {
      assert.ok(commands.includes(command), `the palette is missing \`${command}\``);
    }

    // Auto-start [editor.extension.autostart]: both files arrived with NO
    // divider; the extension's own `build --insert-regions` pass adds it and
    // the first generation fills it. Nobody ran anything.
    for (const relative of ['lib/profile.dart', 'lib/settings.dart']) {
      await until(`${relative} to gain its generated region`, () => {
        const generated = generatedRegion(relative);
        return generated !== null && generated.includes('copyWith');
      });
    }
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle']);
    assertGeneratedModel('lib/settings.dart', 'Settings', ['theme']);
    assertUserCode('lib/profile.dart', [
      "import 'package:dmx/dmx.dart';",
      "@dmx('model')",
      'const Profile({required this.handle});',
      'final String handle;',
    ]);
    assertUserCode('lib/settings.dart', ["@dmx('model')", 'final String theme;']);

    // The launcher stripped the execute bit before boot; generation having run
    // proves the extension restored it, and this proves it directly
    // [editor.extension.binary].
    if (process.platform !== 'win32') {
      const bundled = path.join(extension.extensionPath, 'bin', BINARY);
      assert.ok(fs.existsSync(bundled), 'the unpacked bundle carries no binary');
      fs.accessSync(bundled, fs.constants.X_OK);
    }
  });

  it('regenerates on every save as the model is grown, renamed, and shrunk', async () => {
    const { document, editor } = await openInEditor('lib/profile.dart');

    // Grow: one new field, typed where a user types it.
    await addField(editor, 'handle', 'int', 'age');
    await saveAndAwaitRegeneration(document, 'lib/profile.dart', 'age ?? this.age');
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle', 'age']);
    assertUserCode('lib/profile.dart', ['final int age;', 'required this.age']);

    // The buffer caught up with the watcher's write without any command.
    assert.ok(document.getText().includes('age ?? this.age'), 'the buffer missed the regeneration');
    assert.equal(document.getText(), read('lib/profile.dart'), 'buffer and disk disagree');

    // Rename: the old name must vanish from every generated member.
    await editOnce(editor, 'required this.age}', 'required this.years}');
    await editOnce(editor, 'final int age;', 'final int years;');
    await saveAndAwaitRegeneration(document, 'lib/profile.dart', 'years ?? this.years');
    await until('the rename to purge `age` from the region', () => {
      const generated = generatedRegion('lib/profile.dart');
      return generated !== null && !/\bage\b/.test(generated);
    });
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle', 'years'], ['age']);

    // Shrink: delete the field; the deleted member must not linger.
    await editOnce(editor, ', required this.years}', '}');
    await editOnce(editor, '\n  final int years;', '');
    assert.ok(await document.save(), 'lib/profile.dart did not save');
    await until('the deletion to purge `years` from the region', () => {
      const generated = generatedRegion('lib/profile.dart');
      return generated !== null && !generated.includes('years');
    });
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle'], ['years', 'age']);
    await awaitEditorCatchUp(document, 'lib/profile.dart');
  });

  it('refuses a broken file alone: bytes intact, neighbours generated, fix resumed', async () => {
    const intact = generatedRegion('lib/settings.dart');
    assert.ok(intact !== null && intact.includes('copyWith'), 'lib/settings.dart lost its region');

    // Break settings.dart the way a half-typed edit breaks it [engine.api]:
    // a rescan MUST NOT fail because one file did.
    const settings = await openInEditor('lib/settings.dart');
    await editOnce(settings.editor, 'final String theme;', 'final String theme;\n  final int ===;');
    assert.ok(await settings.document.save(), 'the broken buffer did not save');

    // The healthy neighbour still regenerates, so the watcher survived.
    const profile = await openInEditor('lib/profile.dart');
    await addField(profile.editor, 'handle', 'bool', 'flag');
    await saveAndAwaitRegeneration(profile.document, 'lib/profile.dart', 'flag ?? this.flag');
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle', 'flag']);

    // The broken file was refused, not destroyed: the user's garbage is still
    // there and the stale region was left byte-identical.
    const broken = read('lib/settings.dart');
    assert.ok(broken.includes('final int ===;'), "the refused file lost the user's edit");
    assert.equal(regionOf(broken), intact, 'a refused file had its generated region rewritten');

    // Fixing the file brings it straight back into generation. Reopen it the
    // way a user tabs back: showing profile.dart closed the settings editor.
    const fixed = await openInEditor('lib/settings.dart');
    await editOnce(fixed.editor, 'final int ===;', 'final int retries;');
    await editOnce(fixed.editor, 'required this.theme}', 'required this.theme, required this.retries}');
    await saveAndAwaitRegeneration(fixed.document, 'lib/settings.dart', 'retries ?? this.retries');
    assertGeneratedModel('lib/settings.dart', 'Settings', ['theme', 'retries']);
    assertUserCode('lib/settings.dart', ['final int retries;', 'required this.retries']);
  });

  it('stop, build, and restart from the palette all drive the real engine', async () => {
    await vscode.commands.executeCommand('dmx.stopWatcher');

    // While nothing watches: a class annotated in a brand-new file, and an
    // edit saved in an existing one. Neither can generate until asked.
    await vscode.workspace.fs.writeFile(
      vscode.Uri.file(at('lib/badge.dart')),
      Buffer.from(annotatedClass('Badge', 'label')),
    );
    const profile = await openInEditor('lib/profile.dart');
    await addField(profile.editor, 'flag', 'String', 'note');
    assert.ok(await profile.document.save(), 'lib/profile.dart did not save');

    // The palette build inserts the new region and catches up on the edit.
    await vscode.commands.executeCommand('dmx.build');
    await until('dmx.build to give lib/badge.dart its region', () => {
      const generated = generatedRegion('lib/badge.dart');
      return generated !== null && generated.includes('copyWith');
    });
    await until('dmx.build to regenerate the stopped-time edit', () => {
      const generated = generatedRegion('lib/profile.dart');
      return generated !== null && generated.includes('note ?? this.note');
    });
    assertGeneratedModel('lib/badge.dart', 'Badge', ['label']);
    assertGeneratedModel('lib/profile.dart', 'Profile', ['handle', 'flag', 'note']);

    // Restart: its own insert-regions pass picks up a file created before it,
    // and the watcher it starts is really watching — a plain save regenerates
    // with no further command.
    await vscode.workspace.fs.writeFile(
      vscode.Uri.file(at('lib/chip.dart')),
      Buffer.from(annotatedClass('Chip', 'name')),
    );
    await vscode.commands.executeCommand('dmx.restartWatcher');
    await until('the restarted build to give lib/chip.dart its region', () => {
      const generated = generatedRegion('lib/chip.dart');
      return generated !== null && generated.includes('copyWith');
    });
    assertGeneratedModel('lib/chip.dart', 'Chip', ['name']);

    const chip = await openInEditor('lib/chip.dart');
    await addField(chip.editor, 'name', 'int', 'level');
    await saveAndAwaitRegeneration(chip.document, 'lib/chip.dart', 'level ?? this.level');
    assertGeneratedModel('lib/chip.dart', 'Chip', ['name', 'level']);
    assertUserCode('lib/chip.dart', ['final int level;']);

    await vscode.commands.executeCommand('dmx.showOutput');
  });
});
