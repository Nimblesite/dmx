# Shipping

Two files and no wrapper.

| File | What it is |
| --- | --- |
| `shipping.td` | the definition — pure typeDiagram, byte for byte what any typeDiagram tool reads |
| `shipping.wire.mustache` | a second template over the same definition — pure Mustache |

`dmx build models lib` writes [`../lib/shipping.dart`](../lib/shipping.dart)
and [`../lib/shipping_wire.dart`](../lib/shipping_wire.dart). There is no Dart
source of truth for these types, no `@dmx` annotation anywhere, and nothing to
extract from anything: the definition *is* the source.

## Where the first file comes from

`shipping.td` has no template beside it, so it renders through the **canonical
model template** dmx ships. That is the one template every model class in the
product comes out of: a `final class` with a `const` constructor, its fields,
`==`, `hashCode`, `toString` and `copyWith` — and JSON on a `ShipmentJson`
extension rather than on the class, so the class stays exactly what the diagram
said it was. The union becomes a sealed class with one case per variant, each
case an immutable value in its own right, and the alias becomes a `typedef`.

To reshape that output, write `shipping.mustache` beside `shipping.td` and it
takes the canonical template's place.

## What the second file does

`shipping.wire.mustache` reads the same definition and writes something
completely different: the snake-case wire names each declaration uses, as a
constant table a serializer can index. One definition, two outputs, no copying.

## How the files are bound

By their names, and by nothing else. `shipping.wire.mustache` renders the
`shipping.td` beside it into `lib/shipping_wire.dart`. Add a third template and
you have a third file; delete one and its file goes with it.

A template that wants a different destination says so in a leading Mustache
comment, which every engine renders to nothing:

```mustache
{{! dmx output=lib/models/shipping.dart }}
```

Change a field in `shipping.td` and both outputs move together, because both
are functions of the same definition.
