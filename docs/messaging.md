# dmx Messaging

## Canonical message

> **Dart Macros**
>
> **Fast, Reliable Code Generation on Every Save**
>
> **No Generated `part` Files. Fully Customizable.**

Keep these three lines and their order intact.

## Target audience

The target audience is the Flutter community. Specifically, we want to reach Flutter developers who need generated model code but are not satisfied with generated `part` files, package-defined APIs, and generated-file churn.

Model generation is the primary entry point. Speak directly to developers using or evaluating Freezed, `json_serializable`, dart_mappable, built_value, and similar packages for `copyWith`, equality, `hashCode`, `toString`, unions, and JSON. Those packages solve real problems; dmx offers a different workflow and lets the project control the generated Dart. Do not imply feature-for-feature compatibility.

## Core pitch

**Open the project. Edit Dart. Save. The generated code updates automatically.**

Before installing anything, users can open the [web playground](https://dmx.dev/playground.html), edit Dart and the real Mustache template, and run the production generator in their browser. No package, CLI, or toolchain is required, and neither input leaves the tab.

The VS Code extension starts the file watcher when the workspace opens. It regenerates only affected declarations and skips unchanged writes, so there is no generation command to remember or rerun. With another editor, start `dmx watch` once and leave it running.

Plain Dart classes have no generated value semantics or typed JSON codecs: equality is identity by default, while Flutter recommends code generation for larger JSON models because manual decoding is repetitive and error-prone and runtime reflection is disabled.[^why-codegen] Code generation fills that gap; dmx keeps the benefit while letting each team define the emitted model shape instead of inheriting a package's fixed structure.

`@dmx('model')` generates `copyWith`, equality, `hashCode`, `toString`, and JSON.[^dart-mappable] Generated members stay inside the class—no `part` directive, `.g.dart` fragment, generated mixin, or delegating factory.

dmx does not replace one fixed model shape with another. Teams encode their exact Dart conventions in simple Mustache templates. When a template is not enough, a custom Dart macro can inspect parsed declarations and siblings, read another source of truth, and generate members or complete files. The [SQLite example](../examples/dmx_sqlite_example/README.md) reads a live database and generates one row class per table or view.

Custom macros and Mustache templates are one system, not competing options. A macro can return Dart directly, and a small one usually should. It can also hand its model to a Mustache template and let dmx render it with the same engine the built-ins use, which is how a project keeps generation logic and output shape in separate files: the macro answers questions only the project can answer, and the template decides what the emitted Dart looks like. The [OpenAPI example](../examples/dmx_openapi_example/README.md) reads a published API document and renders a typed client and its models through project-owned templates.

Some models have no Dart file to annotate yet. A `*.dmx.md` document holds a [typeDiagram](https://typediagram.dev/docs/) definition and, immediately below it, the Mustache templates that generate from it. Save the document and dmx writes the `.dart` files those templates name. The definition still renders as a diagram in any typeDiagram viewer, so one page is the model, the documentation, and the build input. The [shipping document](../examples/storefront/docs/shipping.dmx.md) defines four types once and generates two different Dart files from them.

> Save the file and keep coding. Use a built-in, change what it emits with a Mustache template, write a macro in Dart—and render a Mustache template from inside that macro too—or define the types in Markdown and let the templates write the Dart.

## Ready-to-use copy

### One line

**Save a Dart file and dmx updates the generated code automatically—without a generated `part` file.**

### Homepage

**Code generation on every save.**

Open the project, edit Dart, and save. dmx updates generated code automatically, validates it, and writes it directly into the declaration. Use built-ins for common jobs, shape the output with Mustache, or write a Dart macro that generates something entirely new.

[Try the real generator in your browser](https://dmx.dev/playground.html)—no install required.

### Models in Markdown

**Define the types once in a `*.dmx.md` document; the Mustache templates under the diagram write the Dart.** Save the document and every file it names updates—no annotated Dart source, no `part` file, and the diagram still renders.

### Repository

Fast Dart code generation on every save, with no generated `part` files: built-in macros, team-owned Mustache templates, custom Dart macros, models defined in Markdown, and validated inline output.

## Message order

1. **File watching:** save and see generated code update automatically.
2. **Try it without installing:** the browser playground runs the real generator with editable Dart and Mustache.
3. **No generated `part` files:** read, search, review, and debug one complete Dart file.
4. **Useful immediately:** built-ins cover common models, unions, routes, clients, and more.
5. **The team's shape:** Mustache controls the exact generated Dart.
6. **Full custom macros:** inspect typed declaration data, read project data, and generate members or complete files—returning Dart directly, or rendering it through the same Mustache engine the built-ins use.
7. **Models with no Dart to annotate:** a `*.dmx.md` document defines the types once and its Mustache templates generate the `.dart` files.
8. **Reliable writes:** validate complete Dart files before writing and preserve handwritten source on failure.

## Demo order

1. Open the browser playground and change both Dart and Mustache—no install.
2. In the installed workflow, start with the watcher already running; do not lead with a terminal build.
3. Rename a field, save, and show generated members update immediately.
4. Change a Mustache template and show the team's model shape appear.
5. Add a SQLite table and show a complete Dart file appear.
6. Add a field to a `*.dmx.md` diagram, save, and show both generated Dart files change together.

## Positioning

Flutter's own issue tracker records complaints about generator setup, delayed feedback, `part` boilerplate, and generated files polluting navigation and search.[^flutter-codegen-issue] Developers also report navigation landing in generated fragments, merge conflicts, inflexible generated APIs, and unwanted indirection.[^community-parts][^community-generated-files]

Use their language: **no generated `part` files**, **one complete Dart file**, **save and keep coding**, and **control the generated Dart**. Do not lead with implementation terminology.

`build_runner` is Dart's supported general-purpose build system. It supports watch mode and continues to improve.[^dart-build-runner][^build-runner-changelog] Do not position dmx by attacking it. Generated `part` files and fixed output shapes are decisions made by model-generator packages layered on that system, not defects in `build_runner` itself. The dmx distinction is inline output, complete-file validation, team-owned templates, and custom Dart macros.

Dart stopped work on its experimental compiler macros before they shipped.[^dart-macros-stopped] Current Dart documentation directs projects to external generation and explicitly says Dart's compilers do not support macros.[^dart-build-runner] This is context for why another code-generation workflow is useful; never imply that dmx is the abandoned compiler feature.

| Common generator workflow | dmx |
| --- | --- |
| Builder and asset graph | Direct annotation-to-source pipeline |
| `.g.dart` or `part` fragments | Members inside the declaration |
| Package-defined output shape | Team-owned Mustache shape |
| Package authors define generators | Projects define macros in Dart |

Custom macros may intentionally author complete standalone files for jobs such as SQLite schema generation. These are whole files with explicit ownership, not fragments that complete a handwritten class.

Version control is the team's choice. Commit generated files when a checkout should work immediately, or add complete generated files to `.gitignore` and recreate them on save or in CI. Inline regions live inside tracked source, so Git cannot ignore only the generated region.

## Guardrails

- **Write plainly. No slogans.** Every headline, card title, and section opener must name something concrete: a member dmx writes (`copyWith`, `toJson`, `==`), a file it does not write (`.g.dart`, `part`), a command, or an event (save, validate, rewrite). Lines whose meaning depends on the reader already knowing the product—"keeps you in the flow", "start ready, make it yours", "the point where dmx becomes an open metaprogramming system", "built for visible code and quiet runtimes"—are not acceptable copy. A Flutter developer who reads the line alone must be able to say what the tool does.
- **Do not describe the product with abstractions.** "Metaprogramming system", "open platform", and "catalogue" describe categories, not behaviour. Say what runs, when it runs, and what it writes.
- **"Metaprogramming" and "out-of-band" are banned from user-facing copy.** Neither means anything to a Flutter developer, and both describe how dmx is built rather than what it does for the reader. dmx is a **code generator**; it runs **when you save**, **before you build**, and **outside the compiler**. Say that instead. The terms remain correct in `docs/specs/` and `docs/plans/`, which are written for people working on dmx.
- Numbers must match the code. The registry in `src/macros/mod.rs` is the count of built-in macros; catalogue-only designs are not built-ins.
- Describe generation as immediate on save. Publish numeric speed comparisons only with a reproducible benchmark.
- “No install” describes the web playground; project file watching requires the extension or CLI.
- “No command” is true with the VS Code extension; other editors start `dmx watch` once.
- dmx covers familiar Freezed and dart_mappable jobs; do not imply feature-for-feature compatibility.
- Today, Dart macros receive typed parsed declarations, not the analyzer's full semantic model. Full semantic resolution, type inference, and typed staged expansion are planned; do not imply they are implemented yet.
- “Complete” means built-ins, templates, and custom macros—not every form of compile-time reflection.
- Never present Mustache and custom Dart macros as a choice between two paths. A macro may return Dart directly *or* render a template, and the two are designed to be used together; describing templates as what you use “instead of” a macro, or a macro as what you write “when Mustache runs out”, misstates the design.
- “Commit or ignore” applies directly to complete generated files; inline output is committed with its source file.
- Never attack `build_runner`, claim it cannot watch, call dmx a drop-in Freezed replacement, or lead with compiler-architecture jargon. Lead with no generated `part` files.
- Never say dmx "runs typeDiagram", "calls the typeDiagram CLI", or "uses `typediagram --to dart`". It does none of those. dmx reads the definition itself, and the Mustache template decides every generated byte—so never imply the output shape comes from typeDiagram either. Installing dmx installs nothing else: no Node, no npm package, no `typediagram` executable.
- A template binds to the definition **immediately above it**. Never show or describe a heading, a paragraph, or another fence between them, and never suggest binding follows a heading's text or a fence's position in the document.
- An output path in a document is relative to the package the document belongs to—the nearest `pubspec.yaml`. Do not describe it as relative to the document, to the repository root, or to wherever dmx was run.

## Calls to action

- **[Try dmx in your browser—no install](https://dmx.dev/playground.html)**
- **[Install the VS Code extension](vscode:extension/Nimblesite.dmx)** — the `vscode:` link opens the extension in the editor. Pair it with the Marketplace link below rather than using either alone: the `vscode:` link does nothing for a reader who has no VS Code installed, and the Marketplace page is the only one a browser can render.
- **[View on the Marketplace](https://marketplace.visualstudio.com/items?itemName=Nimblesite.dmx)** — the publisher is `Nimblesite`, capitalised. The lowercase `nimblesite.dmx` form is what the gallery API accepts and the web page 404s on, so it must never appear in a link.
- **See code generation on every save**
- **Shape your team's model template**
- **Build a custom Dart macro**
- **See a SQLite schema become Dart**
- **See an OpenAPI document become a typed client**

## Sources

Research checked 15 August 2026.

[^dart-mappable]: The official [dart_mappable package documentation](https://pub.dev/packages/dart_mappable) lists serialization, equality, `toString`, and `copyWith`.
[^why-codegen]: The Dart API documents [identity-based default equality](https://api.dart.dev/dart-core/Object/operator_equals.html); Flutter's official [JSON guide](https://docs.flutter.dev/data-and-backend/serialization/json) recommends code generation for medium-to-large projects and explains that runtime reflection is disabled.
[^dart-build-runner]: The official [`build_runner` documentation](https://pub.dev/packages/build_runner) describes its build and watch commands.
[^build-runner-changelog]: The official [`build_runner` changelog](https://github.com/dart-lang/build/blob/master/build_runner/CHANGELOG.md) records recent incremental-build and startup improvements.
[^flutter-codegen-issue]: Flutter's triaged [Code generation experience needs improvements](https://github.com/flutter/flutter/issues/63323) issue records setup, command, feedback, `part`-file, and generated-file complaints.
[^dart-macros-stopped]: Dart's official [macros and data serialization update](https://dart.dev/blog/an-update-on-dart-macros-data-serialization) explains why work stopped before general-purpose macros shipped.
[^community-parts]: Developers call out generated-file sprawl in [Riverpod generator usage](https://www.reddit.com/r/FlutterDev/comments/1h7kwmw) and `part`-file friction in [What is your biggest pain as a Flutter developer?](https://www.reddit.com/r/FlutterDev/comments/12iyslb).
[^community-generated-files]: Developers discuss navigation, extra `.g.dart` files, inflexible output, and merge conflicts in [What is your opinion on code generation in Flutter?](https://www.reddit.com/r/FlutterDev/comments/1pn9a8j) and [I don't like tools like Freezed](https://www.reddit.com/r/FlutterDev/comments/1b1lijj).
