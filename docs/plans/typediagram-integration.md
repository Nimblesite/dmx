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

- [ ] Add golden `*.dmx.md` fixtures showing one definition/one template, one definition/several templates, several independent groups, documentation-only diagrams, and ordinary Mustache examples that dmx ignores.
- [ ] Add failing fixtures for malformed JSON metadata, orphan templates, duplicate outputs, invalid typeDiagram, unsafe paths, and unowned destination files.
- [ ] Freeze the JSON fence metadata and adjacency rules from [typediagram.binding].
- [ ] Freeze `typeDiagram` as the built-in macro name and the synthesized invocation shape from [typediagram.macro].
- [ ] Exit: every syntax choice has a fixture and no binding depends on prose text, headings, or document-global ordering.

### [typediagram.delivery.phases.markdown] TD1 — Parse and Bind Markdown

- [ ] Parse Markdown into a CommonMark AST with source spans; do not scan fences with regex.
- [ ] Discover `*.dmx.md` recursively while allowing any Markdown file when explicitly named.
- [ ] Preserve non-generation content and distinguish plain examples from dmx-enabled Mustache fences.
- [ ] Build immutable generation-group values containing the definition span, template span, metadata, and normalized output path.
- [ ] Translate every valid group into one macro invocation without creating a parallel rendering path.
- [ ] Exit: the binding golden suite passes over upstream-compatible backtick fences, longer fences, CRLF, Unicode, interleaved prose, unrelated code blocks, and multiple groups.

### [typediagram.delivery.phases.model] TD2 — Build the typeDiagram Model

- [ ] Implement the documented typeDiagram grammar as a small tokenizing/LL(1) Rust front end, returning `Result` diagnostics with source spans.
- [ ] Resolve declarations, generic parameters, built-ins, and nested references into one immutable model without casts or exceptions.
- [ ] Serialize the resolved model to the same semantic shape as the pinned typeDiagram model JSON.
- [ ] Differential-test every compatibility fixture against typeDiagram's public parser/model JSON in CI so upstream language drift is visible.
- [ ] Reject unknown or unsupported references before Mustache rendering.
- [ ] Exit: the Rust model and typeDiagram oracle agree structurally for the complete corpus, including diagnostics for invalid definitions.

### [typediagram.delivery.phases.context] TD3 — Enrich the Mustache Context

- [ ] Add `src/macros/typediagram.rs` and register it as the built-in `typeDiagram` macro for Markdown generation-group targets.
- [ ] Define and version the root context specified by [typediagram.model].
- [ ] Preserve each declaration exactly once and in source order; add mutually exclusive kind flags instead of duplicating declarations into per-kind lists.
- [ ] Precompute generic declarations, Dart type text, commas, first/last markers, constructor fragments, and every other value required to keep templates logic-free.
- [ ] Reuse the existing Mustache renderer, partial resolver, span mapping, normalizer, and deterministic ordering.
- [ ] Add `dmx explain` snapshots of the exact context for every declaration kind.
- [ ] Exit: a Mustache template generates analyze-clean records and sealed unions without implementing type resolution inside the template.

### [typediagram.delivery.phases.emission] TD4 — Validate and Emit Whole Files

- [ ] Route rendered Dart through hygiene and full-file parse validation before any write.
- [ ] Add ownership headers containing both fence identities and content hashes.
- [ ] Reuse macro-authored-file safety: atomic writes, no-op writes, unowned-file refusal, stale collection, and `--check` drift.
- [ ] Make build cache keys depend on semantic definition content, template content, resolved partials, context version, and dmx version—not unrelated Markdown prose.
- [ ] Extend watch mode to retain the last valid output during invalid edits and recover deterministically.
- [ ] Exit: build/check/watch tests pass without rewriting the Markdown source or an unowned output.

### [typediagram.delivery.phases.product] TD5 — Product Integration

- [ ] Add `DMX8xxx` diagnostics with Markdown, definition, template, and generated-output spans.
- [ ] Add C16 to the conformance suite and enforce byte-identical generation, zero-write second builds, analyzer-clean Dart, and cross-platform paths.
- [ ] Add a worked storefront document that defines models once and generates at least two Dart files from different Mustache templates.
- [ ] Document the feature in the CLI help, README, website, editor highlighting, and VS Code packaging tests.
- [ ] Add editor diagnostics and regeneration for saved `*.dmx.md` documents through the existing engine contract.
- [ ] Exit: the worked document builds from a clean checkout and remains current under the full `make ci` gate.

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

## [typediagram.delivery.acceptance] Acceptance Criteria

- Ordinary typeDiagram Markdown remains valid and renderable outside dmx.
- `typeDiagram` is resolved by the ordinary built-in macro registry; Markdown binding does not create a second macro engine.
- Production parsing and resolution run entirely in Rust without typeDiagram tooling or Node.
- A definition is authored once and may feed multiple Mustache outputs without copying the model.
- Mustache, not typeDiagram's language emitter, controls every generated byte.
- Templates receive resolved, target-ready values and contain no type-system logic.
- Invalid definitions, metadata, templates, paths, or Dart fail without changing output.
- `build`, `check`, `watch`, and `explain` agree on groups, context, dependencies, and output paths.
- Every generated file is deterministic, owned, analyzer-clean, below 500 lines, and reproducible from its Markdown source.
