# dmx — Worked Examples

Part of the [dmx implementation plan](PLAN.md).

## [corpus] Worked Examples

One example file showed one macro on one shape. The example package is now a single coherent domain — a storefront — so the macros are seen *composing* rather than posing. Every file analyzes clean under `dart analyze`, and every file has a test suite that runs its generated members as real behaviour.

- [x] `catalog.dart` — `@dmx('model')` + `@dmx('diff')`: nesting, collections, enums, dates, `fieldRename`, `@dmx('key', {'ignore': true})`, path-carrying decode failures
- [x] `payments.dart` — `@dmx('enum')`: wire names, labels, `unknown:` fallback, and a hand-written member sitting beside five generated ones
- [x] `orders.dart` — `@dmx('union')`: sealed state machine, discriminated JSON, exhaustive `when`, narrowing accessors
- [x] `checkout.dart` — `@dmx('validate')`: accumulating violations keyed by field
- [x] `routes.dart` — `@dmx('route')` + `@dmx('router')` + `@dmx('model')`: deep links that build and parse from one pattern
- [x] `api.dart` — `@dmx('restClient')`: interface hand-written, implementation generated, failures classified rather than thrown
- [x] `db.dart` — `@dmx('table')`: DDL, indexes, foreign keys, parameterised statements, row mapping
- [x] `tokens.dart` — `@dmx('lerp')`: interpolation composing through nested types
- [x] `tool.dart` — `@dmx('cli')`: two commands, recursive parser, usage text
- [x] `testing.dart` — `@dmx('fake')`: deterministic fixtures, no randomness
- [x] `events.dart` — `@dmx('event')`: analytics names and flat parameters
- [x] `settings.dart` — `@dmx('prefs')`: typed settings that never fail to load
- [x] `inventory.dart` — `@dmx('diff')`: audit-trail changes, collections by content
- [x] `l10n.dart` — `@dmx('strings')`: typed placeholders and plural categories
- [x] Dart tests exercising every macro's output as real behaviour (`examples/storefront/test`, 179 tests, no mocks), and every `examples/storefront/lib` file regenerating byte-identically — asserted by `the_example_is_up_to_date`
- [ ] Golden corpus samples under `tests/golden` for the macros beyond `@dmx('model')`, regenerating byte-identically from the Rust pipeline
