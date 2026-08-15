# dmx — Rendering, Hygiene, and Validation

Part of the [dmx specification](SPEC.md).

## [rendering] Rendering

`mustache` (via `ramhorns`) MUST be supported and is the default; `jinja` (via `minijinja`) SHOULD be. Engine is inferred from file extension, or supplied by a worker ([extensions.worker-protocol]).

Engines MUST be deterministic, MUST NOT perform I/O (partials are resolved by the driver and supplied), and MUST emit span mappings.

Fragments emit in ascending declared order, ties broken lexicographically by id — guaranteeing G4. Generated output MUST NOT be re-scanned for triggers; `--fixpoint` MAY be offered with a ceiling of 4 (`DMX3003` on overrun).

---

## [hygiene] Hygiene

`dmx` cannot rename user bindings (G8), so it unconditionally renames everything it introduces.

1. Every introduced binding MUST be renamed `_$<12-char blake3 base36>_<name>` — deterministic (G4), collision-resistant.
2. Generated top-level declarations MUST be prefixed `_$`; generated mixins `_$<Target>`; generated extensions `$<Target><Role>`.
3. A generated name MUST NOT collide with any name declared on the target or its siblings → `DMX5001`.
4. References to user names (`this.id`, `other.name`) are *uses*, not bindings, and MUST NOT be renamed.
5. The driver SHOULD parse each rendered fragment and compare its actual binders against the declared `introduced` list; undeclared binders produce `DMX5002`. This is the check that keeps foreign engines ([extensions]) honest.

| Hazard | Without hygiene | With [hygiene] |
|---|---|---|
| Field named `other` | `other.other == other` | Parameter becomes `_$k3m9p1x7_other` |
| Field named `json` | `json['json']` shadows the parameter | Parameter renamed |
| Field named `hashCode` | Override collides | `DMX5001` at generation time |
| Field named `instance` | Silent wrong `toJson` | Parameter renamed |

**Under `inline`, hygiene matters more, not less.** Generated members share a scope with user members directly, with no mixin or file boundary between them.

---

## [validation] Validation

After rendering and hygiene, before emission, the driver MUST re-parse the **complete candidate file** — under `inline`, that means the user's file with the region substituted, not the fragment alone.

- Any `ERROR` or `MISSING` node MUST produce `DMX4001` and MUST NOT write.
- The diagnostic MUST report output line/col, originating template and line, fragment id, and the target's source span.
- Emission MUST be atomic: temp file → `fsync` → rename. Never partial output.

This separates `dmx` from string-interpolation codegen, and it is the safety net that makes [extensions] tolerable: a foreign engine can emit anything, and anything malformed is caught here.

---

