# dmx — Macro Catalogue

Part of the [dmx implementation plan](PLAN.md).

## [catalogue] Macro Catalogue

P0–P3 shipped one macro against one template, which proved the pipeline and nothing else: every guarantee ([hygiene], [validation], [emission.inline-backend.byte-exactness]) was demonstrated on data classes alone. **PM** generalizes the engine to a *registry* of macros over a *file* of declarations, and populates it — the evidence that "rich context, dumb templates" [authoring.intelligence] is a general architecture and not a data-class trick.

### [catalogue.engine] Engine

- [x] `RawDecl` replaces `RawClass`: classes **and enums**, modifiers (`sealed`, `abstract`, `final`), supertypes, type parameters, methods
- [x] Region location generalized from `class_body` to any member body (`enum_body` included) [emission.inline-backend.region-location]
- [x] Legacy qualified markers (`//#region dmx:generated …`) recognised and migrated to the bare divider on next write
- [x] Macro registry: annotation name → context builder → template; one template file per macro under `src/dmx/templates/`
- [x] Composition: several macros on one declaration emit ordered fragments into the single region [rendering]
- [x] File-level context: a macro may read sibling declarations (a sealed base reads its variants; a fake reads its interface) [frontend.name-index]
- [x] The crate builds: every macro `src/macros/mod.rs` registers has a module, and `make ci` is green end to end
- [x] `src/emit.rs::zones()` shares `region_opener`/`is_region_end` with the front end, so migrating a legacy marker keeps byte-exactness
- [x] Constructor parameter defaults reach the context, so `@dmx('cli')` usage text and `@dmx('route')` query fallbacks state the author's own default rather than inventing one [frontend]
- [x] Parameter annotations reach the context, so `@dmx('query')` and `@dmx('body')` can say how an endpoint's arguments travel [catalogue.macros]
- [ ] The registry accepts a synthesized Markdown generation-group invocation and invocation-supplied Mustache templates for the built-in `typeDiagram` macro [typediagram.macro]
- [ ] `src/frontend.rs` (883 LOC) and `src/macros/cli.rs` (607 LOC) split below 500 LOC

### [catalogue.shipping] Shipping

- [x] `dmx --version`, and a CLI suite over the argument surface itself [cli]
- [x] One version, `Cargo.toml` as its source: a stamper that derives the VSIX manifest from `cargo metadata`, a `--check` mode in ordinary CI, and a tag check that holds the crate, the manifest and the changelog to the tag [release.version]
- [x] `release.yml`: one VSIX per platform built on the architecture it runs on, each proven to carry a binary that reports the released version; a universal bundle proven to carry none; CodeQL-gated publish; rehearsal via `workflow_dispatch` that never publishes [release]
- [x] `pages.yml`: the site published from `main`, downstream of the same `make website` CI runs [playground.hosting]
- [x] Dart highlighting: the divider dmx owns and its annotations in two tiers, with the example and corpus as the drift gate [editor.dart-highlighting]
- [x] A packaging suite over the manifest's promises — files, icon, commands, settings, workspace trust, and every local module the entry point requires [editor.extension.bundle]
- [x] A workspace is watched at every Dart package it holds, not only a `lib` at its root, so a repo keeping packages under `examples/` or `packages/` is generated rather than silently ignored [editor.extension.paths]
- [ ] `win32-arm64` has no bundle: cross-compiling it needs an `aarch64` mingw toolchain that is not in apt. That platform installs the universal bundle and resolves `dmx` from `PATH` until there is one.
- [ ] Open VSX publishing, so the extension is installable in VSCodium/Cursor

### [catalogue.macros] Built-in macros

Each macro lands in three pieces, in this order, because the output is the specification and the builder is an implementation of it:

1. **Example** — the worked file in `examples/storefront/lib`, with the region filled in by hand and Dart tests that run it. This *is* the acceptance criterion: it analyzes clean under `--fatal-infos` and obeys every hygiene rule.
2. **Template** — `src/dmx/templates/<macro>.mustache`, the shape of that output with the varying parts named.
3. **Builder** — `src/macros/<macro>.rs`, the Rust that computes what the template names, until `dmx build` reproduces the example byte-for-byte.

`typeDiagram` is a built-in macro even though it is not written as `@dmx('typeDiagram')`: a Markdown definition/template group synthesizes its invocation [typediagram.macro]. Its template is supplied by the bound Mustache fence rather than the built-in template directory; its example remains the acceptance criterion and its Rust builder still owns all context intelligence.

| Macro | Target | Generates | Example | Template | Builder |
|---|---|---|---|---|---|
| `typeDiagram` | Markdown generation group | whole Dart model files shaped by bound Mustache templates | [ ] | Bound fence | [ ] |
| `@dmx('model')` | class | JSON codec, `==`/`hashCode`, `toString`, `copyWith` | [x] | [x] | [x] |
| `@dmx('union')` | sealed class | `when`/`maybeWhen`, `isX`, `asX`, discriminated JSON | [x] | [x] | [x] |
| `@dmx('enum')` | enum | wire codec, `label`, `tryParse`, predicates, `unknown:` fallback | [x] | [x] | [x] |
| `@dmx('diff')` | class | field-level structural diff against another instance | [x] | [x] | [x] |
| `@dmx('lerp')` | class | `lerp` over a design-token set (Flutter `ThemeExtension`) | [x] | [x] | [x] |
| `@dmx('validate')` | class | accumulating `Result` validation from `@dmx('check.*')` constraints | [x] | [x] | [x] |
| `@dmx('table')` | class | SQL DDL, indexes, column constants, `toRow`/`fromRow`, upsert | [x] | [x] | [x] |
| `@dmx('route')` | class | path/query codec, `location`, `parse` | [x] | [x] | [x] |
| `@dmx('router')` | sealed class | one matcher over every sibling `@dmx('route')` | [x] | [x] | [ ] |
| `@dmx('cli')` | class | `argv` parser, `usage` text | [x] | [x] | [x] |
| `@dmx('fake')` | class | deterministic seeded fixtures and fixture lists | [x] | [x] | [x] |
| `@dmx('restClient')` | class | HTTP method implementations for a sibling interface | [x] | [x] | [x] |
| `@dmx('event')` | class | analytics event name and flat parameter map | [x] | [x] | [ ] |
| `@dmx('prefs')` | class | namespaced keys and a total read over a key-value store | [x] | [x] | [ ] |
| `@dmx('strings')` | class | one formatter per message, placeholders typed, plurals | [x] | [x] | [ ] |

### [catalogue.runtime] Runtime support

- [x] Leaf decoders as `DmxDecode<T>` values: `dmxString`, `dmxInt`, `dmxDouble`, `dmxBool`, `dmxDateTime`, `dmxUri`, `dmxDuration`, `dmxAny`
- [x] `dmxKey` — read a key from an `Object?` without a cast, so a nullable field decodes without a second `switch`
- [x] `Result`, `DecodeError` are value types (`==`/`hashCode`), so generated output can be compared in a test without unwrapping first
- [x] `DmxChange`, `Violation`, `RouteMismatch`, `UsageError` — the small value types the macros return instead of throwing
- [x] `dmxLocation` / `dmxQuery` — URL assembly without a stray `?`
- [x] `dmxScanArguments` — the recursive argv scanner, taking its tables as arguments so two commands in one file cannot share one
- [x] `dmxLerpDouble` / `dmxLerpInt` / `dmxLerpDuration` / `dmxLerpStep`
- [x] `DmxTransport` / `DmxRequest` / `DmxResponse` / `ApiError` — the seam a generated REST client is written against, testable without a mock library

### [catalogue.dartmacros] User-defined macros

[dartmacros] end to end turns the catalogue from a fixed list into a floor:

- [x] `src/dart_packages/dmx` package: `DmxMacro`, `DmxInvocation`, `DmxOutput` (fragment/refuse — a refusal is a value, never a throw), and `dmxServeMacros` speaking the worker protocol so authors never see a frame
- [x] `expand` op in the Rust driver: handshake `macros:[…]`, per-target dispatch, refusals and diagnostics surfaced as `DMX2100` [dartmacros.protocol]
- [x] Discovery of `tool/dmx/macros.dart` by convention, walking up from the file being generated with the working directory as fallback; one worker per worker file, spawned only when the file is there, so a workspace of several packages generates every one of them [dartmacros.discovery]
- [x] Resolution: built-ins win with `DMX7005` on collision, `DMX7006` on duplicate user names; an unserved name stays inert [dartmacros.resolution]
- [x] User fragments through the same normalizer and inline emission as a built-in's [dartmacros.pipeline]
- [x] **Macro-authored files** [dartmacros.files]: an `expand` reply carries `files`, whole sibling Dart files named by the macro — driver-owned marker line, never overwrite an unmarked file (`DMX7008`), bare-`.dart`-name validation (`DMX7007`), stale collection when the source of truth drops a table, `--check` drift, atomic no-op-aware writes
- [x] Macro-authored files kept current under `watch`: editing one re-runs the seed its marker names, deleting one writes it again, and a pass where no macro ran collects nothing — a missing worker is never read as a dropped table [dartmacros.files], [execution.modes]
- [x] Example: `examples/dmx_sqlite_example` — ONE annotated seed class, and the macro reads the LIVE SQLite schema and authors one file per table and view (fields, constructor, keys, foreign-key lookups, SELECT/INSERT, `toRow`/`fromRow`), named after the tables; nobody types a table name anywhere. Dart tests over the generated code, gated in CI by `make example-sqlite`
- [ ] **Universal invocation context**: `RawDecl` reflected whole — declaration, fields with per-field annotation args, sibling file, `@dmx` args as raw source — serialized once and shared by `dmx explain`, the `expand` op, and transforms [dartmacros.api]
- [ ] `dmx explain <file>` printing the exact `DmxInvocation` JSON per declaration; the discovery path a macro author starts from
- [ ] `introduced`/`spans` obligations enforced against the fragment a worker returns, rather than taken on trust [dartmacros.protocol]
- [ ] AOT compile of the worker, cached by source hash under `.dart_tool/dmx/` [dartmacros.discovery]
- [ ] `DMX2013` nearest-name warning on an unknown `@dmx` [dartmacros.resolution]
- [ ] User fragments through the hygiene and validation stages too; region headers `user/<name>@<version>` [dartmacros.pipeline]
- [ ] A worked file composing a user macro with a built-in on the same declaration
- [ ] C15 in the conformance suite; `--verify-extensions` covering macro workers

### [catalogue.stretch] Stretch

- [ ] Generic models: `typeParams` in the context, decoder/encoder parameters per type argument (`Page<Product>`) [context.root]
