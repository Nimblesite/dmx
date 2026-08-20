# dmx — typeDiagram Markdown Macro

Part of the [dmx specification](SPEC.md).

## [typediagram] typeDiagram Definitions plus Mustache Templates

dmx MUST generate Dart model source from typeDiagram definitions embedded in Markdown and user-authored Mustache templates:

```mermaid
flowchart LR
    markdown["Markdown document"] --> fences["CommonMark fenced blocks"]
    fences --> invocation["Built-in typeDiagram macro invocation"]
    invocation --> model["Native Rust typeDiagram model"]
    model --> context["dmx template context"]
    context --> render["Mustache render"]
    render --> validation["Hygiene and Dart validation"]
    validation --> output["Owned generated file"]
```

[typeDiagram](https://typediagram.dev/docs/) supplies the model language and semantics. Mustache supplies the output shape. dmx joins them and owns parsing, validation, deterministic execution, and safe file emission. The production path MUST NOT invoke the typeDiagram CLI, library, runtime, `--to dart`, or any other typeDiagram language emitter.

### [typediagram.macro] Built-in Macro

`typeDiagram` is a built-in dmx macro. Its target is a Markdown generation group rather than a Dart declaration, so it is activated by [typediagram.binding] instead of an `@dmx('typeDiagram')` annotation.

The Markdown front end MUST synthesize one immutable macro invocation per definition/template group and dispatch it through the same built-in macro registry as annotation-triggered macros. The invocation carries the typeDiagram source span, each bound template and output path, and the parsed model. The macro's Rust context builder enriches that model for Mustache; each render becomes a macro-authored whole file using the existing output branch in [dartmacros.files].

Built-in resolution, determinism, diagnostics, caching, explain output, hygiene, validation, and emission rules apply unchanged. The different trigger syntax MUST NOT create a second macro engine or bypass the shared pipeline.

### [typediagram.documents] Source Documents

`dmx build <path>` and `dmx watch <path>` MUST accept an explicit Markdown file. Recursive discovery MUST include files named `*.dmx.md` and MUST ignore other Markdown files unless they are passed explicitly.

dmx MUST parse Markdown with a CommonMark-compatible parser and inspect fenced-code nodes. It MUST NOT locate fences with regex. Prose, headings, links, lists, quotes, HTML, and unrelated fenced blocks are documentation and MUST remain untouched byte-for-byte.

A typeDiagram source fence uses backticks, has the ordinary upstream-compatible info string `typeDiagram`, compared case-insensitively, and contains valid typeDiagram DSL. The fence MUST remain renderable by typeDiagram's Markdown tooling; dmx-specific metadata is therefore never added to the typeDiagram fence.

### [typediagram.binding] Definition-to-Template Binding

A generation group is one typeDiagram fence followed immediately in the Markdown AST by one or more dmx-enabled Mustache fences. Blank lines do not create AST nodes and do not break the group. Any other Markdown node ends the group.

A dmx-enabled Mustache fence uses `mustache` as its language and a JSON object as the remainder of its info string. The object MUST contain `dmx.output`, an output path relative to the document's output root ([typediagram.output]), and MAY contain `dmx.target`, the name of a generation target, defaulting to `dart`. Any other key under `dmx` is an error rather than a value dmx ignores, so a misspelling is reported instead of silently generating nothing. Metadata that does not begin with `{` belongs to another convention and MUST be left alone:

```typeDiagram
type Product {
  id: String
  name: String
  price: Decimal
}
```

````markdown
## Store models

A template binds to the definition immediately above it, so the two fences are
consecutive: a heading between them would end the group.

```typeDiagram
type Product {
  id: String
  name: String
  price: Decimal
}
```

```mustache {"dmx":{"output":"lib/models/store.dart"}}
{{#declarations}}
{{#isRecord}}
final class {{name}}{{genericDeclaration}} {
  const {{name}}({{#fields}}required this.{{name}}{{comma}}{{/fields}});
{{#fields}}
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/isRecord}}
{{/declarations}}
```
````

One typeDiagram fence MAY feed several consecutive dmx-enabled Mustache fences, allowing the same model to generate several files without duplicating definitions. A typeDiagram fence with no bound dmx template remains documentation-only and MUST be ignored by dmx. A Mustache fence without `dmx` metadata is an example and MUST be ignored.

Malformed metadata, a dmx-enabled Mustache fence without an immediately preceding definition group, or two templates resolving to the same output path MUST fail the build. Association MUST never depend on a heading's text, fence ordinal across the document, or implicit global state.

### [typediagram.model] Model and Context

dmx MUST tokenize, parse, resolve, and validate the definition natively in Rust with typeDiagram-compatible semantics, then build one immutable context for each generation group. Production generation MUST require no Node process, npm package, `typediagram` executable, or network access. The compatibility baseline MUST be pinned and covered in development by differential fixtures against typeDiagram's public parser and versioned model JSON.

Declaration order, field order, variant order, generic parameter order, explicit discriminants, and recursively nested type arguments MUST be preserved. Unknown or unsupported type references MUST fail before rendering rather than pass through as source text.

The Mustache root contains `source`, `declarations`, and `modelVersion`. `source` contains the Markdown path and one-based fence position. `declarations` contains each typeDiagram declaration exactly once in source order.

Every declaration exposes `kind`, `name`, `generics`, and mutually exclusive `isRecord`, `isUnion`, `isAlias`, and `isFunction` booleans. Records expose `fields`; unions expose `variants`; aliases expose `target`; functions expose `signatures`. Nested members carry `first`, `last`, and `comma` values so templates remain logic-free. Every type reference exposes its canonical typeDiagram spelling and a precomputed `dartType`; templates MUST NOT implement type resolution or Dart type mapping.

The context builder MAY add further derived strings and booleans, but it MUST NOT discard or reorder source model data. Context schema changes require a version bump and golden fixtures.

Every target-language decision MUST be confined to one generation target: the mapping from a resolved reference to that language's type text, the extension its outputs carry, and the validation a finished file passes. Nothing else in the feature — tokenizer, parser, model, binder, context builder, emitter — may name a language. A target a document names but this build does not carry is `DMX8007`.

### [typediagram.templates] Rendering

The built-in `typeDiagram` macro renders each bound Mustache body once against its group's complete context. It follows all determinism, partial-resolution, span-mapping, and no-I/O requirements in [rendering]. All target-language decisions needed by the template MUST be finished in the macro's Rust context builder; the template only selects and places prepared values.

A template failure MUST identify the Markdown file, template fence, template line, and bound typeDiagram fence. Rendering one output MUST NOT mutate context observed by another output in the same group.

### [typediagram.output] Validation and Emission

`dmx.output` MUST resolve against the document's **output root**: the nearest ancestor of the document that carries a project marker any target recognises — `pubspec.yaml` for Dart — bounded by the workspace, and the workspace itself when there is none. `lib/models.dart` therefore means *this package's* `lib`, so a document generates the same bytes in the same place whether dmx was run from the package, from the repository root, or from an editor that opened the whole tree.

`dmx.output` MUST normalize to a path inside that root, MUST carry the extension its target generates, and MUST NOT traverse a symbolic link outside it. Absolute paths and parent traversal are errors, and an output path equal to the source document is an error. A document is identified, in its ownership markers and its templates' contexts, by its path relative to that same root, so nothing recorded in a generated file depends on where dmx was launched.

Rendered output MUST pass the same whitespace normalization, hygiene, full-file Dart re-parse, and `dart analyze --fatal-infos` corpus gates as other generated Dart. It MUST NOT contain `throw`, casts, null assertions, or other constructs forbidden in generated Dart; that is [hygiene], enforced over the tree-sitter CST rather than over the text. The file MUST carry a dmx ownership marker containing the source Markdown path, fence identity, template hash, typeDiagram definition hash, context version, and dmx version. Its first line MUST be the same ownership marker whole-file emission already uses [dartmacros.files], so one predicate decides ownership for every backend that writes a file dmx owns.

Whole-file emission follows [dartmacros.files]: never overwrite an unmarked file, write atomically, avoid no-op writes, remove stale owned outputs when their template disappears, and report drift without writing under `--check`. The source Markdown is never rewritten.

An output MUST have one live source. A target already carrying another source's ownership marker MUST be refused while that source still exists, because each pass would otherwise undo the other's. A marker naming a source that is gone identifies an orphan, and taking it over is what renaming a document is supposed to do.

### [typediagram.execution] Build, Check, Watch, and Explain

`build`, `check`, and `watch` MUST treat the Markdown document, definition fence, template fence, and resolved partials as dependencies of every output. A change to prose outside a generation group MUST NOT invalidate its output. A semantic definition or template change MUST invalidate every dependent output.

`watch` MUST retain the last valid output after an invalid edit and recover on the next valid save. `dmx explain <file.dmx.md>` MUST print each group, its source spans, normalized output paths, dependency hashes, and exact context JSON without rendering or writing.

Stale collection is scoped to the roots the pass was asked to manage: an output whose ownership marker names this document, which the document no longer produces, MUST be removed (or, under `--check`, reported) when it is inside those roots.

### [typediagram.diagnostics] Diagnostics

The feature owns the `DMX8xxx` range:

| Code | Meaning |
|---|---|
| `DMX8001` | Malformed or incomplete JSON metadata on a dmx-enabled Mustache fence |
| `DMX8002` | dmx-enabled Mustache fence is not bound to a typeDiagram definition group |
| `DMX8003` | Two templates claim the same normalized output path |
| `DMX8004` | typeDiagram definition failed parsing, resolution, or compatibility validation |
| `DMX8005` | Output path is absolute, escapes the workspace, crosses an unsafe symlink, or is not Dart |
| `DMX8006` | Output exists without the matching dmx ownership marker |
| `DMX8007` | typeDiagram compatibility or context schema version is unsupported |
| `DMX8008` | A bound Mustache template does not compile, or its render is not source the target accepts |

Every diagnostic MUST carry the Markdown path and fenced-block span. When applicable it also carries the typeDiagram line/column, template line/column, generated Dart line/column, and output path.

Rendered source that does not parse, or that breaks [hygiene], is refused by the shared diagnostics those stages already own — `DMX4001` and `DMX4003` — wrapped in a `DMX8008` that names the document, the group, and the template fence. A macro name this registry serves from a Markdown group MUST NOT be reachable as an annotation: `@dmx('typeDiagram')` is `DMX2006`.
