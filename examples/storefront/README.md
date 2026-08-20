# The storefront

One shop, built with every macro dmx has. Each of the fourteen files below is
an ordinary Dart library you can read top to bottom: what you wrote is above
the `//#region` divider, what dmx wrote is below it, in the same file, with no
`part`, no mixin, and no delegating factory in between.

It all passes `dart analyze`, and `test/` covers it with 179 tests — no mocks,
no HTTP server, no fixture library.

```
dart pub get
dart analyze
dart test
```

## The catalogue

| File | Macros | What it is there to show |
|---|---|---|
| [catalog.dart](lib/catalog.dart) | `@dmx('model')` `@dmx('diff')` | Nested models, collections, enums, dates, `fieldRename`, `@dmx('key', {'ignore': true})`. A decode failure names the path it was reached by — `Product.variants[1].price` — not the Dart type that blew up. |
| [payments.dart](lib/payments.dart) | `@dmx('enum')` | Wire names that are not the Dart identifiers, labels that are neither, and `unknown:` — so a constant added upstream after you shipped is data rather than an outage. |
| [orders.dart](lib/orders.dart) | `@dmx('union')` `@dmx('model')` | A sealed state machine that decodes. The union reads its variants out of the same file: no type resolver, no build graph. |
| [checkout.dart](lib/checkout.dart) | `@dmx('validate')` `@dmx('model')` | Violations accumulate. A form that reports one problem, gets fixed, then reports the next one is how you lose someone at checkout. |
| [routes.dart](lib/routes.dart) | `@dmx('route')` `@dmx('router')` `@dmx('model')` | `location` and `parse` generated from one pattern, so they cannot disagree. The parser is a list pattern over `pathSegments` — Dart matching, not a regex over a URL. |
| [api.dart](lib/api.dart) | `@dmx('restClient')` | The interface is hand-written and readable; the eleven lines per endpoint are not. Failures are classified — transport, status, decode — never thrown. |
| [db.dart](lib/db.dart) | `@dmx('table')` `@dmx('model')` | DDL, indexes, foreign keys, upsert, and the row mapper, all from one field list, so the migration and the mapper cannot drift. Values are bound, never interpolated. |
| [tokens.dart](lib/tokens.dart) | `@dmx('lerp')` `@dmx('model')` | Interpolation that composes: a field whose type also carries `@dmx('lerp')` blends by calling its own `lerp`, so a theme animates all the way down. |
| [tool.dart](lib/tool.dart) | `@dmx('cli')` `@dmx('model')` | Two commands in one file. The parser is recursive, so nothing in it is reassigned, and the usage text is derived rather than maintained. |
| [testing.dart](lib/testing.dart) | `@dmx('fake')` `@dmx('model')` | Fixtures with no randomness in them. A failing test fails identically tomorrow, on CI, and on the machine of whoever picks it up. |
| [events.dart](lib/events.dart) | `@dmx('event')` | Analytics that cannot be misspelled. An absent optional parameter is absent, not null — a null becomes a real bucket in most dashboards. |
| [settings.dart](lib/settings.dart) | `@dmx('prefs')` | Reading settings never fails. Missing key, wrong type, unparseable date — every one falls back to the declared default. |
| [inventory.dart](lib/inventory.dart) | `@dmx('diff')` `@dmx('model')` | What changed, as data, for audit trails and unsaved-changes banners. Collections compare by content, so `diff` agrees with `==`. |
| [l10n.dart](lib/l10n.dart) | `@dmx('strings')` | A message is a method signature. `{count}` in the template must correspond to a parameter called `count`, checked at generation time rather than by a customer. |

## One model, defined in Markdown

[docs/shipping.dmx.md](docs/shipping.dmx.md) has no annotated Dart behind it at
all. The types are declared once in a typeDiagram fence, and the two Mustache
fences under it generate [lib/shipping.dart](lib/shipping.dart) — records,
a sealed union, and a typedef — and
[lib/shipping_wire.dart](lib/shipping_wire.dart), a constant wire-name table.
Both are functions of the same definition, so a field added to the diagram
changes both files together. The definition still renders as a diagram, so that
one page is the model, its documentation, and the build input.

[test/shipping_test.dart](test/shipping_test.dart) constructs the generated
types, switches over the union without a default arm, and checks that the two
generated files agree.

## Reading order

If you read one file, read [catalog.dart](lib/catalog.dart) — the decoder there
is the idea the rest of the catalogue is built on. It takes `Object?` rather
than `Map<String, dynamic>`, so a nested model's `fromJson` *is* a decoder and
composes without a cast; it carries a `path`, so failures locate themselves;
and it returns a `Result`, so a malformed payload is a value you handle rather
than an exception you remembered to catch.

If you read two, read [orders.dart](lib/orders.dart) next, for what a macro can
do once it can see the file rather than one declaration.

## The generated code follows the same rules as the rest

Below every divider in this package there is no `throw`, no `as` cast, no `!`,
no `late`, and no `dynamic`. The parameters are called `json`, `other` and
`value` — no mangled names — because you have to read this code like any other.
