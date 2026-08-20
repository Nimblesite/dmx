---
layout: layouts/docs.njk
title: Models in Markdown
description: Define your types once in a *.dmx.md document and let Mustache templates write the Dart files.
eleventyNavigation:
  key: Models in Markdown
  order: 4
---

# Models in Markdown

Sometimes there is no Dart file to annotate yet. The types exist in a design
document, an API contract, or somebody's head, and writing them out in Dart
first — then annotating that Dart — is work you only do so that a generator has
something to read.

A `*.dmx.md` document skips it. You write the types once, in a
[typeDiagram](https://typediagram.dev/docs/) fence, and put the Mustache
templates that generate from them immediately below. Save the document and dmx
writes the `.dart` files those templates name.

The fence is an ordinary typeDiagram fence, so the same page still renders as a
diagram anywhere typeDiagram is supported. One page is the model, the
documentation, and the build input.

## A whole document

{% raw %}
````markdown
# Shipping

```typeDiagram
type Parcel {
  id:      Uuid
  weightG: Int
  insured: Option<Decimal>
  labels:  List<String>
}
```

```mustache {"dmx":{"output":"lib/parcel.dart"}}
{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
```
````
{% endraw %}

Save it and `lib/parcel.dart` appears:

```dart
// dmx: generated from docs/shipping.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition bd16c86d…, template abae1bb2…, context v1, dmx 0.3.0.

final class Parcel {
  const Parcel({required this.id, required this.weightG, this.insured, required this.labels});

  final String id;
  final int weightG;
  final String? insured;
  final List<String> labels;
}
```

Note what the template did not have to do. `Option<Decimal>` became `String?`
and `List<String>` became `List<String>` before the template ran, and
`constructorParameters` arrived already written — `required` on the fields that
need it, plain on the optional one. Templates place prepared values; they never
work out Dart types.

## How a template binds to a definition

A template belongs to the typeDiagram fence **immediately above it**. Blank
lines are fine; anything else — a heading, a paragraph, another fence — ends
the group. Nothing depends on a heading's text or on where the fence sits in
the document, so a template can never quietly attach to the wrong definition.

One definition can feed several templates, as long as their fences follow it
one after another:

````markdown
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

Both files are functions of the same definition, so they cannot disagree. Add a
field and both change. Delete a template fence and its file is removed.

A typeDiagram fence with no template under it is documentation and generates
nothing. A `mustache` fence with no `dmx` metadata is an example and generates
nothing. Everything else in the document — prose, headings, links, code in
other languages — is left exactly as you wrote it. dmx never rewrites the
document.

## The fence metadata

The JSON object after `mustache` is the whole configuration:

| Key | Meaning |
| --- | --- |
| `dmx.output` | Required. The file to write, relative to the package the document belongs to — the nearest `pubspec.yaml`. |
| `dmx.target` | Optional, `dart` by default. The language the output is written in. |

`dmx.output` cannot be an absolute path, cannot climb out of the package with
`..`, cannot be the document itself, and must end in the extension its target
generates. A misspelled key is reported rather than silently ignored.

## Seeing what a template will get

`dmx explain` prints each generation group, the files it writes, the digests
its outputs depend on, and the exact context the templates will render against:

```bash
dmx explain docs/shipping.dmx.md
```

It writes nothing. It is the fastest way to find out what a name is called
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
| `first`, `last`, `comma`, `index` | every list member | Separators without arithmetic |

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
