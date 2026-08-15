# dmx

Dart code generation that runs when you save. Annotate a class, and `copyWith`,
equality, `hashCode`, `toString` and typed JSON appear **inside the same file**,
below a divider — no `part` directive, no `.g.dart`, no mixin, no delegating
factory, and no `build_runner` run.

This package holds the `@dmx` annotation, the types the generated code uses,
and the API for writing your own macros in Dart. The generating itself is done
by the dmx tool, which the
[VS Code extension](https://marketplace.visualstudio.com/items?itemName=Nimblesite.dmx)
bundles and runs for you.

[Run the real generator in your browser](https://dmx.dev/playground.html) ·
[Documentation](https://dmx.dev/docs/) ·
[Source](https://github.com/Nimblesite/dmx)

## Install

Add the package your generated code composes with:

```bash
dart pub add dmx
```

Then install the generator. In VS Code, install
**dmx — Dart code generation**; it carries the generator and starts watching
when you open a trusted Dart workspace, so there is no command to run:

```bash
code --install-extension nimblesite.dmx
```

In any other editor, install the CLI and start the watcher once:

```bash
brew install nimblesite/tap/dmx   # or: scoop install dmx
dmx watch lib
```

## Annotate a class and save it

What you write:

```dart
import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({required this.id, required this.name, this.email});

  final String id;
  final String name;
  final String? email;
}
```

What is in the file after you save it — complete, unabridged, and in the same
file you were already looking at:

```dart
  //#region
  static Result<User, DecodeError> fromJson(Object? json, [String path = 'User']) =>
      switch (json) {
        {
          'id': final String id,
          'name': final String name,
        } =>
          switch ((
            dmxNullable<String>(dmxKey(json, 'email'), '$path.email', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final email),
            ) =>
              Ok(User(
                id: id,
                name: name,
                email: email,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'User', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'name': name,
        'email': email,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is User &&
          other.id == id &&
          other.name == name &&
          other.email == email);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        name,
        email,
      );

  @override
  String toString() => 'User(id: $id, name: $name, email: $email)';

  User copyWith({
    String? id,
    String? name,
    DmxPatch<String?> email = const DmxKeep(),
  }) =>
      User(
        id: id ?? this.id,
        name: name ?? this.name,
        email: switch (email) { DmxKeep() => this.email, DmxTo(value: final value) => value },
      );
  //#endregion
```

dmx owns the bare `//#region` block and nothing else. Everything above it —
your constructor, your fields, your handwritten members, your comments — is
yours, and the emitter checks those bytes before it writes. A region you label,
such as `//#region Helpers`, is yours too.

## What the runtime gives you

Generated code calls into this package rather than inlining its own helpers.
Nothing here throws, and nothing here casts.

| | |
| --- | --- |
| `Result<T, E>`, `Ok`, `Err` | Sealed, so a `switch` over a decode is exhaustive at compile time |
| `DecodeError` | Why a decode failed, and where: `User.tags[2]: expected String, got 42 (int)` |
| `dmxString`, `dmxInt`, `dmxDouble`, `dmxList`, `dmxSet`, `dmxMap`, `dmxNullable` | The decoders the JSON codec composes |
| `DmxPatch`, `DmxKeep`, `DmxTo` | `copyWith()` keeps a field, `copyWith(email: DmxTo(null))` clears it, and `copyWith(email: DmxTo(42))` does not compile |
| `dmxDeepEquals`, `dmxDeepHash` | Structural equality and a hash that agrees with it, for `List`, `Set` and `Map` fields |
| `DmxTransport`, `DmxRequest`, `DmxResponse` | What a generated REST client calls, so your app picks its own HTTP client |

Decoding returns a value, so a malformed payload is a branch rather than a
crash:

```dart
switch (User.fromJson(payload)) {
  case Ok(value: final user):
    render(user);
  case Err(error: final e):
    log('${e.path}: expected ${e.expected}');
}
```

## The built-in macros

Eleven macros ship in the current release. `@dmx('model')` covers the familiar
Freezed and dart_mappable jobs; the rest generate from the same declaration you
already wrote.

| | | |
| --- | --- | --- |
| `@dmx('model')` immutable data class | `@dmx('union')` sealed sum types | `@dmx('enum')` wire-safe enums |
| `@dmx('diff')` changes as data | `@dmx('lerp')` interpolation | `@dmx('validate')` accumulating constraints |
| `@dmx('table')` SQL schema and rows | `@dmx('route')` typed deep links | `@dmx('cli')` argument parsing and usage |
| `@dmx('fake')` deterministic fixtures | `@dmx('restClient')` HTTP implementations | |

A second argument configures one: `@dmx('model', {'fieldRename': 'snake'})`
renames every JSON key, and `@dmx('key', {'name': 'order_id'})` on a field
renames one. The
[macro catalogue](https://dmx.dev/docs/macros/)
lists every option.

## Change what a macro emits

Every built-in renders through a Mustache template, and that template is a file
you can edit. Copy the one you want into your project, change how the members
are laid out, and dmx uses yours instead.

dmx still works out every decode, encode, equality, hash and copy expression
and hands the template the finished code, so changing how the output looks
never means working out Dart types yourself.

## Write a macro in Dart

A custom macro is a Dart program you write. dmx hands it a typed view of the
annotated declaration — the class name, its fields, their types, its
annotations — and the macro returns the Dart to emit:

```dart
import 'package:dmx/macros.dart';

final class Audit extends DmxMacro {
  @override
  String get name => 'audit';

  @override
  DmxOutput expand(DmxInvocation invocation) =>
      DmxFragment('  // ${invocation.declaration.name}\n');
}

void main() => dmxServeMacros([Audit()]);
```

`expand` returns a `DmxFragment` to add members to the annotated class, a
`DmxGeneratedFile` to author a whole file of its own, or a `DmxRefusal` to
report a diagnostic and leave the source alone.

A macro can also hand its model to a Mustache template and let dmx render it:
`invocation.templates.render(…)` reaches the same engine the built-ins use.
That is how a project keeps generation logic and output shape in separate
files — the macro answers the questions only your project can answer, and the
template decides what the emitted Dart looks like.

Two worked examples do exactly that: one
[reads a live SQLite database](https://github.com/Nimblesite/dmx/tree/main/examples/dmx_sqlite_example)
and generates a row class per table, and one
[reads an OpenAPI document](https://github.com/Nimblesite/dmx/tree/main/examples/dmx_openapi_example)
and renders a typed client through the project's own templates.

Today your macro sees the class as dmx parsed it, not the analyzer's full
picture of your program, so there is no type inference to lean on yet. That is
planned.

## It never writes broken Dart

dmx parses the whole finished file before writing it, so only valid Dart is
ever saved. A broken template or a model it cannot handle gives you an error
message, and your file is left exactly as it was.

## Licence

[BSD-3-Clause](LICENSE), copyright Nimblesite Pty Ltd. The generated code is
ordinary Dart and ships under it too.
