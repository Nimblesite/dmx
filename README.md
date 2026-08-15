# dmx

Dart code generation that runs when you save. Annotate a class, and `copyWith`,
equality, `hashCode`, `toString` and typed JSON appear **inside the same file**,
below a divider — no `part` files, no `.g.dart`, no mixins, no delegating
factories, and no `build_runner` run.

## Try it without installing anything

**[Run the real generator in your browser →](https://dmx.dev/playground.html)**

The playground compiles this repository's Rust generator to WebAssembly, so it
builds the same context, renders the same Mustache template, validates the same
way, and splices the same region as the CLI. Edit the Dart *and* the template.
Neither input leaves the tab.

## Install

**VS Code** — install **dmx — Dart code generation** from the
[Marketplace](https://marketplace.visualstudio.com/items?itemName=Nimblesite.dmx),
or from a terminal:

```bash
code --install-extension nimblesite.dmx
```

The extension bundles the `dmx` binary and starts watching when you open a
trusted Dart workspace. There is no Rust, no Cargo, and no command to run.

**Any other editor** — install the CLI and leave the watcher running:

```bash
brew install nimblesite/tap/dmx   # or: scoop install dmx
dmx watch lib
```

## Getting started

Add the runtime the generated code composes with:

```bash
dart pub add dmx
```

Annotate a class and save the file:

```dart
import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({required this.id, this.email, this.tags = const []});

  final String id;
  final String? email;
  final List<String> tags;
}
```

The members appear below the divider, in the file you were already looking at:

```dart
  //#region
  static Result<User, DecodeError> fromJson(Object? json, [String path = 'User']) =>
      switch (json) {
        {
          'id': final String id,
          'tags': final List<dynamic> tags,
        } =>
          switch ((
            dmxNullable<String>(dmxKey(json, 'email'), '$path.email', dmxString),
            dmxList<String>(tags, '$path.tags', dmxString),
          )) {
            (Ok(value: final email), Ok(value: final tags)) =>
              Ok(User(id: id, email: email, tags: tags)),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'User', json)),
      };
  //#endregion
```

Plus `toJson`, `==`, `hashCode`, `toString` and `copyWith`. Decoding returns a
sealed `Result`, so a bad payload is a branch that names the field that failed
— `Order.lines[2].product.price` — rather than a thrown type error.

The [getting started guide](https://dmx.dev/docs/) covers the rest.

## What dmx does

**Eleven built-in macros.** Each one is an annotation name, a Rust context
builder, and a Mustache template.

| | | |
|---|---|---|
| `@dmx('model')` immutable data class | `@dmx('union')` sealed sum type | `@dmx('enum')` wire-safe enums |
| `@dmx('diff')` what changed, as data | `@dmx('lerp')` interpolation | `@dmx('validate')` accumulating constraints |
| `@dmx('table')` SQL schema and rows | `@dmx('route')` typed deep links | `@dmx('cli')` argv parser and usage |
| `@dmx('fake')` deterministic fixtures | `@dmx('restClient')` HTTP implementations | |

**Templates you own.** [`src/dmx/templates/model.mustache`](src/dmx/templates/model.mustache)
decides what the output looks like, top to bottom. Every expression reaches the
template already worked out (`resultExpr`, `equalsExpr`, `copyArg`, …), so
changing the layout never means working out Dart types yourself.

**Macros written in Dart.** A macro receives a typed view of the declaration —
name, fields, types, annotations — and returns the Dart to emit, or hands its
model to a Mustache template and lets dmx render it with the same engine the
built-ins use. Two worked examples do exactly that: one reads a live
[SQLite database](examples/dmx_sqlite_example/README.md), one reads an
[OpenAPI document](examples/dmx_openapi_example/README.md).

**It never writes broken Dart.**

| Guarantee | Mechanism |
|---|---|
| Never emits unparseable Dart | The whole candidate file is re-parsed before writing [validation] |
| Never touches your code | Bytes outside the region are diffed pre-write [emission.inline-backend.byte-exactness] |
| Leaves labelled folds alone | Only the bare, unlabelled `//#region` block is machine-owned |
| Repairs a region you gutted | dmx empties the region, re-parses, and regenerates [emission.inline-backend.region-recovery] |
| Zero writes when nothing changed | Byte-compare before write [emission.inline-backend.no-op-writes] |
| Generated code obeys the house rules | No `throw`, `as`, `!` or `_$` names — asserted over the whole golden corpus |

## CLI

```
dmx build [PATHS...] [--insert-regions] [--check]
dmx watch [PATHS...]
```

Both default to `lib`. `watch` regenerates changed `.dart` files and debounces
save bursts. `--check` writes nothing and exits 2 on drift, for CI.

## Working on dmx

```bash
make help    # every target
make ci      # every gate CI runs: fmt, clippy, duplication, tests, Dart, website, build
```

[The specifications](docs/specs/SPEC.md) and [plans](docs/plans/PLAN.md) give
every requirement a dotted identifier — `[emission.inline-backend]` — that code,
tests, and diagnostics cite, so `grep -r` returns the requirement, its
implementation, and its tests together.

## Licence

[BSD-3-Clause](LICENSE), copyright Nimblesite Pty Ltd. The crate, the extension,
the published Dart package, and the generated code all ship under it.
