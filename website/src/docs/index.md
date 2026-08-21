---
layout: layouts/docs.njk
title: Getting started with dmx
description: Install dmx, generate your first Dart model, and learn how its macros work.
eleventyNavigation:
  key: Getting started
  order: 1
---

# Getting started with dmx

dmx writes ordinary Dart into the file you are already editing. Add an
`@dmx(...)` annotation to a class, save, and the generated members appear
inside that class, between two `//#region` markers. There is no `part` file, no
generated mixin, and no `build_runner` run.

This guide takes you from installing dmx to a generated immutable model.

## What is a macro?

A macro is a job dmx can do for you, and you pick one by name:
`@dmx('model')` generates the data-class members, `@dmx('enum')` generates
wire-safe enums, and so on.

Macros come from two places. The eleven **built-in** macros ship inside dmx and
are ready to use. A **[custom macro](/docs/dart-custom-macros/)** is a Dart
program in your own project, for generating something the built-ins do not
cover — reading a database schema, say, or an API document.

Not every model starts as Dart. When the types live in a design document rather
than in a class, write them once in a `.td` definition and dmx writes the Dart:
immutable classes that compare by value, with JSON beside them instead of
inside them — see **[Models from a diagram](/docs/models-from-a-diagram/)**.

You opt in with the package's single annotation type:

```dart
@dmx('model')
class User {
  // Your fields and constructor stay handwritten.
}
```

The annotation does not run anything at runtime. dmx reads your Dart file,
works out what to generate, checks that the result is valid Dart, and only then
writes it. All of that happens before you build, so your app ships plain Dart
with no reflection added to it.

dmx reads the annotated class, its members, and other declarations in the same
file. That is how `union` finds its variants and `restClient` finds the
interface it implements.

## Install

### VS Code extension (recommended)

The extension is the easiest way to use dmx. It bundles the CLI for your
platform, adds the editor commands and syntax highlighting, and automatically
watches a trusted Dart workspace. No Rust toolchain, global CLI, background
task, or terminal is required.

Install **dmx — Dart code generation** in any of these ways:

- In VS Code, open **Extensions** (`⇧⌘X` on macOS or `Ctrl+Shift+X` on
  Windows and Linux), search for **dmx**, and select **Install**.
- From a terminal, run `code --install-extension Nimblesite.dmx`.
- Select [Install the VS Code extension](vscode:extension/Nimblesite.dmx), which
  opens the extension in the editor you already have.
- Open the [dmx Marketplace page](https://marketplace.visualstudio.com/items?itemName=Nimblesite.dmx)
  and select **Install**.

Open or reload a trusted Dart workspace after installation. The extension runs
an initial build, inserts missing generated regions, and keeps them current as
you save.

Platform-specific extensions are published for macOS on ARM64, Linux on x64 and
ARM64, Alpine on x64 and ARM64, and Windows on x64. VS Code selects the correct
bundle automatically, and installs a build carrying no binary anywhere else —
that one runs `dmx` from `PATH`.

For an offline or air-gapped installation, download the VSIX for your platform
from the [latest GitHub release](https://github.com/Nimblesite/dmx/releases/latest).
In VS Code, open the Extensions panel, select the `…` menu, and choose **Install
from VSIX...**. You can also install the downloaded file from a terminal:

```bash
code --install-extension ./dmx-darwin-arm64.vsix
```

Platforms without a native bundle receive the universal VSIX. It provides the
editor features but requires a CLI-only installation on `PATH`, or an explicit
path in the `dmx.path` setting.

### CLI only (Homebrew or Scoop)

Use a package-manager install for terminal and CI workflows, or to supply the
binary used by the universal VSIX.

On macOS or Linux with Homebrew:

```bash
brew install nimblesite/tap/dmx
dmx --version
```

On Windows with Scoop:

```powershell
scoop bucket add nimblesite https://github.com/Nimblesite/scoop-bucket
scoop install dmx
dmx --version
```

These packages put `dmx` on `PATH`. They do not install the VS Code commands,
syntax highlighting, or automatic watcher.

### Build from source

A source build requires stable Rust and GNU Make:

```bash
make build
./src/dmx/target/release/dmx --version
```

Extension development additionally requires Node.js. Build and install a local
platform VSIX with:

```bash
make rebuild-install-vsix
```

## Add the dmx package

The extension and package-manager installs provide the generator. Your Dart
application also needs the small [`dmx` package](https://pub.dev/packages/dmx),
which provides the `@dmx(...)` annotation and the types the generated code
uses:

```bash
dart pub add dmx
```

Contributors working from a checkout of this repository can use a path
dependency instead, relative to the application's `pubspec.yaml`:

```yaml
dependencies:
  dmx:
    path: ../dmx/src/dart_packages/dmx
```

## Generate your first model

Create `lib/user.dart` in the Dart application:

```dart
import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({required this.id, this.email});

  final String id;
  final String? email;
}
```

If you installed the CLI separately, point it at the application's `lib`
directory:

```bash
dmx build path/to/app/lib --insert-regions
```

Use `--insert-regions` the first time you annotate a class: it adds the empty
`//#region` block that the generated members go into. After that, dmx replaces
only what is inside that block, and checks that everything outside it is
untouched before writing.

The `model` macro generates:

- `fromJson` and `toJson`;
- value equality and `hashCode`;
- `toString`;
- a type-safe `copyWith` that distinguishes an omitted value from explicit
  `null`.

Generated decoding returns `Result<T, DecodeError>` instead of throwing. A bad
value carries the exact failing path, including nested fields and collection
indices.

Commit the generated region with the rest of the Dart file. It is ordinary,
visible, reviewable source and does not require dmx at application runtime.

## Keep generated code current

Run the watcher after the first build:

```bash
dmx watch path/to/app/lib
```

The watcher regenerates changed Dart files without rewriting unchanged output.
It deliberately does not insert a missing region, so run one build with
`--insert-regions` whenever you add the first dmx annotation to a declaration.

The VS Code extension performs this watch workflow for you.

## Configure fields

An argument on the class configures the whole macro. An annotation on a field
configures just that field:

```dart
import 'package:dmx/dmx.dart';

@dmx('model', {'fieldRename': 'snake'})
class User {
  const User({required this.id, required this.displayName});

  @dmx('key', {'name': 'user_id'})
  final String id;

  final String displayName;
}
```

Here, `id` uses the JSON key `user_id`, while `displayName` follows the
class-wide snake-case setting and uses `display_name`. A field annotation only
configures the macro on the class; it never generates a region of its own.

## Compose macros

You can put more than one macro on the same class:

```dart
@dmx('model')
@dmx('diff')
class Order {
  const Order({required this.id, required this.total});

  final String id;
  final double total;
}
```

Both write into the same region, in the order you wrote the annotations. Here
`model` generates the data-class members and `diff` adds change tracking.

## Available macros

dmx generates these eleven macros today:

| Macro | Put it on | What you get |
| --- | --- | --- |
| `model` | class | JSON codecs, equality, `toString`, `copyWith` |
| `union` | sealed base class | tagged decoding, exhaustive matching, narrowing helpers |
| `enum` | enum | wire values, labels, parsing, JSON conversion |
| `diff` | class | field changes, changed names, difference checks |
| `lerp` | class | composable interpolation between values |
| `validate` | class | accumulated violations and typed validation result |
| `table` | class | SQL schema, statements, bindings, and row conversion |
| `route` | class | typed locations and URI parsing |
| `cli` | class | argument parsing, usage text, flags and options |
| `fake` | class or enum | deterministic fixtures and fake JSON |
| `restClient` | implementing class | typed methods over `DmxTransport` |

The repository also has designs and templates for `router`, `event`, `prefs`,
and `strings`, but dmx cannot generate them yet — annotating a class with one
of those names does nothing. The [macro catalogue](/docs/macros/) gives the
status of every macro.

## Useful commands

| Command | Purpose |
| --- | --- |
| `dmx build path/to/lib --insert-regions` | Generate and create missing regions |
| `dmx build path/to/lib` | Regenerate existing regions once |
| `dmx build path/to/lib --check` | Check whether committed generated code is current |
| `dmx watch path/to/lib` | Regenerate existing regions as files change |
| `make example` | Generate, analyze, and test the repository example |

If you built from source, use `./src/dmx/target/release/dmx` in place of `dmx`.

## Troubleshooting

### No region appears

Run `build` with `--insert-regions`. Watch mode only updates regions that
already exist.

### Nothing is generated for a macro name

Check it against the list above. Names such as `key`, `check.*`, `query`,
`flag`, and `opt` only configure a macro on the class — they generate nothing on
their own. The four catalogue designs do not generate anything yet either.

### dmx reports invalid Dart

Fix the syntax errors in your own code first. dmx parses your file before
generating and checks the finished file before writing, so it will never leave
you with Dart that does not parse.

### Generated code is out of date in CI

Run the build locally and commit the updated region. Add `dmx build
path/to/lib --check` to CI to catch it next time — it writes nothing and fails
if the committed code is stale.

## Next steps

- Try a generator in the [browser playground](/playground.html).
- Browse the [macro catalogue](/docs/macros/).
- Follow the [parse-to-emit pipeline](/docs/pipeline/).
