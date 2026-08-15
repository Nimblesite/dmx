# dmx — Execution, CLI, and Engine

Part of the [dmx specification](SPEC.md).

## [execution] Execution Modes and Caching

### [execution.modes] Modes

| Mode | Behaviour | Use |
|---|---|---|
| `incremental` | Default; re-run only where context changed | Development |
| `always` | Ignore cache | Release, debugging |
| `once` | Generate; never re-run automatically | Committed output |
| `check` | Generate to memory, diff against disk, exit non-zero on drift; never writes | CI |
| `watch` | Debounced watcher, incremental | Active development |

The consumer chooses when cost is paid. Under `inline` with Strategy B ([emission.inline-backend.version-control]), CI MUST run `dmx build` after checkout before `check` or `analyze`, since the checked-out tree has empty regions.

`watch` answers what happened on disk, not what an editor meant by it. One save arrives as a burst of events and MUST be answered once, a debounce window after the last of them. Every path the burst named is judged when that window closes rather than when its event arrived: a rename — an editor saving, or dmx's own atomic write — passes through a moment where the destination does not exist, and judging on arrival would read a file as deleted on every save. A path still missing when the window closes was deleted, and the watched directory it was in is what gets re-read; a generated file names its seed nowhere any more, and re-running that seed is what writes it again [dartmacros.files].

### [execution.cache-key] Cache key

```
blake3( tool_version ‖ macro_version ‖ template_digest ‖ engine_id
      ‖ extension_digests ‖ canonical(context) ‖ backend )
```

Keyed on **context**, not source bytes. Reformatting, editing an unrelated method body, or changing a comment does not change the context and does not regenerate. This is the largest incrementality win over `build_runner`, which invalidates file-wise and barrel-transitively.

`extension_digests` covers every transform and engine binary or script ([extensions.obligations]), so changing a worker invalidates correctly.

### [execution.store] Store

Content-addressed under `.dart_tool/dmx/cache/`, safe to delete, concurrency-safe by atomic rename. An index maps `(source, target) → key`.

**Orphan collection** differs by backend: under `part`, delete generated files whose target no longer exists. Under `inline`, empty and remove regions whose target class was renamed or deleted — and because that edits a user file, it MUST be reported and MUST be gated behind `dmx fix` or `--prune`, never implicit.

### [execution.parallelism] Parallelism

Targets MUST be processed in parallel. Determinism holds because per-target output depends only on that target's context. Under `inline`, targets sharing a file MUST be batched into a single read-modify-write to satisfy [emission.inline-backend.byte-exactness] and [emission.inline-backend.concurrent-modification].

---

## [cli] CLI

```
dmx build   [PATHS...] [--mode=…] [--backend=inline|part|augment]
                       [--insert-regions] [--force] [--prune] [--jobs=N]
dmx check   [PATHS...] [--boilerplate]
dmx watch   [PATHS...]
dmx explain <FILE> [--target=NAME] [--stages]
dmx fix     [PATHS...] [--dry-run]
dmx git-filter clean
dmx clean
dmx bench
```

`PATHS` may name Dart sources, recursively discovered `*.dmx.md` documents, or any Markdown document passed explicitly for [typediagram]. Markdown generation writes only its declared owned outputs; it never rewrites the document.

`dmx explain` prints the context as readable key/value output; `--stages` adds raw render → post-hygiene → post-validation → final region. It is the template author's only tool and MUST have excellent output.

`dmx fix` is the sole path by which `dmx` edits user code: inserting regions, inserting `part` directives or `with _$X` under the `part` backend, and pruning orphans. It MUST support `--dry-run`, MUST make no other edit, and MUST NOT run as part of `build`.

Exit codes: `0` ok · `1` errors · `2` drift · `3` bad invocation · `4` internal.

---

## [engine] Engine

Nothing calls the pipeline directly. Live generation state is reached through one contract — `lspkit::EngineApi` — so the process that watches a folder, a language server answering an editor, and an MCP adapter answering an agent are three consumers of the same state rather than three implementations of it.

`dmx watch` is the first consumer. That is deliberate: a contract with one consumer is a guess, and the watcher is the consumer whose requirements (incremental scopes, per-file failure, a readable account of what changed) are the same ones a language server has.

### [engine.api] The EngineApi contract

| Member | Obligation |
|---|---|
| `generation()` | Monotonic. Two successful rescans produce two ordered generations |
| `rescan(scope, progress)` | Regenerate a scope; return the ticket naming the generation it produced |
| `report(query, cancel)` | A generation-tagged snapshot; MUST observe cancellation within a bounded delay |
| `subscribe()` | Stream of generation-change events |
| `shutdown()` | Drain, complete every subscriber stream; every later call MUST return an error |

Normative:

1. A rescan MUST NOT fail because one file did. A source that does not parse is recorded as a refused outcome carrying its diagnostic, and its neighbours are generated anyway. Only a failure to *enumerate* a scope fails the pass.
2. Every pass MUST record the files it examined, including the ones it left alone [emission.inline-backend.no-op-writes]. A report that lists only changes cannot distinguish "nothing to do" from "never looked".
3. A consumer detects staleness by comparing the `generation` of a snapshot against the ticket it holds. An engine MUST NOT return a snapshot tagged with a generation it has not reached.
4. A scope this implementation does not recognise MUST degrade to a full rescan — always correct, only slower — never to no rescan.

---
