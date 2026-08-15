# dmx — Architecture

Part of the [dmx specification](SPEC.md).

## [architecture] Architecture

```mermaid
flowchart TD
    source["User Dart source"] --> parse["1. Parse with tree-sitter"]
    parse --> cst["Lossless concrete syntax tree"]
    library["lib tree"] --> index["2. Cached incremental name index"]
    cst --> context["3. Context builder in Rust"]
    index --> context
    semantics["Type dispatch, codec selection, equality strategy, expression building"] -.-> context
    context --> transform["4. Optional transform"]
    transform --> render["5. Render with Mustache or an extension engine"]
    templates["Template directory"] --> render
    render --> dart["Generated Dart text"]
    dart --> hygiene["6. Hygiene"]
    hygiene --> validate{"7. Re-parse and validate"}
    validate -->|valid| emit["8. Emit: part, inline, or augment"]
    validate -->|error| fail["Hard failure; source remains untouched"]
    emit --> cache["9. Cache by Context"]
```

Stages MUST be pure functions of their declared inputs: no clock, no network, no ambient environment. Stages 4 and 5 may run in an external process ([extensions.worker-protocol]) but inherit the same obligation. This is what makes G4 and [execution] caching sound.

Critically, **stages 6–8 are unconditional**. No matter which language produced the text in stage 5, it is hygienized, re-parsed, and rejected if malformed. Extension authors cannot bypass the safety pipeline.

[typediagram] is a built-in macro with a second source path into the same pipeline. For a `*.dmx.md` input, native Rust CommonMark and typeDiagram parsing synthesize the macro invocation in stages 1–3; the bound Markdown Mustache fence supplies stage 5. The path uses the same macro registry and rejoins before hygiene, validation, emission, and caching, so Markdown-authored models receive the same generated-Dart safety guarantees as annotation-authored models.

This diagram specifies the v0.3 pipeline, not its architectural ceiling. The `tree-sitter` CST is the lossless syntax and provenance layer on which [semantic-expansion] intends to add scope and binding resolution, a project semantic graph, constraint solving, inferred types, and typed staged expansion. Those layers do not exist yet. They MUST refine the existing parse → context boundary rather than discard CST fidelity or weaken final full-file validation.

---
