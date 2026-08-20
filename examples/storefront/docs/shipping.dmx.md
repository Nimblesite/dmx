# Shipping

Everything under the diagram is generated from it. There is no Dart source of
truth for these types and no `@dmx` annotation anywhere — the definition *is*
the source, the templates decide the shape, and `dmx build docs lib` writes the
files. The fence renders as a diagram in any typeDiagram viewer, so this page is
documentation and a build input at the same time.

## What the two templates do

The first template turns every declaration into immutable Dart: records become
`final class`es with a `const` constructor, the union becomes a sealed class
with one subclass per variant, and the alias becomes a `typedef`. It places
prepared values and computes nothing — `dartType`, `constructorParameters` and
`owner` are all finished before it runs.

The second reads the same definition and writes something completely different:
the snake-case wire names each declaration uses, as a constant table a
serializer can index. One definition, two outputs, no copying.

## The definition and its templates

A template binds to the definition immediately above it, so the fences below are
consecutive: a heading between them would end the group and orphan the template.
That is the whole binding rule — no ordinals, no headings, no document-global
state.

```typeDiagram
# A parcel on its way to a customer.
type Parcel {
  id:      Uuid
  weightG: Int
  insured: Option<Decimal>
  labels:  List<String>
}

# Where the parcel has got to. One of these, never two.
union Leg {
  Pickup    { at: DateTime }
  Transit   { carrier: String, etaHours: Int }
  Delivered { at: DateTime, signedBy: Option<String> }
}

alias TrackingNumber = String

type Shipment {
  parcel:   Parcel
  legs:     List<Leg>
  tracking: TrackingNumber
}
```

```mustache {"dmx":{"output":"lib/shipping.dart"}}
// Generated from docs/shipping.dmx.md. Edit the diagram, not this file.
{{#declarations}}
{{#isAlias}}

/// `{{name}}` as the diagram declares it.
typedef {{name}}{{{genericDeclaration}}} = {{{dartType}}};
{{/isAlias}}
{{#isRecord}}

/// {{label}}, generated from the shipping diagram.
final class {{name}}{{{genericDeclaration}}} {
  /// Every field of {{label}}, in the order the diagram declares them.
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  /// The `{{name}}` field, declared as `{{{typeDiagram}}}`.
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/isRecord}}
{{#isUnion}}

/// {{label}} — exactly one of the variants below.
sealed class {{name}}{{{genericDeclaration}}} {
  /// The shared constructor every variant delegates to.
  const {{name}}();
}
{{#variants}}

/// The `{{name}}` case of {{owner}}.
final class {{name}} extends {{owner}}{{{ownerGenericDeclaration}}} {
  /// Every field of this case, in diagram order.
  const {{name}}({{{constructorParameters}}}) : super();
{{#fields}}

  /// The `{{name}}` field, declared as `{{{typeDiagram}}}`.
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/variants}}
{{/isUnion}}
{{/declarations}}
```

```mustache {"dmx":{"output":"lib/shipping_wire.dart"}}
// Generated from docs/shipping.dmx.md. Edit the diagram, not this file.

/// The wire name of every field, keyed by declaration and then by Dart name.
const shippingWireNames = <String, Map<String, String>>{
{{#declarations}}
{{#isRecord}}
  '{{name}}': <String, String>{
{{#fields}}
    '{{name}}': '{{snakeName}}',
{{/fields}}
  },
{{/isRecord}}
{{#isUnion}}
{{#variants}}
  '{{owner}}.{{name}}': <String, String>{
{{#fields}}
    '{{name}}': '{{snakeName}}',
{{/fields}}
  },
{{/variants}}
{{/isUnion}}
{{/declarations}}
};

/// Every declaration the shipping diagram carries, in source order.
const shippingDeclarations = <String>[
{{#declarations}}
  '{{name}}',
{{/declarations}}
];
```

Delete either fence and its file goes with it. Change a field and both files
move together, because both are functions of the same definition.
