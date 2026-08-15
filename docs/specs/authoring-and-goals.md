# dmx — Authoring, Conformance, and Goals

Part of the [dmx specification](SPEC.md).

## [authoring] The Authoring Surface

Normative, and constrains every other section.

> **A consumer chooses one authoring path: Dart source with annotations, or a `*.dmx.md` document containing typeDiagram definitions and bound Mustache templates ([typediagram]). Neither path requires configuration or scripts.**

### [authoring.roles] The roles

| Role | Writes | Must understand | Present by default |
|---|---|---|---|
| **Consumer** (99%) | Dart + annotations | The annotations | Always |
| **Model-first consumer** | `*.dmx.md`: typeDiagram + Mustache | typeDiagram definitions and [typediagram.model] context | Only when generating models from a shared definition |
| **Template author** | + `.mustache` files | Context variables ([context]), learned via `dmx explain` | Only if overriding |
| **Macro author** | + `tool/dmx/macros.dart` — **Dart** | The invocation model ([dartmacros.api]), learned via `dmx explain` | Only if defining macros |
| **Extension author** | + a transform or engine in any language | The worker protocol ([extensions.worker-protocol]) | **Never** — fully opt-in |
| **Built-in author** (us) | Rust | Everything | N/A |

[typediagram] is an alternative consumer surface, not a requirement on the Dart-first path. [dartmacros] and [extensions] serve macro and extension authors without imposing anything on either consumer path. If a capability forces a consumer upward, it belongs in Rust instead. A macro author writes Dart — the language they already know — and nothing else: no Rust, no fork, no configuration.

### [authoring.intelligence] Where the intelligence lives

The hard problems — dispatching a decoder on `List<Map<String, Address?>>`, deciding when a field needs `DeepCollectionEquality`, distinguishing "omitted" from "explicitly null" — are solved **in Rust**, once. The result reaches the template as a finished string:

```mustache
{{#fields}}
      {{jsonKey}}: {{decodeExpr}},
{{/fields}}
```

Rich context, dumb templates. This is what makes "Dart only" achievable for consumers, and it is why the context matters even though nobody writes it.

The same rule governs [typediagram]: typeDiagram supplies model facts, Rust resolves them and computes Dart-ready values, and Mustache controls only the output shape.

### [authoring.real-metaprogramming] "Real metaprogramming like Rust"

In Rust, `#[derive(Serialize)]` requires the *consumer* to write zero Rust; only the derive *author* writes arbitrary compile-time code. `dmx` keeps that split and extends it: arbitrary computation is available at four levels of escalation, each optional. Level 2 is what makes this *real* metaprogramming rather than a fixed catalogue: a project defines its own `@dmx('name')` in Dart, and it is indistinguishable at the use site from a built-in.

| Level | Who | Mechanism | Reach for it when |
|---|---|---|---|
| 0 | Consumer | Annotations | Always sufficient for built-ins |
| 0M | Model-first consumer | typeDiagram definitions + bound Mustache templates ([typediagram]) | A model definition should generate one or more Dart files |
| 1 | Template author | Swap a template directory | Layout/style differs |
| 2 | **Macro author** | **A new `@dmx('name')` written in Dart ([dartmacros])** | The catalogue lacks the macro |
| 3 | Extension author | Transform or engine in any language ([extensions]) | Rewriting how *all* macros expand |
| 4 | Built-in author | Rust | Building `dmx` itself |

---

## [conformance] Conformance Language

RFC 2119 keywords. A conforming implementation MUST satisfy every normative topic indexed by [SPEC.md](SPEC.md) and pass [suite].

---

## [goals] Goals, Non-Goals, Position

### [goals.objectives] Goals

| # | Goal |
|---|---|
| G1 | Generate Dart out-of-band from Dart annotations or typeDiagram Markdown, with no Dart runtime required in the default path. |
| G2 | Zero-config by default. |
| G3 | Consumer surfaces are Dart annotations and the opt-in typeDiagram-plus-Mustache model path; everything beyond is opt-in. |
| G4 | Deterministic: identical inputs → byte-identical output. |
| G5 | Fast enough that regenerating everything on every build is reasonable. |
| G6 | Never emit unparseable Dart; malformed output fails the build pointing at its origin. |
| G7 | Hygienic. |
| G8 | Additive only: never modify or delete user-authored code. |
| G9 | Forward-compatible with augmentations without a rewrite. |
| G10 | Zero required boilerplate in the default backend. |
| G11 | Let one typeDiagram definition feed several Mustache-controlled Dart outputs without duplicating the model. |
| G12 | Build progressively from the lossless CST and source provenance into a semantic graph, full type inference, and typed static metaprogramming ([semantic-expansion]). |

### [goals.non-goals] v0.3 Non-Goals

These are architectural exclusions. Planned capabilities such as full type inference and typed static metaprogramming are goals, so they do not belong in this table.

| # | Non-Goal |
|---|---|
| N1 | A required configuration language or scripting language. |
| N2 | Runtime metaprogramming. |
| N3 | Being a build system. |
| N4 | Modifying user-authored code — with the single, bounded exception of the generated region in [emission.inline-backend], which is machine-owned territory inside a user file. |

### [goals.architectural-position] Architectural position

Dart's macro effort died because expansion sat **inside** the compiler's incremental pipeline, on the critical path of analysis, completion, and hot reload. `dmx` expands **out-of-band** into ordinary source on disk. The analyzer sees plain Dart written before it started. That latency budget does not apply, because no macro runs in the compiler. Cost becomes schedulable ([execution]) and small ([performance]), and output stays reviewable and debuggable.

### [goals.capability-parity] Capability parity with the abandoned proposal

| Macro capability | Dart proposal | `dmx` | Notes |
|---|---|---|---|
| **Macros authored by users, in Dart** | ✓ — the entire point | ✓ | [dartmacros]: `tool/dmx/macros.dart`, zero config |
| Introspect fields, types, annotations | ✓ | ✓ | Syntax + name index, not full inference ([frontend.no-type-inference]) |
| Add members to an existing class | ✓ | ✓ | `inline` ([emission.inline-backend]) or `augment` |
| Add top-level declarations | ✓ | ✓ | All backends |
| Arbitrary computation at generation time | ✓ | ✓ | Dart ([dartmacros]), or [extensions] in any language |
| Hygiene | ✓ | ✓ | [hygiene] |
| Typed AST construction | ✓ | Partial | Text + mandatory re-parse ([validation]) rather than a typed builder |
| Modify existing method bodies | ✓ (early) | ✗ | Dart also dropped this; `augmented()` was removed from the spec |
| Full semantic type resolution | ✓ | **Planned** | Central objective of the incremental semantic layer over the CST; not implemented yet ([semantic-expansion]) |
| IDE latency impact | Severe — the reason it died | None | Out-of-band; the analyzer sees finished Dart |
| Cost paid | Every analysis, every build | When the consumer schedules it | [execution.modes] |

Full semantic type resolution is not implemented in v0.3, but it is a central architectural objective. dmx is intended to become a full-fledged static metaprogramming system with compiler-grade semantic understanding, type inference, typed elaboration, and hygienic staged expansion while generated output remains ordinary, analyzer-valid Dart ([semantic-expansion]). Body rewriting is different: it remains outside the additive-only contract.

---
