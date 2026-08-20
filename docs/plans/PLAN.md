# dmx — Plan

Forward-looking companion to [SPEC.md](../specs/SPEC.md). Identifiers continue the same space as the specification. Nothing here is normative; when an item lands it moves into `SPEC.md` and keeps its identifier.

The backlog is split by topic so each plan stays small and focused.

## Index

| Plan | IDs | Purpose |
|---|---|---|
| [Open questions](open-questions.md) | `[questions]` | Decisions and empirical questions that can block later work |
| [Implementation phases](implementation-phases.md) | `[phases]` | Release sequence and phase exit criteria |
| [Macro catalogue](macro-catalogue.md) | `[catalogue]` | Built-in and user-defined macro backlog |
| [Worked examples](worked-examples.md) | `[corpus]` | Storefront and golden-corpus coverage |
| [typeDiagram integration](typediagram-integration.md) | `[typediagram.delivery]` | typeDiagram Markdown definitions plus Mustache templates to generated Dart — **delivered**; the normative rules are [typediagram] in `SPEC.md`, and what is left is [typediagram.delivery.next] |
| [Semantic front end and static metaprogramming](semantic-metaprogramming.md) | `[semantic-expansion]` | Scope resolution, type inference, typed expansion, and elaborated type-system extensions |

Every section in these files has a unique, hierarchical, non-numeric identifier shared with `SPEC.md`. Adding a plan file does not create a second identifier namespace.
