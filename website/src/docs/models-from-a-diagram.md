---
layout: layouts/docs.njk
title: Models from a diagram
description: Define your types once as a typeDiagram definition and dmx writes the Dart — immutable classes with value equality and JSON beside them.
eleventyNavigation:
  key: Models from a diagram
  order: 4
---

# Models from a diagram

Sometimes there is no Dart file to annotate yet. The types exist in a design
document, an API contract, or somebody's head, and writing them out in Dart
first — then annotating that Dart — is work you only do so that a generator has
something to read.

A [typeDiagram](https://typediagram.dev/docs/) definition skips it. You write
the types once and save. dmx writes the Dart.

## Two files

```text
models/parcel.td          the definition
lib/parcel.dart           what dmx writes
```

Nothing is embedded in anything. `parcel.td` is pure typeDiagram — the same
file any typeDiagram tool reads, and the same one that renders as a diagram:

```typeDiagram
type Parcel {
  id:      Uuid
  weightG: Int
  insured: Option<Decimal>
  labels:  List<String>
}
```

Run `dmx build models lib` — or leave the watcher running and just save — and
`lib/parcel.dart` appears:

```dart
// dmx: generated from models/parcel.td — do not edit.
// dmx: rendered through the canonical model template, definition bd16c86d…, template b1cdad67…, context v1, dmx 0.3.0.

// Generated from models/parcel.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// Parcel — an immutable value from the diagram.
final class Parcel {
  /// Every field, in the order the diagram declares them.
  const Parcel({required this.id, required this.weightG, this.insured, required this.labels});

  final String id;
  final int weightG;
  final String? insured;
  final List<String> labels;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Parcel &&
          other.id == id &&
          other.weightG == weightG &&
          other.insured == insured &&
          dmx.dmxDeepEquals(other.labels, labels));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        weightG,
        insured,
        dmx.dmxDeepHash(labels),
      );

  @override
  String toString() => 'Parcel(id: $id, weightG: $weightG, insured: $insured, labels: $labels)';

  /// A copy of this value with the named fields replaced.
  Parcel copyWith({
    String? id,
    int? weightG,
    dmx.DmxPatch<String?> insured = const dmx.DmxKeep(),
    List<String>? labels,
  }) => …;
}

/// JSON for [Parcel].
extension ParcelJson on Parcel {
  static dmx.Result<Parcel, dmx.DecodeError> fromJson(Object? json, [String path = 'Parcel']) => …;

  Map<String, Object?> toJson() => …;
}
```

## One canonical model template

There is no template in that project, and there did not need to be. A
definition with nothing beside it renders through the **canonical model
template** dmx ships — one template, compiled into the binary, and every model
class dmx generates from a diagram comes out of it.

What it writes is a value, not a bag of fields. Two parcels built from the same
data are equal and hash alike; the `List<String>` compares by content, not by
reference; a nullable field's `copyWith` tells "leave it alone" apart from "set
it to null". A union becomes a sealed class with one immutable case per
variant, and an alias becomes a `typedef`.

**JSON lives beside the class, not inside it.** `toJson` and `fromJson` are on
the `ParcelJson` extension, so the class reads as exactly what the diagram said
and nothing more. Nested types decode through their own extensions, and a
union's extension reads the case's tag out of the payload.

A declaration dmx cannot build a codec for still gets its class and its value
semantics — it just has no extension. That happens for a generic declaration, an
untagged union, a `Unit` member, and a map keyed by anything but a string;
`dmx explain` names the reason.

Note what none of this asked you to write. `Option<Decimal>` became `String?`
and `Uuid` became `String` before anything was rendered, and every comparison,
hash, and decode expression was finished in Rust. Templates place prepared
values; they never work out Dart types.

## Deciding the shape yourself

Put `parcel.mustache` beside `parcel.td` and it takes the canonical template's
place. It is pure Mustache — no front matter, no directives, nothing an engine
would choke on:

{% raw %}
```mustache
{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
```
{% endraw %}

Save, and `lib/parcel.dart` is what your template says instead.

## How a template binds to a definition

By its name, and by nothing else. `parcel.mustache` renders the `parcel.td`
beside it. A dotted suffix is a second template over the same definition:

| Template | Definition | Output |
| --- | --- | --- |
| none | `parcel.td` | `lib/parcel.dart`, from the canonical model template |
| `parcel.mustache` | `parcel.td` | `lib/parcel.dart`, instead of the canonical one |
| `parcel.wire.mustache` | `parcel.td` | `lib/parcel_wire.dart`, as well |

Every file is a function of the same definition, so they cannot disagree. Add a
field and they all change. Delete a template and its file is removed — except
the first, which goes back to the canonical template.

A `.mustache` file with no definition beside it is somebody else's Mustache file
and is left alone. When a template's name matches two definitions — `parcel.td`
and `parcel.wire.td` both present — it binds to the longer one.

## Sending the output somewhere else

By default a template writes to `lib/`, under the name it has, in the extension
its target generates. To send it elsewhere, put a Mustache comment on the first
line:

{% raw %}
```mustache
{{! dmx output=lib/models/parcel.dart target=dart }}
```
{% endraw %}

It is a comment, so every Mustache engine renders it to nothing and the
template is still an ordinary template.

| Setting | Meaning |
| --- | --- |
| `output` | The file to write, relative to the package the definition belongs to — the nearest `pubspec.yaml`. |
| `target` | Optional, `dart` by default. The language the output is written in. |

`output` cannot be an absolute path, cannot climb out of the package with `..`,
cannot be the definition itself, and must end in the extension its target
generates. A misspelled key is reported rather than silently ignored. The
settings are `key=value` pairs rather than JSON because a Mustache comment ends
at the first `}` inside it.

## Or keep it all on one page

When the model, the diagram, and the prose explaining them belong together, put
the definition and its templates in a `*.dmx.md` document instead. A template
binds to the typeDiagram fence **immediately above it**; blank lines are fine,
anything else — a heading, a paragraph, another fence — ends the group:

{% raw %}
````markdown
# Shipping

```typeDiagram
type Parcel { id: Uuid, weightG: Int }
```

```mustache {"dmx":{"output":"lib/parcel.dart"}}
…the model classes…
```

```mustache {"dmx":{"output":"lib/parcel_wire.dart"}}
…the wire-name table…
```
````
{% endraw %}

Here `dmx.output` is required — a fence has no file name to derive one from —
and `dmx.target` is the same optional key. Everything else in the document —
prose, headings, links, code in other languages — is left exactly as you wrote
it. dmx never rewrites the document.

## Seeing what a template will get

`dmx explain` prints each generation group, the files it writes, the digests
its outputs depend on, and the exact context the templates will render against:

```bash
dmx explain models/parcel.td
```

It writes nothing, and it takes a definition, a template bound to one, or a
`*.dmx.md` document. It is the fastest way to find out what a name is called
before you use it.

## What the templates can read

The root of the context carries `modelVersion`, `target`, `source`, and
`declarations`. Every declaration appears once, in the order you wrote it, with
mutually exclusive `isRecord`, `isUnion`, `isAlias` and `isFunction` flags, so a
template selects a shape rather than filtering a list.

| Name | On | What it is |
| --- | --- | --- |
| `name`, `camelName`, `pascalName`, `snakeName`, `screamingSnakeName`, `label` | declarations, fields, variants | The identifier, in every casing |
| `genericDeclaration` | declarations | `<A, B>`, or empty |
| `constructorParameters` | records, variants | `{required this.a, this.b}`, ready to place |
| `dartType`, `targetType` | fields, aliases, returns | The Dart type text, already resolved |
| `typeDiagram` | fields | The type as the diagram spells it |
| `isOptional`, `isRequired`, `parameter` | fields | Whether it is an `Option`, and its constructor fragment |
| `owner`, `ownerGenericDeclaration` | variants | The union the variant belongs to, which its own `name` would otherwise hide |
| `discriminant`, `hasDiscriminant`, `isTuple`, `isBare` | variants | The variant's shape |
| `untagged` | unions | Whether the cases are told apart by shape rather than a tag |
| `signatures`, `hasOverloads` | functions | Every overload, and whether there is more than one |
| `parameterList`, `returnType`, `isAsync`, `params`, `isOverload` | signatures | One signature, ready to place |
| `first`, `last`, `comma`, `index` | every list member | Separators without arithmetic |

A tuple variant's positional members arrive as `value1`, `value2`, … The
diagram spells them `_0`, `_1`, and the model keeps that spelling, but a
leading underscore is private in Dart — illegal as a named constructor
parameter and dead as a field — so the target sees a name it can compile.

## Two things worth knowing before you write a template

{% raw %}
**Use `{{{triple}}}` braces for anything holding a type.** `{{name}}` escapes
its value as HTML, so `{{genericDeclaration}}` renders `<T>` as `&lt;T&gt;` and
the file fails validation rather than being written. Every value that can hold
`<`, `>` or `&` — `dartType`, `targetType`, `genericDeclaration`,
`ownerGenericDeclaration`, `parameterList`, `returnType`,
`constructorParameters` — wants triple braces.

**A section reads names from the level it was entered on.** Opening
`{{#hasOverloads}}` inside `{{#signatures}}` finds `hasOverloads` on the
*function*, so `{{index}}` inside that section is the function's ordinal, not
the signature's. That is why a signature carries its own `isOverload`: entering
the section on the signature's own name keeps the signature in scope.
{% endraw %}

## Where the definitions come from

dmx reads the typeDiagram language itself, in Rust. Installing dmx installs
nothing else: no Node, no npm package, no `typediagram` executable, and no
network access at build time. A compatibility corpus in the dmx repository holds
the parser to typeDiagram's own, fixture by fixture, so the two cannot drift
apart quietly.

The definition supplies the model. Mustache decides every generated byte.

## Safety

Generated files carry an ownership marker on their first line. dmx will not
overwrite a file that does not have one, so a hand-written file is never lost to
a typo in an output path. Rendered source is parsed as a complete file before
anything is written, and checked for the constructs generated code may not
contain — `throw`, `as` casts, `!` null assertions — so a template mistake fails
the build instead of shipping.

`dmx build --check` writes nothing and exits non-zero when an output is out of
date, which is what CI should run.
