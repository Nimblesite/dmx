# dmx for VS Code

Dart code generation that runs when you save. Annotate a class and `copyWith`,
equality, `hashCode`, `toString` and typed JSON appear **inside the same file**,
below a divider — no `part` files, no `.g.dart`, no mixins, no delegating
factories, and no `build_runner` run.

## Install

Install **dmx — Dart code generation** from the
[VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=nimblesite.dmx),
or run:

```bash
code --install-extension nimblesite.dmx
```

For an offline installation, download the VSIX for your platform from the
[latest GitHub release](https://github.com/Nimblesite/dmx/releases/latest),
then choose **Extensions: Install from VSIX...** or run `code
--install-extension path/to/dmx-<target>.vsix`.

The extension includes `dmx` on macOS and Linux x64/ARM64, Alpine x64/ARM64,
and Windows x64. On any other platform, install the CLI with Homebrew or Scoop
so the universal extension can find it on `PATH`:

```bash
# macOS or Linux
brew install nimblesite/tap/dmx
```

```powershell
# Windows
scoop bucket add nimblesite https://github.com/Nimblesite/scoop-bucket
scoop install dmx
```

Then add the package the generated code uses:

```bash
dart pub add dmx
```

This extension does three things.

## 1. It generates when you save

The `dmx` generator ships inside this extension, so there is nothing else to
install. Open a Dart workspace and it starts watching: save a file and the
generated members are rewritten; delete one by hand and it comes straight back.
No terminal to keep open and no build command to remember.

The status bar shows `$(eye) dmx` while it is watching — click it for the log.

| Command | What it does |
|---|---|
| **dmx: Build (insert regions)** | One-shot generation, adding a divider to newly annotated classes |
| **dmx: Restart Watcher** | After a crash, or a `dmx` you rebuilt yourself |
| **dmx: Stop Watcher** | Stop watching this window |
| **dmx: Show Output** | Everything the binary has said |

| Setting | Default | Meaning |
|---|---|---|
| `dmx.autoStart` | `true` | Start watching when a workspace folder opens |
| `dmx.paths` | `["lib"]` | Workspace-relative paths to generate and watch; missing paths are skipped |
| `dmx.path` | `""` | A `dmx` binary of your own, absolute or workspace-relative |
| `dmx.insertRegions` | `true` | Run `dmx build --insert-regions` once before watching, so a class annotated while the editor was closed still gets its divider |

The extension looks for `dmx` in this order: the `dmx.path` setting, the copy
bundled here, the workspace's own `target/release/dmx` and `target/debug/dmx`,
then `dmx` on `PATH`. On a platform with no bundled copy, the universal build
uses `dmx` from your `PATH`.

The extension stays off until you trust the folder, because it runs a program
over the folder you opened and `dmx.path` can point at one inside it.

## 2. It shows you what is generated

The `//#region` divider is highlighted as a divider rather than as one more
comment, so you can see where your code stops and the generated code starts. A
divider you label yourself, such as `//#region Helpers`, is left alone — dmx
never writes into one.

dmx annotations are highlighted in two tiers: the ones that generate code
(`@dmx('model')`, `@dmx('union')`, `@dmx('route')`, …) apart from the ones that
configure it (`@dmx('key')`, `@dmx('check.…')`, `@dmx('query')`, …). Dart's own
annotations are left to Dart.

This works with whichever Dart extension you already have.

## 3. It highlights dmx templates

A dmx template is a `.mustache` file: Dart with Mustache tags in it. Both are
highlighted, including tags inside Dart string literals, which is where
templates put them constantly.

The Dart highlighting comes from your Dart extension. Without it, the tags still
highlight and the Dart around them stays plain.

## What generation looks like

```dart
@dmx('model')
class User {
  const User({required this.id, this.email, this.tags = const []});

  final String id;
  final String? email;
  final List<String> tags;

  //#region
  // fromJson, toJson, ==, hashCode, toString, copyWith — regenerated on save
  //#endregion
}
```

Decoding never throws. `fromJson` returns a result that names the field that
failed — `Order.lines[2].product.price` — so a bad payload is a branch you
handle rather than an exception you hope somebody caught.

Before writing, dmx checks that everything outside the divider is unchanged, and
writes nothing at all when the generated code already matches.

Full documentation: [dmx.dev/docs](https://dmx.dev/docs/) ·
[github.com/Nimblesite/dmx](https://github.com/Nimblesite/dmx)
