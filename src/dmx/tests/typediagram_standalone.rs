//! `.td` + `.mustache` → `.dart`, driven the way a user drives it
//! [typediagram.standalone].
//!
//! Three files and no wrapper, through the real binary over real files. What
//! this suite is for is the *binding*: which template renders which definition,
//! where the output lands, what a watcher does when a template changes, and
//! what happens to a Mustache file that has nothing to do with dmx. The
//! pipeline behind it is the shared one, proven again over the whole parity
//! corpus by `typediagram_golden`.

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

#[path = "support/watch.rs"]
mod watch;

#[path = "support/workspace.rs"]
mod workspace;

use std::fs;
use std::io;

use watch::WatchProcess;
use workspace::Workspace;

/// The definition every fixture here renders.
const DEFINITION: &str = "# A parcel on its way to a customer.
type Parcel {
  id:      Uuid
  weightG: Int
  insured: Option<Decimal>
}
";

/// The template every fixture here renders it through.
const TEMPLATE: &str = "{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}

  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
";

/// A second template over the same definition, writing something else.
const WIRE_TEMPLATE: &str = "{{#declarations}}
const parcelWireNames = <String>[{{#fields}}'{{snakeName}}'{{comma}}{{/fields}}];
{{/declarations}}
";

/// A package with `models/parcel.td` in it, and whatever else `files` names.
fn package(files: &[(&str, &str)]) -> Workspace {
    let workspace = Workspace::create(
        "dmx-td-standalone",
        &["build", "models", "lib"],
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/parcel.td", DEFINITION),
        ],
    );
    for (name, contents) in files {
        workspace.write(name, contents);
    }
    workspace
}

/// [typediagram.canonical]: a definition on its own is enough. It renders
/// through the canonical model template, into an immutable value with its JSON
/// beside it — and a template beside it takes that template's place.
#[test]
fn a_definition_alone_generates_a_model_class() {
    let workspace = package(&[]);
    let first = workspace.build();
    assert!(first.contains("1 of 1 file(s) updated"), "{first}");

    let generated = workspace.read("lib/parcel.dart");
    assert!(
        generated.starts_with("// dmx: generated from models/parcel.td — do not edit.\n"),
        "{generated}"
    );
    assert!(
        generated.contains("// dmx: rendered through the canonical model template, definition "),
        "{generated}"
    );
    for expected in [
        "import 'package:dmx/dmx.dart' as dmx;",
        "final class Parcel {",
        "  bool operator ==(Object other) =>",
        "  int get hashCode => Object.hash(",
        "  String toString() => 'Parcel(",
        "  Parcel copyWith({",
        "extension ParcelJson on Parcel {",
        "  static dmx.Result<Parcel, dmx.DecodeError> fromJson(",
        "  Map<String, Object?> toJson() => <String, Object?>{",
    ] {
        assert!(
            generated.contains(expected),
            "missing `{expected}`:\n{generated}"
        );
    }

    // Idempotent, and `--check` sees no drift in what it just wrote. The file
    // count grew by one: what was written is Dart, and a pass over `lib` reads
    // it like any other source.
    let second = workspace.build();
    assert!(second.contains("0 of 2 file(s) updated"), "{second}");
    let checked = workspace.dmx(&["build", "models", "lib", "--check"]);
    assert!(
        checked.status.success(),
        "--check found drift in its own output:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    // A template of the definition's own name replaces the canonical one, and
    // removing it hands the file back rather than collecting it.
    workspace.write("models/parcel.mustache", TEMPLATE);
    let _ = workspace.build();
    let replaced = workspace.read("lib/parcel.dart");
    assert!(
        replaced.contains("rendered through models/parcel.mustache"),
        "{replaced}"
    );
    assert!(!replaced.contains("operator =="), "{replaced}");

    fs::remove_file(workspace.path("models/parcel.mustache")).expect("remove template");
    let _ = workspace.build();
    assert!(
        workspace
            .read("lib/parcel.dart")
            .contains("rendered through the canonical model template"),
        "the definition stopped rendering when its template went away"
    );
}

/// [typediagram.standalone]: a definition and the template beside it generate
/// Dart, a second template generates a second file, and a second build writes
/// nothing.
#[test]
fn a_definition_and_its_templates_generate_dart() {
    let workspace = package(&[
        ("models/parcel.mustache", TEMPLATE),
        ("models/parcel.wire.mustache", WIRE_TEMPLATE),
    ]);
    let first = workspace.build();
    // One source, whatever it writes: the definition is what a pass generates
    // from, and the two outputs are what it produced.
    assert!(first.contains("wrote: models/parcel.td"), "{first}");
    assert!(first.contains("1 of 1 file(s) updated"), "{first}");

    let generated = workspace.read("lib/parcel.dart");
    assert!(
        generated.starts_with("// dmx: generated from models/parcel.td — do not edit.\n"),
        "{generated}"
    );
    assert!(
        generated.contains("// dmx: rendered through models/parcel.mustache, definition "),
        "{generated}"
    );
    assert!(generated.contains("context v1"), "{generated}");
    assert!(generated.contains("final class Parcel {"), "{generated}");
    assert!(
        generated
            .contains("const Parcel({required this.id, required this.weightG, this.insured});"),
        "{generated}"
    );
    // `Option<Decimal>` is resolved before the template runs, and `Uuid` with it.
    assert!(generated.contains("final String? insured;"), "{generated}");
    assert!(generated.contains("final String id;"), "{generated}");

    let wire = workspace.read("lib/parcel_wire.dart");
    assert!(
        wire.contains("const parcelWireNames = <String>['id','weight_g','insured'];"),
        "{wire}"
    );
    assert!(
        wire.contains("rendered through models/parcel.wire.mustache"),
        "{wire}"
    );

    let second = workspace.build();
    assert!(second.contains("0 of "), "a second build rewrote: {second}");
    assert_eq!(
        workspace.read("models/parcel.td"),
        DEFINITION,
        "the definition is the source of truth and is never rewritten"
    );
    assert_eq!(
        workspace.read("models/parcel.mustache"),
        TEMPLATE,
        "the template is never rewritten either"
    );
}

/// [typediagram.standalone]: a leading Mustache comment moves the output, and
/// renders to nothing because it is a comment.
#[test]
fn a_leading_comment_moves_the_output() {
    let workspace = package(&[(
        "models/parcel.mustache",
        &format!("{{{{! dmx output=lib/models/parcel.dart }}}}\n{TEMPLATE}"),
    )]);
    let _ = workspace.build();

    assert!(!workspace.exists("lib/parcel.dart"));
    let generated = workspace.read("lib/models/parcel.dart");
    assert!(generated.contains("final class Parcel {"), "{generated}");
    assert!(!generated.contains("output="), "{generated}");
}

/// [typediagram.standalone]: a Mustache file with no definition beside it is
/// somebody else's, and a build leaves it and the tree alone.
#[test]
fn a_template_with_no_definition_generates_nothing() {
    let workspace = package(&[("templates/preview.mustache", TEMPLATE)]);
    let report = workspace.build();
    assert!(!workspace.exists("lib/preview.dart"), "{report}");
    // The definition beside it still renders, through the canonical model
    // template [typediagram.canonical] — the preview is simply not a source.
    assert!(
        workspace
            .read("lib/parcel.dart")
            .contains("rendered through the canonical model template"),
        "{report}"
    );

    // Naming it explicitly is the same answer, not a different one.
    let named = workspace.dmx(&["build", "templates/preview.mustache"]);
    assert!(
        named.status.success(),
        "{}",
        String::from_utf8_lossy(&named.stderr)
    );
    assert_eq!(workspace.read("templates/preview.mustache"), TEMPLATE);
}

/// [typediagram.standalone]: every refusal carries its code and names the file
/// a reader has to open — with no fence anywhere in the sentence.
#[test]
fn every_refusal_is_coded_and_names_a_file() {
    for (code, needle, files) in [
        (
            "DMX8004",
            "in models/parcel.td is not valid",
            vec![
                ("models/parcel.td", "type A { x: Int }\ntype B { y }\n"),
                ("models/parcel.mustache", TEMPLATE),
            ],
        ),
        (
            "DMX8001",
            "`dmx.ouput` is not a setting dmx knows",
            vec![(
                "models/parcel.mustache",
                "{{! dmx ouput=lib/parcel.dart }}\nx\n",
            )],
        ),
        (
            "DMX8001",
            "`typo` is not a `key=value` setting",
            vec![("models/parcel.mustache", "{{! dmx typo }}\nx\n")],
        ),
        (
            "DMX8005",
            "is an absolute path",
            vec![(
                "models/parcel.mustache",
                &format!("{{{{! dmx output=/etc/parcel.dart }}}}\n{TEMPLATE}"),
            )],
        ),
        (
            "DMX8005",
            "does not end in `.dart`",
            vec![(
                "models/parcel.mustache",
                "{{! dmx output=lib/parcel.txt }}\nx\n",
            )],
        ),
        (
            "DMX8003",
            "both generate `lib/parcel.dart`",
            vec![
                ("models/parcel.mustache", TEMPLATE),
                (
                    "models/parcel.wire.mustache",
                    &format!("{{{{! dmx output=lib/parcel.dart }}}}\n{TEMPLATE}"),
                ),
            ],
        ),
        (
            "DMX8010",
            "has no name left to generate under",
            vec![
                (
                    "models/parcel.td",
                    "type Circle { r: Float }\ntype ShapeCircle { r: Float }\nunion Shape { Circle { r: Float } }\n",
                ),
                ("models/parcel.mustache", TEMPLATE),
            ],
        ),
        (
            "DMX4003",
            "never throws",
            vec![(
                "models/parcel.mustache",
                "int probe() => throw StateError('{{#declarations}}{{name}}{{/declarations}}');\n",
            )],
        ),
    ] {
        let owned: Vec<(&str, &str)> = files.clone();
        let workspace = package(&owned);
        let error = workspace.build_failure();
        assert!(error.contains(code), "expected {code}:\n{error}");
        assert!(
            error.contains(needle),
            "expected {needle:?} in {code}:\n{error}"
        );
        assert!(
            error.contains("models/parcel."),
            "{code} must name the file to open:\n{error}"
        );
        assert!(
            !error.contains("fence"),
            "{code} talks about fences in a file:\n{error}"
        );
        assert!(
            !workspace.exists("lib/parcel.dart"),
            "{code} wrote an output anyway"
        );
    }
}

/// [typediagram.output]: an output that exists without dmx's marker is
/// hand-written and is never overwritten.
#[test]
fn a_hand_written_output_is_refused() {
    let workspace = package(&[
        ("models/parcel.mustache", TEMPLATE),
        ("lib/parcel.dart", "// mine\n"),
    ]);
    let error = workspace.build_failure();
    assert!(error.contains("DMX8006"), "{error}");
    assert_eq!(workspace.read("lib/parcel.dart"), "// mine\n");
}

/// [typediagram.output]: a template that goes away takes its output with it.
#[test]
fn a_removed_template_collects_its_output() {
    let workspace = package(&[
        ("models/parcel.mustache", TEMPLATE),
        ("models/parcel.wire.mustache", WIRE_TEMPLATE),
    ]);
    let _ = workspace.build();
    assert!(workspace.exists("lib/parcel_wire.dart"));

    fs::remove_file(workspace.path("models/parcel.wire.mustache")).expect("remove the template");
    let report = workspace.build();
    assert!(report.contains("1 of "), "{report}");
    assert!(
        !workspace.exists("lib/parcel_wire.dart"),
        "a dropped template means a dropped file"
    );
    assert!(
        workspace.exists("lib/parcel.dart"),
        "the other output stays"
    );
}

/// [typediagram.execution]: `--check` reports drift, writes nothing, and exits
/// 2 — and says nothing once the tree is up to date.
#[test]
fn check_reports_drift_without_writing() {
    let workspace = package(&[("models/parcel.mustache", TEMPLATE)]);
    let drifted = workspace.dmx(&["build", "models", "lib", "--check"]);
    assert_eq!(drifted.status.code(), Some(2));
    let report = String::from_utf8_lossy(&drifted.stdout);
    assert!(report.contains("drift: models/parcel.td"), "{report}");
    assert_eq!(
        report.matches("drift:").count(),
        1,
        "one source drifted, so one line: {report}"
    );
    assert!(!workspace.exists("lib/parcel.dart"));

    let _ = workspace.build();
    let clean = workspace.dmx(&["build", "models", "lib", "--check"]);
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stdout)
    );
}

/// [typediagram.execution]: `dmx explain` takes the definition or a template
/// bound to it, and answers with the same report either way.
#[test]
fn explain_takes_the_definition_or_its_template() {
    let workspace = package(&[("models/parcel.mustache", TEMPLATE)]);
    let reports: Vec<String> = ["models/parcel.td", "models/parcel.mustache"]
        .into_iter()
        .map(|named| {
            let output = workspace.dmx(&["explain", named]);
            assert!(
                output.status.success(),
                "`dmx explain {named}`:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout).into_owned()
        })
        .collect();

    assert_eq!(reports[0], reports[1], "a template explains its definition");
    let report = &reports[0];
    assert!(
        report.contains("models/parcel.td: 1 generation group(s)"),
        "{report}"
    );
    assert!(
        report
            .contains("-> lib/parcel.dart (target dart, template models/parcel.mustache, digest "),
        "{report}"
    );
    assert!(report.contains("group 1 — the definition file"), "{report}");
    assert!(
        report.contains("\"template\": \"models/parcel.mustache\""),
        "{report}"
    );
    assert!(report.contains("\"dartType\": \"String\""), "{report}");
    assert!(!workspace.exists("lib/parcel.dart"), "explain wrote a file");
}

/// [typediagram.execution]: the watcher generates on startup, and answers an
/// edit to the *template* — which is not a source of its own — by regenerating
/// the definition it is bound to.
#[test]
fn watch_answers_an_edit_to_either_file() -> io::Result<()> {
    let workspace = package(&[("models/parcel.mustache", TEMPLATE)]);
    let mut watcher = WatchProcess::spawn_ready_in(workspace.root(), &["models", "lib"])?;
    let first = workspace.read("lib/parcel.dart");
    assert!(first.contains("final int weightG;"), "{first}");

    // The definition.
    workspace.write("models/parcel.td", &DEFINITION.replace("Int", "Float"));
    watcher.wait_for_line_on("stdout: wrote: ", "parcel.td")?;
    let second = workspace.read("lib/parcel.dart");
    assert!(second.contains("final double weightG;"), "{second}");

    // The template. Nothing generates *from* a `.mustache` file, so what has
    // to happen is that its definition runs again.
    workspace.write(
        "models/parcel.mustache",
        &TEMPLATE.replace("final class", "abstract final class"),
    );
    watcher.wait_for_line_on("stdout: wrote: ", "parcel.td")?;
    let third = workspace.read("lib/parcel.dart");
    assert!(third.contains("abstract final class Parcel {"), "{third}");

    // An invalid save keeps the last valid output and the watcher alive.
    workspace.write("models/parcel.td", "type Parcel {\n  weightG:\n}\n");
    watcher.wait_for_line_on("stderr: ", "DMX8004")?;
    assert_eq!(
        workspace.read("lib/parcel.dart"),
        third,
        "an invalid definition must leave the last valid output alone"
    );
    assert!(
        watcher.is_running()?,
        "watcher stopped:\n{}",
        watcher.output()
    );
    Ok(())
}
