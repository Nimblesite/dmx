<!-- agent-pmo:a72c926 -->
# dmx — Agent Instructions

⚠️ **TOKEN DISCIPLINE.** Check file size first. `Grep` over `Read`. Use `offset`/`limit`. Smallest diff that solves the problem. Delete dead code, unused imports, stale comments. Call out irrelevant context before proceeding. Bloat degrades reasoning. ⚠️

⚠️ **ACT AUTONOMOUSLY. DO NOT STOP TO ASK QUESTIONS.** When something is ambiguous, pick the most reasonable default, note the assumption, and work to completion. Deliver finished work plus a short list of assumptions. ⚠️

## What dmx Is

**Fast, reliable Dart code generation on every save. No generated `part` files. Fully customizable.**

Annotate a Dart class, save, and the generated members appear **inside the same file**, below a divider — no `part` directive, no `.g.dart`, no mixin, no delegating factory. The VS Code extension runs the watcher; other editors run `dmx watch` once. Built-ins cover the familiar Freezed / dart_mappable jobs; Mustache templates let teams reshape the output; custom Dart macros generate members or whole files from any source of truth.

Pipeline: **parse → context → render → validate → emit inline.** tree-sitter reads the Dart CST, a Rust context builder finishes every decode/encode/equals/hash/copy expression, Mustache renders dumb strings, the emitter splices between the dividers byte-exactly.

**Languages:** Rust (the CLI). Dart is the *output* plus the `src/dart_packages/dmx` runtime.

## Messaging — [docs/messaging.md](docs/messaging.md) is the ONLY authority

Any user-facing words — README, website, docs, blog, release notes, CLI help, extension copy, PR descriptions — must match [docs/messaging.md](docs/messaging.md). Read it before writing prose.

- **Never restate the pitch from memory.** Reuse its ready-to-use copy verbatim where one fits.
- **Obey its Guardrails section literally.** No numeric speed claims without a reproducible benchmark; no "drop-in Freezed replacement"; no compiler jargon in the lead.
- **Lead with the workflow:** *save and keep coding*, *no `build_runner` dance*, *no generated `part` files*, *one complete Dart file*.
- Changing the message means editing `docs/messaging.md` in the same commit as the copy quoting it.

# HARD RULES — ALL LANGUAGES, NO EXCEPTIONS

- ZERO DUPLICATION. Run Deslop often.
- ZERO DEAD CODE. Check often.
- THROWING EXCEPTIONS IS ⛔️ ILLEGAL.
- FP STYLE ALL THE WAY: `Result<T,E>`, immutable.
- Pattern matching only. NO CASTING.
- FILES OVER 500 LOC ARE ILLEGAL.
- NO REGEX on structured data — real parsers for JSON/YAML/TOML/Dart.
- NO PLACEHOLDERS that silently no-op. No linter suppressions — fix the code.
- **GENERATED CODE OBEYS EVERY RULE ABOVE.** No `throw`, no `as`, no `!`.
- **PROPER NAMES** in generated code: `json`, `other`, `value` — never hash-mangled. Suffix only on a real collision.
- **TYPEDIAGRAM MACRO** [typediagram]: the built-in `typeDiagram` macro parses Markdown and the typeDiagram DSL natively in Rust. typeDiagram definitions are the model source; bound Mustache templates are the sole authority for generated code shape. Never invoke typeDiagram's language emitters in the production path.

## Hard Rules — Rust

- No `unwrap()`/`expect()` in production (tests may `expect`). No `panic!`/`todo!`/`unimplemented!`/`unreachable!`.
- No `unsafe {}` or `allow(clippy::...)` without documented justification.
- Every public item has `///` docs; every module opens with `//!` citing the spec ID it implements.
- `thiserror` for library errors; `anyhow` only in the CLI binary.
- Lints live in `Cargo.toml` `[lints.rust]` / `[lints.clippy]` — every rule on, at ERROR. [LINT-RUST]

## Hard Rules — Dart (emitted code and `src/dart_packages/dmx`)

- No `late`, `!`, `dynamic`, `as` casts (use `is` + smart casts), `.then()` (use `async`/`await`), or `throw`.
- Generated Dart must satisfy `dart analyze --fatal-infos` **and** every rule above.
- `examples/storefront/lib` and `src/dmx/tests/golden/*.dart` are generator OUTPUT. Never hand-edit between the dividers — change the template or context builder and regenerate (`make golden`, `make example`).

## Logging

- `tracing` + `tracing-subscriber` only. Never `println!`/`eprintln!`/`dbg!` for internal state. The CLI's stdout/stderr (progress lines, `DMX####` diagnostics) is the **output contract**, not logging — leave it alone; CLI tests assert on it.
- Log entry/exit of significant operations (`error|warn|info|debug|trace`). Silent failures are forbidden.
- Structured fields, never interpolation: `{ path: "…", generation: 4 }`.
- NEVER log PII or secrets. Log `"key: present"` or a truncated hash.

# Bugs / Correctness

When fixing a bug, you MUST write a failing test that fails because of the bug before moving forward. Follow this [text](.agents/skills/fix-bug/SKILL.md)

Placeholder code is always illegal. If you encounter code that produces incorrect results or is not complete:

- Immediately DELETE the offending code
- Replace it with a panic or exception
- Report the problem to the user explaining why the code is wrong
- 🛑 STOP 

# Testing

- LOAD THIS REPO UP WITH E2E TESTS. NO MOCKS. Cover the corner cases.
- MANY user interactions per test; MANY assertions per user interaction; MANY input permutations per test
- **E2E tests are black-box:** drive the real `dmx` binary over real files; never reach into internals.
- **Golden corpus is a hard gate:** every `src/dmx/tests/golden/*.dart` sample regenerates byte-identically AND survives `dart analyze --fatal-infos`. Emitting Dart that does not compile is the worst failure this repo can have.
- **Never delete a failing test** — fix the code or the expectation. **Never skip** without a ticket number AND expiry date in the reason.
- **`make test` is FAIL-FAST** (never `--no-fail-fast`) [TEST-RULES] **and always computes AND enforces coverage.** Threshold lives in `coverage-thresholds.json` at the repo root — not env vars, not gh variables, not CI YAML. Below threshold = pipeline fails. Ratchets UP only. [COVERAGE-THRESHOLDS-JSON]
- Meaningful assertions only (`assert!(true)` is illegal). No `catch`/`Result`-swallowing that asserts success anyway.
- Deterministic: no `sleep()`, no timing dependencies, no random state.

# Duplication — Deslop (MANDATORY)

[CI-DESLOP] · [docs](https://deslop.live/docs/for-ai/) · [issues](https://github.com/Nimblesite/Deslop/issues). **Prevention, not cleanup.**

**BEFORE authoring** any function, struct, helper, fixture, or test setup → **`find-similar`**:

| `signals.fused` | Action |
| --- | --- |
| ≥ 0.85, or `identical` / `nearly_identical` | **REUSE. Do not duplicate.** |
| 0.6 – 0.85 | Open the canonical occurrence; bias hard toward extending it. |
| < 0.6 or empty | Proceed. |

**AFTER changing code** → **`rescan`**, then **`top-offenders`** / **`cluster-by-id`** before merging a cluster.

**NEVER game the gate** — no widening `max_duplication_percent`, no `hidden`, no trivially-different shapes. Budget lives in `.deslop.toml`; CI runs `deslop .` and **TANKS** (exit 3) over budget. Ratchet **DOWN only**, in the PR that reduces duplication.

# Documentation

**Every section of [docs/specs/SPEC.md](docs/specs/SPEC.md) and [docs/plans/PLAN.md](docs/plans/PLAN.md) carries a bracketed dotted path** — `[emission.inline-backend.region-location]`. Unique, hierarchical (a parent is a literal prefix of its children), non-numeric, so inserting a requirement never renumbers siblings. One shared ID space; IDs are never reused.

- **Code** cites what it implements: `//! Region location [emission.inline-backend.region-location].`
- **Tests** cite what they verify: `/// [model.copywith]: DmxTo(null) clears.`
- **Diagnostics** cite what they enforce, in the message text.
- **Specs** cite IDs, never `§11.3.5`. Numeric section refs are dead.
- Changing behaviour means changing that ID's section in the same commit.

`grep -r 'emission.inline-backend'` returns the whole subtree — spec, code, tests. A spec section with no citing code and no citing test is unimplemented; that is the point.

**The plans are your backlog. Work on them.** typeDiagram for data models, mermaid for everything else. Remove line endings that exist only to force wrapping — let the editor wrap.

## Website

- 2K LOC TOTAL CSS budget.
- All document pages (docs, blog posts) use the SAME prose CSS classes.

## Git — DO NOT TOUCH IT

**DEFAULT: run no git command at all.** No commit, branch, push, or merge. Leave all of it to the user and CI. Finish the work, leave it in the working tree, and say so. Only act on git when the user has explicitly green-lit it *for this task* — then:

- **NEVER push to `main`.** Always PR → CI green → merge.
- **Own every PR until it's green.** `gh pr merge --auto --squash`, then monitor: on failure pull logs, fix, push, loop. Never hand back a red or still-running PR.
- **NEVER list yourself as co-author.** No `Co-Authored-By`, no agent attribution. Never overridable.
- **Exactly ONE branch**, even with concurrent agents. Reuse the open feature branch; never cut your own. If several exist, merge them into one IMMEDIATELY.
- **Worktrees are forbidden** unless explicitly directed.
- **Auto-memory is OFF.** Persistent rules go through a reviewed PR to this file (`"autoMemoryEnabled": false` in committed `.claude/settings.local.json`).

## Too Many Cooks (Multi-Agent)

If the TMC server is available: register on start (name, intent, files), lock files before editing, broadcast your plan, check messages periodically, release locks when done. Never edit a locked file — wait or take another approach.

## Build

Cross-platform GNU Make. Canonical targets `fmt`, `lint`, `test` never overlap; repo-specific targets sit below them. To debug a single test, call `cargo test` directly — that is not a Makefile target.

## Repo Structure

**Every line of code lives under `src/`.** The repository root carries only the
things their own tooling pins there: `Makefile`, `README.md`, `LICENSE`,
`AGENTS.md`, `coverage-thresholds.json`, `docs/`, `examples/`, `scripts/`,
`website/`. There is no `Cargo.toml` at the root — the crate is self-contained,
so **every cargo call names its manifest**: `cargo <subcommand> --manifest-path
src/dmx/Cargo.toml`, or just use the Makefile, which does it for you.

```
src/
  dmx/                  the Rust crate, self-contained [repo.layout]
    Cargo.toml          NOT at the repo root — cargo needs --manifest-path
    Cargo.lock          committed; dmx ships as a binary
    rustfmt.toml
    src/
      main.rs           CLI entry point [cli]
      lib.rs            pipeline wiring: parse → context → render → validate → emit
      frontend.rs       stage 1 — tree-sitter Dart front end
      context.rs        stage 3 — context builder [context], [model]
      render.rs         stage 5 — Mustache render + whitespace normalizer
      emit.rs           stage 8 — inline emission [emission.inline-backend]
      engine.rs         live generation state, lspkit engine contract [engine]
      watch.rs          debounced incremental watching [execution.modes]
      types.rs          Dart types + JSON codec table [model.json-codec]
      casing.rs         identifier casing helpers [context.helpers]
      macros/           the registry: one module per built-in [catalogue]
    templates/          the SHIPPED templates, compiled in with include_str!
    tests/              e2e, golden, recovery, watch-CLI, editor-hookup suites
      golden/           hand-written Dart samples + expected output
  dart_packages/dmx/    hand-written annotations, runtime, macro-authoring API
  editors/vscode/       the VS Code extension [editor.extension]
examples/storefront/    worked example; examples/storefront/lib is GENERATED
  templates/            catalogue PREVIEWS — no context builder, generate nothing
docs/messaging.md       sole authority for user-facing copy
docs/specs/             normative specs, indexed by SPEC.md
docs/plans/             forward-looking plans, indexed by PLAN.md, same ID space
```

**`src/dmx/templates/` is the product; `examples/storefront/templates/` is not.**
A file in the first is compiled into the binary by an `include_str!` in
`src/dmx/src/macros/`, and `@dmx('<name>')` generates from it. A file in the
second has a design and an example but no registered context builder, so the
annotation expands to nothing and leaves the file untouched. Moving one across
means adding its builder to `REGISTRY` in the same commit.
