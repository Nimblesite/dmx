# dmx — Diagnostics, Conformance, and Performance

Part of the [dmx specification](SPEC.md).

## [diagnostics] Diagnostics

Every diagnostic MUST carry the full chain: **source span → fragment → template line → generated line**.

```
error[DMX6100]: generated region was modified by hand
  ┌─ lib/models/user.dart:23:3
  │
23│   //#region dmx:generated builtin/model@1.0.0 b3:9f2c4ae1 — DO NOT EDIT
  │   ^^^^^^^^^^^^^^^^^^^^^^^ region body hash does not match header
  │
  = expected b3:9f2c4ae1d38b7c05, found b3:1e77a03bc9d24f18
  = refusing to overwrite. Re-run with --force to discard your edits.
```

| Range | Domain |
|---|---|
| `DMX1xxx` | Invocation, I/O |
| `DMX2xxx` | Parsing, resolution, annotation evaluation |
| `DMX3xxx` | Context building |
| `DMX4xxx` | Rendering, output validation |
| `DMX5xxx` | Hygiene |
| `DMX6xxx` | Emission, backends, regions |
| `DMX7xxx` | Extension layers |
| `DMX8xxx` | typeDiagram Markdown model generation |

`--format=json` emits the same fields machine-readably.

---

## [suite] Conformance Suite

| Suite | Requirement |
|---|---|
| **C1 Parse corpus** | Top 500 pub.dev packages parse with zero `ERROR` nodes. |
| **C2 Golden** | Fixture source → byte-identical expected output, per backend. |
| **C3 Idempotence** | Two `build` runs produce identical output; the second is a full cache hit **and performs zero writes** ([emission.inline-backend.no-op-writes]). |
| **C4 Determinism** | Identical across job counts 1/4/16, two machines, two OSes, randomized hash seeds. |
| **C5 Hygiene fuzz** | Fields named `other`, `json`, `instance`, `e`, `k`, `v`, `hashCode`, `runtimeType`, `toString`, `_`, `$`, `copyWith` compile and behave correctly, or produce `DMX5001`. Run under `inline`, where scope sharing is direct. |
| **C6 Compile** | Every golden output passes `dart analyze --fatal-infos` clean. |
| **C7 Semantic** | `==`/`hashCode` satisfy reflexivity, symmetry, transitivity, `a == b ⇒ a.hashCode == b.hashCode`. Property-tested. |
| **C8 Round-trip** | `fromJson(toJson(x)) == x`. Property-tested. |
| **C9 Engine parity** | Same context via `mustache`, `jinja`, and a worker engine → semantically equivalent, analyze-clean output. |
| **C10 Performance** | Meets [performance]. |
| **C11 Zero-config** | A fresh package with only a dependency and an annotation builds correctly with no config file. |
| **C12 Region safety** | Byte-exactness outside regions ([emission.inline-backend.byte-exactness]); a region gutted by hand is regenerated and damage outside it is refused ([emission.inline-backend.region-recovery]); markers in strings/doc comments/nested classes are not matched ([emission.inline-backend.region-location]); concurrent-modification CAS aborts cleanly ([emission.inline-backend.concurrent-modification]); `git-filter clean` round-trips byte-exactly. |
| **C13 Extension determinism** | `--verify-extensions` passes for all bundled example workers; a deliberately non-deterministic worker is detected and fails with `DMX7003`. |
| **C14 Backend equivalence** | The same model generated under `inline`, `part`, and `augment` produces semantically equivalent behaviour under C7 and C8. |
| **C15 User macro** | A Dart-authored macro in `tool/dmx/macros.dart` is discovered with no config file, expands deterministically under `--verify-extensions`, composes with a built-in on the same declaration, and its output passes C2, C3, and C6 ([dartmacros]). |
| **C16 typeDiagram Markdown** | A real `*.dmx.md` document synthesizes an invocation of the built-in `typeDiagram` macro, binds one definition to one or more Mustache templates, generates byte-identical analyzer-clean Dart through the shared macro pipeline, performs zero writes on a no-change build, reports drift under `--check`, recovers under `watch`, and refuses invalid or unowned output ([typediagram]). |

C11 keeps [authoring] honest; C12 is what makes writing into user files defensible.

---

## [performance] Performance Budgets

Reference corpus: 10,000 files, 1,200 targets, 8-core 2023-class laptop.

| Stage | Budget |
|---|---|
| Index, cold / warm | ≤ 800 ms / ≤ 30 ms |
| Parse throughput, 1 thread | ≥ 20 MB/s |
| Context build, per target | ≤ 150 µs |
| Render, per fragment | ≤ 50 µs |
| Hygiene + validate + format, per target | ≤ 400 µs |
| Region locate + splice, per target (`inline`) | ≤ 100 µs |
| Worker round-trip, per target ([extensions.performance-tiers]) | ≤ 300 µs |
| Worker pool startup, once per build | ≤ 400 ms |
| **Cold, all targets, 8 threads** | **≤ 2.0 s** |
| **Warm, 1 file changed** | **≤ 50 ms** |
| **`check`, no drift** | **≤ 300 ms** |
| **Warm, no changes, `inline`** | **0 file writes** |

`dmx bench` MUST report these and fail CI on regression.

---
