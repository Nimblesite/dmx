# dmx — Repository Layout

Part of the [dmx specification](SPEC.md).

## [repo] Repository

### [repo.layout] Every line of code lives under `src/`

The repository root carries only what its own tooling pins there: the Makefile, the licence, the agent instructions, the coverage thresholds, and the `docs/`, `examples/`, `scripts/` and `website/` directories. Everything else — the Rust crate, the Dart runtime package, the editor extension — lives under `src/`.

The crate is self-contained at `src/dmx`, which means **the repository root carries no `Cargo.toml`**. Every cargo invocation MUST name its manifest, and the knowledge of where that manifest is MUST live in exactly one place per language: the Makefile for `make`, `scripts/version.mjs` for the release workflow. A second copy of the path is a second place to forget it.

### [repo.layout.references] Configuration that names a path is proven against the tree

A workflow, a dependabot entry and an editor task are strings claiming a file is somewhere. Nothing compiles them and nothing resolves them until they run, so moving a directory leaves every one of them naming a path that is gone while the whole suite stays green.

Where they surface is what makes them normative rather than tidy. A stale manifest path stops a release in its first job, after the tag is pushed and cannot be taken back. A stale dependabot directory fails **silently** — no pull request, no error, no annotation, just an ecosystem that quietly stops being updated. A stale trigger filter does not fail the workflow; it stops the workflow from running at all.

Therefore, for every path the repository's own configuration names, a test MUST prove the tree carries it:

- every dependabot directory exists and holds the manifest its ecosystem is watched through;
- every path filter a workflow triggers on matches something, taking the literal part before the first wildcard;
- every local workflow a job calls, and every `working-directory` a job runs in, exists;
- every path under `src/` that a `run:` block hands to a tool exists, excluding build output and anything still carrying an unexpanded expression;
- no workflow invokes a cargo subcommand that resolves a package without naming the manifest, per [repo.layout];
- every editor task runs through `make`, so the Makefile stays the one place that knows where anything is.

These files MUST be read with their own formats' parsers — YAML for the workflows, JSON for the editor's JSON-with-comments — never by pattern. Only the shell inside a `run:` block is tokenised, because a shell script is not structured data; that tokenisation MUST find a command inside a substitution, since `version=$(cargo metadata …)` is the shape a broken invocation actually takes.

---
