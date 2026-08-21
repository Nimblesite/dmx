# dmx — typeDiagram Markdown Integration Plan

Part of the [dmx implementation plan](PLAN.md). This plan delivers the built-in `typeDiagram` macro [typediagram.macro]: typeDiagram definitions embedded in Markdown become a rich dmx context, and adjacent Mustache templates turn that context into generated Dart.

## [typediagram.delivery] Outcome

A user writes one `*.dmx.md` document containing an ordinary renderable typeDiagram fence and one or more bound Mustache fences. The Markdown front end synthesizes a `typeDiagram` macro invocation; `dmx build` generates the declared Dart files, `dmx check` detects drift, and `dmx watch` keeps them current.

The equation is deliberately small:

```text
typeDiagram definitions + Mustache templates = generated code
```

dmx parses Markdown and the typeDiagram DSL natively in Rust, then runs the invocation through its existing built-in macro pipeline. It owns context enrichment, Mustache rendering, validation, and safe emission. It does not delegate parsing or code shape to typeDiagram in production.

## [typediagram.delivery.baseline] Compatibility Baseline

The production binary has no typeDiagram package, CLI, Node, or network dependency. Pin a development-only typeDiagram oracle and record its model JSON schema version. The initial compatibility corpus follows the [typeDiagram language reference](https://typediagram.dev/docs/language-reference.html): `type`, `union`, `alias`, and `function` declarations; generic parameters; record and variant fields; tuple and unit variants; explicit discriminants; nested type references; semantic scalars; `Option`, `List`, `Map`, and `Any`; comments; and the optional file header.

Use typeDiagram's public parser, model builder, and [versioned `toJSON` model API](https://github.com/Nimblesite/typeDiagram/blob/main/packages/typediagram/src/model/json.ts) only as the oracle in differential tests. Production generation MUST NOT invoke that API or `typediagram --to dart`: the native Rust parser owns model construction and Mustache remains the only code-shape authority.

## [typediagram.delivery.phases] Delivery Phases

### [typediagram.delivery.phases.contract] TD0 — Freeze the Markdown Contract

- [x] Add golden `*.dmx.md` fixtures showing one definition/one template, one definition/several templates, several independent groups, documentation-only diagrams, and ordinary Mustache examples that dmx ignores.
- [x] Add failing fixtures for malformed JSON metadata, orphan templates, duplicate outputs, invalid typeDiagram, unsafe paths, and unowned destination files.
- [x] Freeze the JSON fence metadata and adjacency rules from [typediagram.binding].
- [x] Freeze `typeDiagram` as the built-in macro name and the synthesized invocation shape from [typediagram.macro].
- [x] Exit: every syntax choice has a fixture and no binding depends on prose text, headings, or document-global ordering.

### [typediagram.delivery.phases.markdown] TD1 — Parse and Bind Markdown

- [x] Parse Markdown into a CommonMark AST with source spans; do not scan fences with regex.
- [x] Discover `*.dmx.md` recursively while allowing any Markdown file when explicitly named.
- [x] Preserve non-generation content and distinguish plain examples from dmx-enabled Mustache fences.
- [x] Build immutable generation-group values containing the definition span, template span, metadata, and normalized output path.
- [x] Translate every valid group into one macro invocation without creating a parallel rendering path.
- [x] Exit: the binding golden suite passes over upstream-compatible backtick fences, longer fences, CRLF, Unicode, interleaved prose, unrelated code blocks, and multiple groups.

### [typediagram.delivery.phases.model] TD2 — Build the typeDiagram Model

- [x] Implement the documented typeDiagram grammar as a small tokenizing/LL(1) Rust front end, returning `Result` diagnostics with source spans.
- [x] Resolve declarations, generic parameters, built-ins, and nested references into one immutable model without casts or exceptions.
- [x] Serialize the resolved model to the same semantic shape as the pinned typeDiagram model JSON.
- [x] Differential-test every compatibility fixture against typeDiagram's public parser/model JSON in CI so upstream language drift is visible.
- [x] Reject unknown or unsupported references before Mustache rendering.
- [x] Exit: the Rust model and typeDiagram oracle agree structurally for the complete corpus, including diagnostics for invalid definitions.

### [typediagram.delivery.phases.context] TD3 — Enrich the Mustache Context

- [x] Add `src/macros/typediagram.rs` and register it as the built-in `typeDiagram` macro for Markdown generation-group targets.
- [x] Define and version the root context specified by [typediagram.model].
- [x] Preserve each declaration exactly once and in source order; add mutually exclusive kind flags instead of duplicating declarations into per-kind lists.
- [x] Precompute generic declarations, Dart type text, commas, first/last markers, constructor fragments, and every other value required to keep templates logic-free.
- [x] Reuse the existing Mustache renderer, partial resolver, span mapping, normalizer, and deterministic ordering.
- [x] Add `dmx explain` snapshots of the exact context for every declaration kind.
- [x] Exit: a Mustache template generates analyze-clean records and sealed unions without implementing type resolution inside the template.

### [typediagram.delivery.phases.emission] TD4 — Validate and Emit Whole Files

- [x] Route rendered Dart through hygiene and full-file parse validation before any write.
- [x] Add ownership headers containing both fence identities and content hashes.
- [x] Reuse macro-authored-file safety: atomic writes, no-op writes, unowned-file refusal, stale collection, and `--check` drift.
- [x] Make build cache keys depend on semantic definition content, template content, resolved partials, context version, and dmx version—not unrelated Markdown prose. *(There is no on-disk cache: the ownership marker records those digests and the no-op write compares the whole candidate file, so prose never invalidates an output and a definition or template edit always does. A persistent cache is [execution.caching]'s, not this feature's.)*
- [x] Extend watch mode to retain the last valid output during invalid edits and recover deterministically.
- [x] Exit: build/check/watch tests pass without rewriting the Markdown source or an unowned output.

### [typediagram.delivery.phases.product] TD5 — Product Integration

- [x] Add `DMX8xxx` diagnostics with Markdown, definition, template, and generated-output spans.
- [x] Add C16 to the conformance suite and enforce byte-identical generation, zero-write second builds, analyzer-clean Dart, and cross-platform paths.
- [x] Add a worked storefront document that defines models once and generates at least two Dart files from different Mustache templates.
- [x] Document the feature in the CLI help, README, website, editor highlighting, and VS Code packaging tests. *(Highlighting: `mustache` and `typeDiagram` fences inside Markdown use the editor's own Markdown grammar; dmx contributes no new grammar.)*
- [x] Add editor diagnostics and regeneration for saved `*.dmx.md` documents through the existing engine contract.
- [x] Exit: the worked document builds from a clean checkout and remains current under the full `make ci` gate.

## [typediagram.delivery.tests] Required Test Matrix

| Area | Required proof |
|---|---|
| Markdown | CommonMark AST binding; unrelated content ignored; exact source spans |
| typeDiagram | Differential model parity with the pinned upstream parser |
| Macro registry | Synthesized Markdown invocation resolves to the built-in `typeDiagram` macro and no annotation is required |
| Context | Golden JSON for every declaration and nested type shape |
| Mustache | One model to one file, one model to many files, partial dependencies |
| Safety | Invalid Dart, unsafe path, duplicate target, unowned file, stale output |
| Execution | Build, check, watch recovery, explain, cache invalidation, no-op write |
| Dart | `dart analyze --fatal-infos` and runtime tests over generated models |
| Determinism | Repeated runs, job counts, LF/CRLF input, macOS/Linux/Windows paths |

No mocks: E2E coverage drives the real `dmx` binary over real Markdown, Mustache, and Dart files. typeDiagram oracle tests invoke the pinned real package.

## [typediagram.delivery.done] What Shipped

Every phase above is implemented, tested, and gated by `make ci`.

| Where | What it is |
|---|---|
| `src/dmx/src/typediagram/{lexer,parser,ast,model}.rs` | The typeDiagram front end, in Rust, with no typeDiagram dependency |
| `src/dmx/src/typediagram/json.rs` | The model in upstream's JSON shape — the compatibility surface, read only by the differential corpus |
| `src/dmx/src/typediagram/binding.rs` | What a binding *is*, shared by both front ends — group, template, output, and the sentence each origin is located by |
| `src/dmx/src/typediagram/standalone.rs` | The `.td` front end: the canonical model template, or the `.mustache` files beside the definition |
| `src/dmx/src/typediagram/semantics.rs` | Value semantics and the JSON codec one generated class gets [typediagram.canonical] |
| `src/dmx/src/typediagram/naming.rs` | What generated code calls each union case [typediagram.canonical.names] |
| `src/dmx/templates/diagram_model.mustache` | The canonical model template itself |
| `src/dmx/src/typediagram/markdown.rs` | The Markdown front end: CommonMark binding over `pulldown-cmark`, fences as AST nodes |
| `src/dmx/src/typediagram/run.rs` | The one pipeline behind both — resolve, invoke, validate, emit, explain |
| `src/dmx/src/typediagram/context.rs` | The Mustache context, versioned by `CONTEXT_VERSION` |
| `src/dmx/src/typediagram/target.rs` | The one place a language appears: type text, extension, project marker, validation |
| `src/dmx/src/typediagram/{emit,document}.rs` | Path safety, ownership markers, stale collection, and the Markdown entry point |
| `src/dmx/src/macros/typediagram.rs` | The built-in macro, in the same registry `@dmx('model')` is in |
| `src/dmx/src/hygiene.rs` | [hygiene] as a CST check, because a user template is nobody's reviewed code |
| `src/dmx/tests/typediagram/corpus` | The `.td` fixtures and the oracle's model JSON |
| `src/dmx/tests/typediagram/golden` | The same fixtures rendered to Dart as standalone files through one shared template, committed and analyzer-gated |
| `scripts/typediagram-oracle.mjs` | Development-only regeneration of that oracle from a typeDiagram checkout |
| `examples/storefront/models/` | `shipping.td` and two templates beside it, two generated Dart files, 9 tests over them |

### [typediagram.delivery.corpus] Corpus → Dart

The `.td` fixtures proved the *model* and nothing else: they were parsed,
serialised, and compared against the oracle's JSON, and no Dart was ever
produced from them. Against [emission] — emitting Dart that does not compile is
the worst failure this repo has — model parity alone was not enough.

The whole corpus is now laid out as standalone files — `models/<name>.td`
beside `models/<name>.mustache` — built by the shipped binary in one `dmx
build`, and committed as `tests/typediagram/golden/lib/<name>.dart`. `cargo
test --test typediagram_golden` holds the bytes; `make corpus` runs `dart
analyze --fatal-infos` over them. Nothing is wrapped, assembled, or extracted:
the `.td` files are copied out of the parity corpus byte for byte, so it stays
the one place a definition is written.

- [x] Every corpus fixture renders to Dart and the output is committed and byte-gated.
- [x] `make corpus` analyzes it with `dart analyze --fatal-infos`.
- [x] Tuple variants emit a name the target can compile. typeDiagram spells positional members `_0`, `_1`, … and the model keeps that spelling; Dart cannot, because a leading underscore makes the member private — illegal as a named constructor parameter and dead as a field. The context now maps them to `value1`, `value2`, … [context.discipline]. **Every tuple variant in the language previously emitted Dart that did not compile, and nothing in the repo could see it.**
- [x] A signature carries `isOverload` so a target without overloading can name each one. `hasOverloads` on the declaration cannot be read from inside `{{#signatures}}`: a section entered on a name the *declaration* carries pushes that value with the declaration beneath it, so the ordinal read back is the declaration's.
- [x] The shipped storefront template stopped using `{{genericDeclaration}}`, which HTML-escapes `<T>` into `&lt;T&gt;`. It only ever worked there because nothing in that document is generic.

### [typediagram.delivery.standalone] Definition File → Template File → Dart

A definition and its templates had exactly one spelling: fences inside a
Markdown document. That made the plain case — a model file, a template file, a
generated file — reachable only by writing prose around it, and it put a
CommonMark parse between an author and their own definition.

`.td` + `.mustache` → `.dart` is now the primary spelling
[typediagram.standalone]. The binding is the file names; the pipeline behind it
is the same one, because both front ends build the same
`binding::Group` and everything after that is `run.rs`.

- [x] `.td` files are discovered recursively by `build` and `watch`, and accepted by name.
- [x] `<name>.mustache` binds to `<name>.td`; `<name>.<suffix>.mustache` is a second output; the longest matching definition wins.
- [x] The default output is the target's own source root, casing, and extension — `shipping.wire.mustache` → `lib/shipping_wire.dart`. A target now carries `source_root`, so the convention is a language's decision rather than a hard-coded `lib`.
- [x] A leading `{{! dmx output=… target=… }}` comment overrides it, and stays in the template because it renders to nothing. `key=value` rather than JSON: a Mustache comment ends at the first `}` inside it, which no object can survive.
- [x] A `.mustache` file with no definition beside it is left alone, so the catalogue's preview templates and every other project's Mustache stay untouched.
- [x] Editing a template regenerates the definition it is bound to — `.mustache` is watched but never generated *from*, so `--check` cannot report the same drift twice.
- [x] `dmx explain` takes a `.td`, a template bound to one, or a document.
- [x] Diagnostics are located in each origin's own terms: a file by its name and a real line number, a fence by its ordinal and the document line. No fence appears in a message about a file.
- [x] The VS Code extension watches `*.td` alongside `*.dmx.md`, and leaves templates to the binary.
- [x] The golden corpus and the storefront example both generate from standalone files.

### [typediagram.delivery.canonical] One Model Template

A definition with no template beside it used to generate nothing, and every
project that wanted model classes had to write — and then maintain — its own
Mustache. The golden corpus and the storefront example each carried a near-copy
of the same one, and neither produced a *value*: the classes had no `==`, no
`hashCode`, no `copyWith`, and no codec.

There is now exactly one model template, shipped in the binary and used
wherever a diagram generates model classes [typediagram.canonical].

- [x] A definition with nothing beside it renders through the canonical model template; a `<name>.mustache` takes its place; a `<name>.<suffix>.mustache` is still an extra output.
- [x] Records and union cases are immutable values: `==`, `hashCode`, `toString`, `copyWith` — built by the same Rust that builds them for `@dmx('model')`, so the annotated path and the diagram path can never say different things about the same type.
- [x] JSON is on an `extension <Name>Json`, never on the class. `types::Decoders` is what makes a nested decode name the extension, and the annotated path keeps naming the class.
- [x] The runtime import is prefixed, so a diagram that declares its own `Result`, `Ok`, or `Err` — as the parity corpus does — cannot hide the one the codec means.
- [x] A declaration dmx cannot build a codec for keeps its class and its value semantics and gets no extension, and `dmx explain` says which member decided that (`DMX8009`).
- [x] A union case is called what typeDiagram calls it — the case's own name — and takes its union's name as a prefix only where Dart's one namespace forces it [typediagram.canonical.names].
- [x] The golden corpus is the canonical template's gate: every shape typeDiagram can express, regenerated byte-for-byte and run through `dart analyze --fatal-infos`.

### [typediagram.delivery.next] Not Yet Done

- [ ] **tdbin interop.** The names are already aligned: a case generates under the name typeDiagram's own emitters give it [typediagram.canonical.names], so a type dmx generated and a type typeDiagram generated are the same type by name. What that interop needs beyond agreeing names — which artefacts are exchanged, in which direction, and what dmx reads or writes — is not yet written down here.
- [ ] **A union case cannot be a field's type unless its union can be decoded.** A field typed by a *generic* or *untagged* union has no codec, so its owner has none either. Tagged, non-generic unions work; the other two need something in the payload that says which case it is, and the diagram does not say it.
- [ ] **Mustache partials.** `model.mustache` and `diagram_model.mustache` place the same prepared expressions in two layouts — one into a class body somebody else owns, one into a whole file. The expressions are shared in Rust; the *layout* is written twice because ramhorns resolves partials from a folder and dmx's templates are compiled in.

- [ ] **A `.td` grammar for the editor.** The extension ships a Mustache grammar and a Dart injection; a `.td` file gets no language id, no comment toggle, and no highlighting. It is the most visible gap now that definitions are files people open.

- [ ] **Decide what `{{ }}` means for a code generator.** Mustache escapes it as HTML, which is never right for Dart: any value holding `<`, `>`, `&` or `"` — every generic type, every function type — silently becomes uncompilable. `{{{ }}}` is the documented way out and the built-in templates use it, but the default is a trap that fails at the analyzer rather than at the template. Either drop escaping for code targets (`jsoncontent.rs` `render_escaped`, plus the two tests that pin the current behaviour) or make an unescaped-by-default tag the documented norm.
- [ ] **A rule for reading a parent's name inside a child section.** `isOverload` solves one instance of a general trap: any `{{#parentFlag}}…{{childName}}…{{/parentFlag}}` reads the parent's value. Either document the rule where template authors will meet it or push the flags every loop body needs onto the loop's own members.
- [ ] **A second generation target.** The seam is in place and carries one row; the value of the split is unproven until a second language uses it. It is also what would force the questions the Dart-only path never asks: identifier casing per target, reserved words, and how positional members are named somewhere other than Dart.
- [ ] **Reserved-word and identifier diagnostics.** A definition whose field is called `class` or `void` fails today at DMX4001 — "not valid Dart" with a line and column into generated source the author never wrote. It fails safe, which is the important half; it does not yet fail *legibly*, pointing at the definition.
- [ ] **Prove `@targets` exclusion end to end.** `targeting.td` selects nothing away for `dart`, so the corpus shows the filter keeping declarations and never shows it dropping one. A fixture that excludes the target under test would.
- [ ] **`dmx explain --stages` for documents.** `explain` prints groups, dependencies, paths, and the exact context, but not the render → hygiene → validation stages [execution].
- [ ] **A persistent build cache.** Outputs are compared whole, which is correct and re-renders more than a cache would.
- [ ] **Partials in templates.** Every template is self-contained, so two templates over one definition cannot share a fragment — which is why the golden corpus copies one template body per fixture instead of referencing it.
- [ ] **Two definitions claiming one output.** Duplicate outputs are refused within one definition's bindings and within one document, but not across two sources in the same pass: each would take the other's file over and the last pass would win. The ownership marker already records which source wrote a file, so the check has what it needs.
- [ ] **A definition and a document in one package.** Nothing prevents it and nothing tests it. The storefront now shows only the file spelling; the document spelling is proved by `typediagram_cli` and the extension's end-to-end suite instead.

## [typediagram.delivery.acceptance] Acceptance Criteria

- A `.td` file is ordinary typeDiagram and an ordinary typeDiagram Markdown document stays renderable outside dmx; a `.mustache` file is ordinary Mustache.
- `typeDiagram` is resolved by the ordinary built-in macro registry; neither front end creates a second macro engine, a second context shape, or a second ownership protocol.
- Production parsing and resolution run entirely in Rust without typeDiagram tooling or Node.
- A definition is authored once and may feed multiple Mustache outputs without copying the model.
- Mustache, not typeDiagram's language emitter, controls every generated byte.
- Templates receive resolved, target-ready values and contain no type-system logic.
- Invalid definitions, metadata, templates, paths, or Dart fail without changing output.
- `build`, `check`, `watch`, and `explain` agree on groups, context, dependencies, and output paths.
- Every generated file is deterministic, owned, analyzer-clean, below 500 lines, and reproducible from its definition and template.
