# dmx — Consumer Surface and Front End

Part of the [dmx specification](SPEC.md).

## [surface] Consumer Surface

```dart
import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({
    required this.id,
    required this.name,
    this.email,
    this.tags = const [],
    required this.createdAt,
  });

  final String id;
  final String name;
  final String? email;
  @dmx('key', {'name': 'created_at'})
  final DateTime createdAt;
  final List<String> tags;

  //#region dmx:generated
  //#endregion
}
```

That is the entire Dart-first consumer surface under the default `inline` backend. One import, one annotation, one empty region — and `dmx fix` or `dmx build --insert-regions` writes the region for you. The model-first alternative is [typediagram].

### [surface.annotations] The `@dmx` trigger (complete, v1)

There is exactly **one** annotation class in `package:dmx/dmx.dart`:

```dart
class dmx {
  final String macro;
  final Map<String, Object?> args;
  const dmx(this.macro, [this.args = const {}]);
}
```

`@dmx('macro')` on a declaration or member opts it into a macro by **name as data** — there is no catalogue of annotation classes to keep in sync, and adding a macro to the binary never changes the annotations package. The map carries the macro's configuration; its keys and values reach the macro as raw CST source, unevaluated. Everything else the macro needs it reads from the **structure of the annotated declaration itself** — fields, types, variants, abstract members, constructor defaults — so any arbitrary shape the CST can express is available to generation without a new annotation.

Normative rules:

- The front end MUST normalize `@dmx('name', {…})` into an annotation named `name` carrying the map's entries as named arguments, read from the CST — never from regex.
- The macro name MUST be a string literal. A `@dmx` whose first argument is anything else MUST be refused with `DMX2005`, in the author's terms.
- Any annotation other than `@dmx` (`@override`, `@Deprecated`, …) MUST be inert: it can never trigger a macro and never collides with a macro name.
- Class-level triggers: `model`, `union`, `enum`, `diff`, `lerp`, `validate`, `table`, `route`, `router`, `cli`, `fake`, `restClient`. Member-level markers: `key`, `value`, `column`, `query`, `flag`, `opt`, `rest`, `get`, `post`, `put`, `delete`, `body`, and the `check.*` constraint family.
- `@dmx('model')` with an empty map MUST produce equality, `hashCode`, `toString`, `copyWith`, and JSON. Everything is opt-*out*.

### [surface.template-choice] Choosing a template

```dart
@dmx('model', {'template': 'tool/dmx/compact'})
```

A directory path. No registration, no manifest, no version pinning. The engine is inferred from file extension.

### [surface.zero-config] Zero config

`dmx` MUST run correctly with no config file:

| Convention | Default |
|---|---|
| Backend | **`inline`** |
| Source roots | `lib/`, plus `bin/`, `test/` if present; `*.dmx.md` under requested roots |
| Excluded | `**/*.g.dart`, `**/.dart_tool/**` |
| Templates | built-in |
| Cache | `.dart_tool/dmx/` |
| Mode | `incremental` |

An OPTIONAL `dmx.yaml` MAY override these. It MUST NOT be required for any documented workflow and MUST NOT be produced by any scaffold.

---

## [context] Template Context

**A variable reference for template authors, in the sense that Mustache documents variables. Not a schema anyone writes, declares, or versions.** Built automatically; printed by `dmx explain`.

### [context.discipline] Discipline (normative)

Every conditional a template needs MUST be a precomputed boolean or finished string. Templates MUST NOT dispatch on type names, parse strings, or do arithmetic.

If a template author wants logic the context cannot express, that is first a **bug filed against the Rust context builder**. [extensions] is the escape hatch of last resort, not the answer to a missing variable.

### [context.root] Root

| Variable | Type | Meaning |
|---|---|---|
| `className` | string | `User` |
| `classNameLower` | string | `user` |
| `typeParams` / `typeParamsBare` | string | `<T, K>` / `T, K` |
| `hasTypeParams` | bool | |
| `isConst` | bool | Has a const constructor |
| `docComment` | string | |
| `fields` | list | [context.fields] |
| `fieldCount` | int | |
| `hashCombiner` | string | `Object.hash` or `Object.hashAll` |
| `needsCollectionEquality` | bool | |
| `wantsJson` / `wantsCopyWith` / `wantsToString` / `wantsEquality` | bool | From `@dmx('model')` |
| `backend` | string | `inline` \| `part` \| `augment` |
| `isInline` | bool | Templates emit bare members rather than a mixin |
| `memberIndent` | string | Indentation to prefix each member with under `inline` |

### [context.fields] `fields[]`

| Variable | Type | Meaning |
|---|---|---|
| `name` | string | `createdAt` |
| `type` / `typeNonNull` | string | `DateTime?` / `DateTime` |
| `jsonKey` | string | `'created_at'` — quoted and escaped |
| `isNullable` / `isCollection` / `isRequired` / `hasDefault` / `isIgnored` | bool | |
| `defaultValue` | string | Dart source text |
| **`decodeExpr`** | string | Finished decode expression |
| **`encodeExpr`** | string | Finished encode expression |
| **`equalsExpr`** | string | Finished comparison |
| **`hashExpr`** | string | Finished hash component |
| **`copyParam`** / **`copyArg`** | string | Finished `copyWith` parameter and argument |
| `isFirst` / `isLast` / `hasMore` | bool | Separators |

The six bolded variables are where all intelligence lands. Everything else exists so templates lay out *structure* and never touch *semantics*.

### [context.example-template] A complete template (inline backend)

```mustache
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is {{className}}{{typeParams}}{{#fields}} &&
          {{equalsExpr}}{{/fields}});

  @override
  int get hashCode => {{hashCombiner}}(
        runtimeType,
{{#fields}}
        {{hashExpr}},
{{/fields}}
      );
```

No conditionals, no type checks, no filters. That is the target.

### [context.helpers] Helpers

Identical across engines, pinned by [suite]: `camelCase`, `pascalCase`, `snakeCase`, `kebabCase`, `screamingSnakeCase`, `escapeDartString`, `escapeDartDoc`, `indent(n)`. No others — a request for one signals a missing context variable.

---

## [frontend] Front End and Resolution

### [frontend.parser] Parser

v1 MUST use `tree-sitter` with the `nielsenko/tree-sitter-dart` grammar. Under the `inline` backend the parser carries additional weight: it is also the mechanism that locates class-body boundaries and comment tokens ([emission.inline-backend.region-location]), so a lossless CST with reliable comment positions is a hard requirement, not a convenience.

```rust
pub trait Frontend {
    fn parse(&self, src: &str, path: &Path) -> Result<Tree, ParseError>;
    fn declarations(&self, tree: &Tree, src: &str) -> Vec<DeclarationHeader>;
    fn target(&self, tree: &Tree, src: &str, trigger: &Trigger) -> Result<Target, Vec<Diagnostic>>;
    fn class_body_span(&self, tree: &Tree, name: &str) -> Option<Span>;
    fn comments(&self, tree: &Tree) -> Vec<CommentToken>;
    fn validate(&self, src: &str) -> Result<(), Vec<ParseError>>;
}
```

### [frontend.migration-triggers] Migration triggers (thresholds, not opinions)

Replace tree-sitter with hand-written recursive-descent over `rowan`/`cstree` if **any** holds:

| Trigger | Threshold |
|---|---|
| Coverage | C1 failure rate > 0.5%, or grammar lags a stable Dart release by > 1 minor |
| Throughput | < 20 MB/s single-threaded |
| Recovery | Cannot localize a syntax error within 3 lines in > 10% of malformed fixtures |
| Comment fidelity | Comment token positions unreliable enough to endanger [emission.inline-backend] |
| Augmentations | Grammar lacks `augment` when that backend leaves experimental |

Precedent: Ruff replaced its generated parser with a hand-written one for roughly 2× throughput plus better recovery. Budget this as a plausible v2.

### [frontend.name-index] Name Index

Answers *"what kind of declaration is `Address`?"* — name-level resolution, not type inference. MUST parse only top-level declaration headers; MUST be cached, incrementally invalidated by content hash, and built in parallel. Collision order: same file → same package → explicitly imported → first by library URI; residual ambiguity yields `DMX2003` and unknown.

### [frontend.no-type-inference] Current semantic boundary

This section states the **v0.3 implementation boundary**, not the final ambition. The current front end exposes typed parsed declarations and a name index; it does not expose expression types, resolved bindings, generic substitutions, flow promotion, overload selection, or an analyzer element model. The lossless CST, stable spans, dependency index, and immutable context are intentionally the substrate for the future semantic graph and staged typed-expansion work in [semantic-expansion]. That work is not implemented yet.

| Unavailable | Consequence | Mitigation |
|---|---|---|
| Static type of an initializer | `var x = foo();` unknown | `DMX2001`: require explicit field types |
| Members inherited from outside the roots | Cannot know if a base defines `==` | Emit `@override` conservatively |
| Transitive cross-package typedefs | One level expanded, then unknown | `DMX2004` |
| Generic bound satisfaction | Cannot verify `T extends Comparable` | The Dart compiler is the backstop |

**Principle:** when `dmx` cannot resolve something it MUST fail loudly, never guess.

Future inference MUST preserve that principle: unresolved or ambiguous facts remain explicit semantic results with source provenance, never guessed strings attached to AST node names.

---
