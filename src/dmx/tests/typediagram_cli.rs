//! typeDiagram documents through the real `dmx` binary [typediagram].
//!
//! Black box throughout: a scratch workspace on a real filesystem, real
//! Markdown, real Mustache, and the shipped binary run the way a person or a
//! Makefile runs it. Nothing here reaches into the crate — what is asserted is
//! the bytes on disk and the output contract on stdout and stderr.

// [TEST-RULES] admits `expect` in a test: a fixture that cannot be built is a
// broken test, and unwinding at the point of failure names it better than any
// `Result` plumbing would. Production code is still held to `unwrap_used` and
// `expect_used` at deny — this relaxation is `cfg(test)`-scoped on purpose.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

mod support;

#[path = "support/workspace.rs"]
mod workspace;

use std::fs;
use std::process::Command;

use workspace::Workspace;

/// A record definition and one template over it — the canonical document.
const STORE: &str = r#"# Store models

The definitions below are the source of truth. Everything under them is
ordinary prose and must survive untouched.

```typeDiagram
type Product {
  id:    Uuid
  name:  String
  price: Decimal
  note:  Option<String>
}

union Availability {
  InStock { count: Int }
  Backordered { until: DateTime }
  Discontinued
}
```

```mustache {"dmx":{"output":"lib/models.dart"}}
{{#declarations}}
{{#isRecord}}
final class {{name}}{{genericDeclaration}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/isRecord}}
{{#isUnion}}
sealed class {{name}}{{genericDeclaration}} {
  const {{name}}();
}
{{#variants}}

final class {{name}} extends {{#last}}{{/last}}Availability {
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/variants}}
{{/isUnion}}
{{/declarations}}
```

```mustache {"dmx":{"output":"lib/names.dart"}}
/// The declared names, in source order.
const declaredNames = <String>[
{{#declarations}}
  '{{name}}',
{{/declarations}}
];
```

That is the whole document.
"#;

/// A workspace holding one document at `docs/store.dmx.md`.
fn document_workspace(document: &str) -> Workspace {
    Workspace::create(
        "dmx-typediagram",
        &["build", "docs", "lib"],
        &[("docs/store.dmx.md", document)],
    )
}

/// [typediagram.execution]: one document, two templates, two owned files —
/// and a second build that writes nothing.
#[test]
fn a_document_generates_every_bound_template_once() {
    let workspace = document_workspace(STORE);

    let first = workspace.build();
    assert!(first.contains("wrote: docs/store.dmx.md"), "{first}");

    let models = workspace.read("lib/models.dart");
    assert!(models.starts_with("// dmx: generated from docs/store.dmx.md — do not edit."));
    assert!(models.contains("// dmx: group 1, fences 1/2,"), "{models}");
    assert!(models.contains("final class Product {"), "{models}");
    assert!(
        models.contains(
            "const Product({required this.id, required this.name, required this.price, this.note});"
        ),
        "{models}"
    );
    assert!(models.contains("  final String? note;"), "{models}");
    assert!(models.contains("sealed class Availability {"), "{models}");
    assert!(
        models.contains("final class InStock extends Availability {"),
        "{models}"
    );
    assert!(models.contains("  final DateTime until;"), "{models}");

    let names = workspace.read("lib/names.dart");
    assert!(names.contains("// dmx: group 1, fences 1/3,"), "{names}");
    assert!(names.contains("'Product',"), "{names}");
    assert!(names.contains("'Availability',"), "{names}");

    let second = workspace.build();
    assert!(
        second.contains("dmx: 0 of") && !second.contains("wrote:"),
        "a second build must write nothing:\n{second}"
    );
    assert_eq!(workspace.read("lib/models.dart"), models);
    assert_eq!(
        workspace.read("docs/store.dmx.md"),
        STORE,
        "the document is never rewritten"
    );
}

/// [typediagram.execution]: generation is deterministic — the same document
/// produces the same bytes from a clean workspace every time.
#[test]
fn generation_is_byte_identical_across_workspaces() {
    let first = document_workspace(STORE);
    let _ = first.build();
    let second = document_workspace(STORE);
    let _ = second.build();
    assert_eq!(
        first.read("lib/models.dart"),
        second.read("lib/models.dart")
    );
    assert_eq!(first.read("lib/names.dart"), second.read("lib/names.dart"));
}

/// [typediagram.documents]: CRLF input generates the same model, and the
/// document still is not rewritten.
#[test]
fn a_crlf_document_generates_the_same_model() {
    let workspace = document_workspace(&STORE.replace('\n', "\r\n"));
    let _ = workspace.build();
    assert!(
        workspace
            .read("lib/models.dart")
            .contains("final class Product {")
    );
    assert!(workspace.read("docs/store.dmx.md").contains("\r\n"));
}

/// [typediagram.execution]: `--check` reports drift, exits 2, and writes
/// nothing; once the outputs are current it exits 0.
#[test]
fn check_reports_drift_and_writes_nothing() {
    let workspace = document_workspace(STORE);

    let drift = workspace.dmx(&["build", "docs", "lib", "--check"]);
    assert_eq!(drift.status.code(), Some(2), "drift must exit 2");
    assert!(
        String::from_utf8_lossy(&drift.stdout).contains("drift: docs/store.dmx.md"),
        "{}",
        String::from_utf8_lossy(&drift.stdout)
    );
    assert!(!workspace.exists("lib/models.dart"), "--check never writes");

    let _ = workspace.build();
    let current = workspace.dmx(&["build", "docs", "lib", "--check"]);
    assert!(current.status.success(), "a current workspace has no drift");
}

/// [typediagram.documents]: recursive discovery takes `*.dmx.md`; another
/// Markdown file is documentation until somebody names it.
#[test]
fn other_markdown_is_documentation_until_it_is_named() {
    let workspace = document_workspace(STORE);
    workspace.write(
        "docs/notes.md",
        "```typeDiagram\ntype Note { body: String }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/notes.dart\"}}\n// {{#declarations}}{{name}}{{/declarations}}\n```\n",
    );

    let _ = workspace.build();
    assert!(
        !workspace.exists("lib/notes.dart"),
        "an ordinary .md is not discovered"
    );

    let named = workspace.dmx(&["build", "docs/notes.md"]);
    assert!(
        named.status.success(),
        "{}",
        String::from_utf8_lossy(&named.stderr)
    );
    assert!(workspace.read("lib/notes.dart").contains("// Note"));
}

/// [typediagram.binding]: a definition nobody templates, a Mustache fence with
/// no dmx metadata, and an unrelated fence all generate nothing.
#[test]
fn documentation_only_content_generates_nothing() {
    let workspace = document_workspace(
        "# Notes\n\n```typeDiagram\ntype A { x: Int }\n```\n\n```mustache\n{{name}}\n```\n\n```dart\nclass A {}\n```\n",
    );
    let output = workspace.build();
    assert!(output.contains("dmx: 0 of"), "{output}");
    assert!(!workspace.exists("lib/a.dart"));
}

/// [typediagram.output]: a hand-written file is never overwritten, and the
/// build fails rather than proceeding.
#[test]
fn a_hand_written_output_is_never_overwritten() {
    let workspace = document_workspace(STORE);
    workspace.write("lib/models.dart", "// mine, by hand\n");

    let error = workspace.build_failure();
    assert!(error.contains("DMX8006"), "{error}");
    assert_eq!(workspace.read("lib/models.dart"), "// mine, by hand\n");
}

/// [typediagram.output]: a dropped template drops its file.
#[test]
fn a_removed_template_collects_its_output() {
    let workspace = document_workspace(STORE);
    let _ = workspace.build();
    assert!(workspace.exists("lib/names.dart"));

    let trimmed = STORE
        .split("```mustache {\"dmx\":{\"output\":\"lib/names.dart\"}}")
        .next()
        .expect("the document up to the second template")
        .to_owned();
    workspace.write("docs/store.dmx.md", &trimmed);

    let _ = workspace.build();
    assert!(
        workspace.exists("lib/models.dart"),
        "the surviving output stays"
    );
    assert!(
        !workspace.exists("lib/names.dart"),
        "the dropped output goes"
    );
}

/// [typediagram.diagnostics]: every refusal names its code, the document, and
/// the line — and none of them writes anything.
#[test]
fn every_refusal_is_coded_and_located() {
    for (code, needle, document) in [
        (
            "DMX8001",
            "line 5",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"ouput\":\"lib/a.dart\"}}\na\n```\n",
        ),
        (
            "DMX8002",
            "line 1",
            "```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n",
        ),
        (
            "DMX8003",
            "on line 5 and the Mustache template in docs/store.dmx.md on line 9",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nb\n```\n",
        ),
        (
            "DMX8004",
            "line 2, column 13",
            "```typeDiagram\ntype A { x: Timestamp }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\n// a\n```\n",
        ),
        (
            "DMX8005",
            "leaves the workspace",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"../escape.dart\"}}\n// a\n```\n",
        ),
        (
            "DMX8007",
            "available: dart",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\",\"target\":\"kotlin\"}}\n// a\n```\n",
        ),
        (
            "DMX8008",
            "template fence on line 5",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nfinal class {{#declarations}}{{name}}{{/declarations}} {\n```\n",
        ),
        (
            "DMX4003",
            "never throws",
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nint probe() => throw StateError('{{#declarations}}{{name}}{{/declarations}}');\n```\n",
        ),
    ] {
        let workspace = document_workspace(document);
        let error = workspace.build_failure();
        assert!(error.contains(code), "expected {code}:\n{error}");
        assert!(
            error.contains(needle),
            "expected {needle:?} in {code}:\n{error}"
        );
        assert!(
            error.contains("store.dmx.md"),
            "{code} must name the document:\n{error}"
        );
        assert!(
            !workspace.exists("lib/a.dart"),
            "{code} wrote an output anyway"
        );
        assert!(
            !workspace.exists("escape.dart"),
            "{code} wrote outside the workspace"
        );
    }
}

/// [typediagram.execution]: `explain` prints the groups, their dependencies,
/// their outputs, and the exact context — and generates nothing.
#[test]
fn explain_prints_the_context_and_writes_nothing() {
    let workspace = document_workspace(STORE);
    let output = workspace.dmx(&["explain", "docs/store.dmx.md"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8_lossy(&output.stdout);

    assert!(
        report.contains("docs/store.dmx.md: 1 generation group(s)"),
        "{report}"
    );
    assert!(report.contains("2 declaration(s)"), "{report}");
    assert!(
        report.contains("-> lib/models.dart (target dart, fence 2"),
        "{report}"
    );
    assert!(
        report.contains("-> lib/names.dart (target dart, fence 3"),
        "{report}"
    );
    assert!(report.contains("\"modelVersion\": 1"), "{report}");
    assert!(report.contains("\"isRecord\": true"), "{report}");
    assert!(report.contains("\"dartType\": \"String?\""), "{report}");
    assert!(
        report.contains("\"typeDiagram\": \"Option<String>\""),
        "{report}"
    );
    assert!(
        !workspace.exists("lib/models.dart"),
        "explain never generates"
    );

    // `explain` names one file. There is no useful default, and a Dart source
    // is not what it explains yet.
    for (args, needle) in [
        (vec!["explain"], "takes exactly one file"),
        (vec!["explain", "docs", "lib"], "takes exactly one file"),
        (
            vec!["explain", "lib/models.dart"],
            "a typeDiagram definition (`.td`)",
        ),
    ] {
        let refused = workspace.dmx(&args);
        assert!(
            !refused.status.success(),
            "`dmx {}` should have failed",
            args.join(" ")
        );
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains(needle),
            "`dmx {}`:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&refused.stderr)
        );
    }
}

/// [typediagram.execution]: prose outside a group is not a dependency of its
/// output, and a definition change is.
#[test]
fn only_the_group_is_a_dependency_of_its_output() {
    let workspace = document_workspace(STORE);
    let _ = workspace.build();
    let before = workspace.read("lib/models.dart");

    workspace.write(
        "docs/store.dmx.md",
        &format!("{STORE}\nAn added paragraph.\n"),
    );
    let after_prose = workspace.build();
    assert!(
        after_prose.contains("dmx: 0 of"),
        "prose is not a dependency:\n{after_prose}"
    );
    assert_eq!(workspace.read("lib/models.dart"), before);

    workspace.write(
        "docs/store.dmx.md",
        &STORE.replace("price: Decimal", "price: Float"),
    );
    let _ = workspace.build();
    let after_definition = workspace.read("lib/models.dart");
    assert_ne!(
        after_definition, before,
        "a definition change is a dependency"
    );
    assert!(
        after_definition.contains("final double price;"),
        "{after_definition}"
    );
}

/// [typediagram.macro]: `typeDiagram` is a built-in macro name, so an
/// annotation may not claim it and a Dart file may not be generated by it.
#[test]
fn the_builtin_name_is_not_an_annotation() {
    let workspace = document_workspace(STORE);
    workspace.write(
        "lib/hand.dart",
        "@dmx('typeDiagram')\nclass Hand {\n  final int a = 0;\n}\n",
    );
    let error = workspace.build_failure();
    assert!(error.contains("DMX2006"), "{error}");
    assert!(error.contains("Markdown generation group"), "{error}");
}

/// [typediagram.output]: an output path is resolved against the package the
/// document belongs to, so the same document generates the same bytes in the
/// same place however dmx was launched.
#[test]
fn outputs_land_in_the_package_the_document_belongs_to() {
    let workspace = document_workspace("# empty\n");
    workspace.write("packages/store/pubspec.yaml", "name: store\n");
    workspace.write("packages/store/docs/models.dmx.md", STORE);

    // From the repository root, naming the package's document directory.
    let root_run = workspace.dmx(&["build", "packages"]);
    assert!(
        root_run.status.success(),
        "{}",
        String::from_utf8_lossy(&root_run.stderr)
    );
    assert!(
        workspace.exists("packages/store/lib/models.dart"),
        "the output belongs to the package, not to the directory dmx ran in"
    );
    assert!(!workspace.exists("lib/models.dart"));
    let from_root = workspace.read("packages/store/lib/models.dart");
    assert!(
        from_root.starts_with("// dmx: generated from docs/models.dmx.md"),
        "the document is recorded relative to its own package:\n{from_root}"
    );

    // From inside the package, naming the same document relatively.
    let inside = Command::new(env!("CARGO_BIN_EXE_dmx"))
        .args(["build", "docs", "lib"])
        .current_dir(workspace.path("packages/store"))
        .output()
        .expect("run dmx");
    assert!(
        inside.status.success(),
        "{}",
        String::from_utf8_lossy(&inside.stderr)
    );
    assert!(
        String::from_utf8_lossy(&inside.stdout).contains("dmx: 0 of"),
        "the same document from another directory must write nothing:\n{}",
        String::from_utf8_lossy(&inside.stdout)
    );
    assert_eq!(workspace.read("packages/store/lib/models.dart"), from_root);
}

/// [typediagram.output]: two live documents may not both generate one file,
/// but a renamed document takes its own outputs with it.
#[test]
fn one_output_has_one_live_source() {
    let workspace = document_workspace(STORE);
    let _ = workspace.build();
    assert!(workspace.exists("lib/models.dart"));

    // A second document claiming the same output: each pass would undo the
    // other's, so the build fails instead of flip-flopping.
    workspace.write("docs/rival.dmx.md", STORE);
    let error = workspace.build_failure();
    assert!(error.contains("DMX8006"), "{error}");
    assert!(error.contains("already generated from"), "{error}");
    assert!(error.contains("store.dmx.md"), "{error}");

    // Renaming the document is not a collision: the marker names a source that
    // is gone, so the new one takes its own outputs over.
    fs::remove_file(workspace.path("docs/rival.dmx.md")).expect("remove the rival");
    fs::rename(
        workspace.path("docs/store.dmx.md"),
        workspace.path("docs/renamed.dmx.md"),
    )
    .expect("rename the document");

    let after = workspace.build();
    assert!(after.contains("wrote: docs/renamed.dmx.md"), "{after}");
    assert!(
        workspace
            .read("lib/models.dart")
            .starts_with("// dmx: generated from docs/renamed.dmx.md"),
        "{}",
        workspace.read("lib/models.dart")
    );
}
