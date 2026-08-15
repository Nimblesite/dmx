# dmx — Specification v0.3

Normative specification. Every section carries a stable, human-readable identifier: a dotted path of words such as `[emission]`, `[emission.inline-backend]`, `[emission.inline-backend.region-location]`. Identifiers are unique, hierarchical, and non-numeric, so inserting a requirement never renumbers its siblings and `rg 'emission.inline-backend'` finds the whole requirement subtree.

Implementation code, tests, and diagnostics MUST cite the identifier they satisfy. Forward-looking work is indexed by [PLAN.md](../plans/PLAN.md). Specification and plan files share one identifier space; an identifier is never reused.

## Index

| File | Top-level IDs | Scope |
|---|---|---|
| [Authoring, conformance, and goals](authoring-and-goals.md) | `[authoring]`, `[conformance]`, `[goals]` | Roles, authoring contract, conformance language, goals and non-goals |
| [Repository layout](repository.md) | `[repo]` | Where code lives, and holding the repository's own tooling to it |
| [Architecture](architecture.md) | `[architecture]` | Parse-to-cache pipeline and purity boundary |
| [typeDiagram Markdown macro](typediagram-markdown.md) | `[typediagram]` | Built-in macro: typeDiagram definitions plus Mustache templates to generated Dart |
| [Consumer surface and front end](consumer-surface.md) | `[surface]`, `[context]`, `[frontend]` | Annotations, template context, parsing and name resolution |
| [Extension layers and Dart macros](extensions.md) | `[extensions]`, `[dartmacros]` | Workers, transforms, engines, and user-defined macros |
| [Rendering, hygiene, and validation](rendering-and-validation.md) | `[rendering]`, `[hygiene]`, `[validation]` | Template execution and generated-source safety |
| [Emission and backends](emission.md) | `[emission]` | Part, inline, and augmentation output |
| [Execution, CLI, and engine](execution.md) | `[execution]`, `[cli]`, `[engine]` | Modes, caching, commands, and live engine contract |
| [Editor integration](editor.md) | `[editor]` | VS Code packaging, startup, E2E, and highlighting |
| [Built-in model macro](model-macro.md) | `[model]` | `copyWith`, equality, JSON codecs, and deferred work |
| [Diagnostics, conformance, and performance](conformance.md) | `[diagnostics]`, `[suite]`, `[performance]` | Diagnostics contract, acceptance suites, and budgets |
| [Playground and release](playground-and-release.md) | `[playground]`, `[release]` | Browser playground, hosting, release, and versioning |

Each topic file links back here. Moving a section between files never changes its identifier.
