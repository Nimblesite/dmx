# dmx — Implementation Phases

Part of the [dmx implementation plan](PLAN.md).

## [phases] Implementation Phases

| Phase | Scope | Exit |
|---|---|---|
| **P0** | tree-sitter front end, name index, context builder, `dmx explain` | C1 passes; parse/index budgets met; **Q1 answered** |
| **P1** | Mustache render + hygiene + validation + `part` backend; `@dmx('model')` equality only | C2, C3, C5, C6, C7, C11 pass |
| **P2** | `copyWith` + full codec table | C8 passes |
| **P3** | **`inline` backend**: region location, splice, region recovery, CAS, no-op writes, `git-filter clean` | **C12, C14 pass**; zero writes on a no-change build |
| **P4** | Cache, five modes, parallelism, orphan collection, `check` | C4 and [performance] met |
| **P5** | Template overrides, MiniJinja, `dmx fix` | C9 passes; a third-party template set authored with no config file |
| **P6** | Worker protocol, Dart AOT reference worker, `--verify-extensions` | C13 passes; a Dart transform and a Dart engine both round-trip |
| **P7** | **User-defined macros in Dart** ([dartmacros]): `package:dmx/macros.dart` library, `expand` op, `tool/dmx/macros.dart` discovery, resolution diagnostics, `dmx explain` | **C15 passes**; a user `@dmx` macro authored in Dart generates in the example with no config file, composed with a built-in on the same declaration |
| **P8** | `augment` backend behind a flag | Output compiles under `--enable-experiment=augmentations` |
| **P9** | Semantic substrate ([semantic-expansion.delivery]): stable declaration identity, scopes, bindings, imports, inheritance, and dependency-aware invalidation | Cross-library resolution corpus agrees with the pinned Dart analyzer oracle; every fact carries source provenance or an explicit unknown reason |
| **P10** | Constraint engine and bidirectional expression typing: nullability, generic substitution and bounds, member lookup, flow promotion, and inferred initializer types | Soundness/differential corpus passes; typed macro queries expose structured facts rather than CST labels or type strings |
| **P11** | Hygienic typed expansion: quote/splice IR, phase separation, generated-binding identity, bounded expansion, and semantic re-analysis | Nested generated declarations reach a deterministic fixed point; capture, phase leaks, cycles, and ill-typed splices fail before emission |
| **P12** | Elaborated type-system extensions: project-defined static judgments and typed DSLs that check and lower into ordinary Dart | At least one non-trivial typed DSL checks custom rules, emits evidence-backed Dart, and passes `dart analyze --fatal-infos` end to end |

P0–P3 is the minimum shippable product: zero-boilerplate inline generation of immutable models. [extensions] is deliberately late — it must never become load-bearing for the default experience. P7 is the promise the project is named for: [dartmacros] rides on P6's worker protocol, and until it lands the catalogue is closed and `dmx` is a code generator, not a macro system.

P9–P12 are explicitly **not implemented**. They are the planned evolution from a lossless parser into a full-fledged semantic and type-directed static metaprogramming system. [semantic-expansion] defines that research and delivery track.
