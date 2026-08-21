//! Three files and no wrapper [typediagram.standalone].
//!
//! Every assertion here is about the *binding*: which template renders which
//! definition, where the render lands, and what happens when a name matches
//! more than one definition. The pipeline behind it is [`super::super::run`],
//! which the Markdown front end proves just as hard.

use std::fs;
use std::path::{Path, PathBuf};

use super::super::scratch::in_workspace;
use super::{definition_of, is_definition, is_template, output_name};
use crate::{Options, Outcome};

/// The definition every fixture below renders.
const DEFINITION: &str = "type Product {\n  id: Uuid\n  name: String\n}\n";

/// The template every fixture below renders it through.
const TEMPLATE: &str = "{{#declarations}}\nfinal class {{name}} {\n  const \
                        {{name}}({{{constructorParameters}}});\n{{#fields}}\n  final \
                        {{{dartType}}} {{name}};\n{{/fields}}\n}\n{{/declarations}}\n";

/// The pipeline options for a real build.
fn build() -> Options {
    Options {
        insert_regions: false,
        check: false,
    }
}

/// The one root a build of these fixtures manages.
fn roots() -> Vec<PathBuf> {
    vec![PathBuf::from("lib")]
}

/// [typediagram.standalone]: a `.td` file and the `.mustache` beside it
/// generate a Dart file, with no document anywhere and no metadata anywhere.
#[test]
fn a_definition_and_the_template_beside_it_generate_dart() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("build"),
                Outcome::Updated
            );
            let generated =
                fs::read_to_string(directory.join("lib").join("product.dart")).expect("output");
            assert!(
                generated.starts_with("// dmx: generated from models/product.td — do not edit.\n"),
                "{generated}"
            );
            assert!(
                generated.contains("// dmx: rendered through models/product.mustache, definition "),
                "{generated}"
            );
            assert!(generated.contains("final class Product {"), "{generated}");
            assert!(
                generated.contains("const Product({required this.id, required this.name});"),
                "{generated}"
            );
            assert!(generated.contains("final String id;"), "{generated}");

            // Idempotent, and neither source is ever rewritten.
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("second build"),
                Outcome::Unchanged
            );
            assert_eq!(
                fs::read_to_string(&definition).expect("definition"),
                DEFINITION
            );
            assert_eq!(
                fs::read_to_string(directory.join("models").join("product.mustache"))
                    .expect("template"),
                TEMPLATE
            );
        },
    );
}

/// [typediagram.standalone]: a second template beside the same definition is a
/// second output, named after the template rather than after the definition.
#[test]
fn a_second_template_is_a_second_output() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
            (
                "models/product.wire.mustache",
                "{{#declarations}}\nconst productWireNames = <String>[{{#fields}}'{{snakeName}}', \
                 {{/fields}}];\n{{/declarations}}\n",
            ),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("build"),
                Outcome::Updated
            );
            let wire = fs::read_to_string(directory.join("lib").join("product_wire.dart"))
                .expect("second output");
            assert!(wire.contains("const productWireNames"), "{wire}");
            assert!(wire.contains("'id', 'name',"), "{wire}");
            assert!(
                wire.contains("rendered through models/product.wire.mustache"),
                "{wire}"
            );
            assert!(directory.join("lib").join("product.dart").is_file());
        },
    );
}

/// [typediagram.standalone]: a leading Mustache comment moves the output, and
/// stays in the template — it is a comment, so it renders to nothing.
#[test]
fn a_leading_comment_moves_the_output() {
    let template = format!("{{{{! dmx output=lib/models/product.dart }}}}\n{TEMPLATE}");
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", &template),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("build"),
                Outcome::Updated
            );
            assert!(!directory.join("lib").join("product.dart").exists());
            let generated =
                fs::read_to_string(directory.join("lib").join("models").join("product.dart"))
                    .expect("output");
            assert!(generated.contains("final class Product {"), "{generated}");
            assert!(!generated.contains("output="), "{generated}");
        },
    );
}

/// [typediagram.canonical]: a definition with nothing beside it renders
/// through the canonical model template, and the class it writes is a value —
/// `==`, `hashCode`, `toString`, `copyWith` — with its JSON on an extension
/// rather than on the class.
#[test]
fn a_definition_alone_renders_through_the_canonical_template() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("build"),
                Outcome::Updated
            );
            let generated =
                fs::read_to_string(directory.join("lib").join("product.dart")).expect("output");
            assert!(
                generated.contains("// dmx: rendered through the canonical model template, "),
                "{generated}"
            );
            for expected in [
                "final class Product {",
                "bool operator ==(Object other) =>",
                "int get hashCode => Object.hash(",
                "String toString() => 'Product(id: $id, name: $name)';",
                "Product copyWith({",
                "extension ProductJson on Product {",
                "static dmx.Result<Product, dmx.DecodeError> fromJson(",
                "Map<String, Object?> toJson() => <String, Object?>{",
            ] {
                assert!(
                    generated.contains(expected),
                    "missing `{expected}`:\n{generated}"
                );
            }
            let class = generated
                .split_once("final class Product {")
                .and_then(|(_, rest)| rest.split_once("\n}\n"))
                .map(|(body, _)| body.to_owned())
                .expect("the class body");
            assert!(
                !class.contains("Json"),
                "JSON reached the class body:\n{class}"
            );
            let report = super::explain(&definition).expect("explain");
            assert!(report.contains("the canonical model template"), "{report}");
        },
    );
}

/// [typediagram.canonical]: a `<name>.mustache` beside the definition takes the
/// canonical template's place rather than adding a second output.
#[test]
fn a_template_of_its_own_replaces_the_canonical_one() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            let _ = super::process(&definition, &roots(), &build()).expect("build");
            let generated =
                fs::read_to_string(directory.join("lib").join("product.dart")).expect("output");
            assert!(
                generated.contains("rendered through models/product.mustache"),
                "{generated}"
            );
            assert!(!generated.contains("operator =="), "{generated}");
            let report = super::explain(&definition).expect("explain");
            assert!(report.contains("1 generation group(s)"), "{report}");
            assert!(!report.contains("the canonical model template"), "{report}");
        },
    );
}

/// [typediagram.standalone]: a template whose name matches two definitions
/// binds to the longer one, and the shorter one never claims it.
#[test]
fn a_template_binds_to_the_most_specific_definition() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.wire.td", "type Wire {\n  at: DateTime\n}\n"),
            ("models/product.mustache", TEMPLATE),
            ("models/product.wire.mustache", TEMPLATE),
        ],
        |directory| {
            let models = directory.join("models");
            assert_eq!(
                definition_of(&models.join("product.wire.mustache")),
                Some(models.join("product.wire.td"))
            );
            assert_eq!(
                definition_of(&models.join("product.mustache")),
                Some(models.join("product.td"))
            );

            for name in ["product.td", "product.wire.td"] {
                assert_eq!(
                    super::process(&models.join(name), &roots(), &build()).expect(name),
                    Outcome::Updated
                );
            }
            let wire = fs::read_to_string(directory.join("lib").join("product_wire.dart"))
                .expect("wire output");
            assert!(wire.contains("final class Wire {"), "{wire}");
            assert!(!wire.contains("final class Product {"), "{wire}");
        },
    );
}

/// [typediagram.standalone]: a removed template takes its output with it.
///
/// The template removed here is the second one. Removing the *first* would
/// hand `lib/product.dart` back to the canonical template rather than collect
/// it [typediagram.canonical] — a definition always renders.
#[test]
fn a_removed_template_collects_its_output() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
            ("models/product.wire.mustache", TEMPLATE),
        ],
        |directory| {
            let definition = directory.join("models").join("product.td");
            let _ = super::process(&definition, &roots(), &build()).expect("build");
            assert!(directory.join("lib").join("product_wire.dart").is_file());

            fs::remove_file(directory.join("models").join("product.wire.mustache"))
                .expect("remove template");
            assert_eq!(
                super::process(&definition, &roots(), &build()).expect("second build"),
                Outcome::Updated
            );
            assert!(
                !directory.join("lib").join("product_wire.dart").exists(),
                "a dropped template means a dropped file"
            );
            assert!(
                directory.join("lib").join("product.dart").is_file(),
                "the first template's output is untouched"
            );
        },
    );
}

/// [typediagram.standalone]: an output that exists without dmx's marker is a
/// hand-written file and is never overwritten.
#[test]
fn a_hand_written_output_is_refused() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
            ("lib/product.dart", "// mine\n"),
        ],
        |directory| {
            let error = format!(
                "{:#}",
                super::process(
                    &directory.join("models").join("product.td"),
                    &roots(),
                    &build()
                )
                .expect_err("hand-written file")
            );
            assert!(error.contains("DMX8006"), "{error}");
            assert_eq!(
                fs::read_to_string(directory.join("lib").join("product.dart")).expect("untouched"),
                "// mine\n"
            );
        },
    );
}

/// [typediagram.diagnostics]: a fault in a definition file is reported at the
/// line the author's editor shows, with no fence anywhere in the sentence.
#[test]
fn a_definition_fault_is_reported_in_file_lines() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", "type A { x: Int }\ntype B { y }\n"),
            ("models/product.mustache", TEMPLATE),
        ],
        |directory| {
            let error = format!(
                "{:#}",
                super::process(
                    &directory.join("models").join("product.td"),
                    &roots(),
                    &build()
                )
                .expect_err("bad definition")
            );
            assert!(error.contains("DMX8004"), "{error}");
            assert!(
                error.contains("in models/product.td is not valid"),
                "{error}"
            );
            assert!(error.contains("line 2, column 12"), "{error}");
            assert!(!error.contains("fence"), "{error}");
        },
    );
}

/// [typediagram.execution]: `dmx explain` names the files rather than fences,
/// and prints the exact context a template author will place.
#[test]
fn explain_names_the_files() {
    in_workspace(
        &[
            ("pubspec.yaml", "name: fixture\n"),
            ("models/product.td", DEFINITION),
            ("models/product.mustache", TEMPLATE),
        ],
        |directory| {
            let report =
                super::explain(&directory.join("models").join("product.td")).expect("explain");
            assert!(
                report.contains("models/product.td: 1 generation group(s)"),
                "{report}"
            );
            assert!(
                report.contains(
                    "-> lib/product.dart (target dart, template models/product.mustache, digest "
                ),
                "{report}"
            );
            assert!(report.contains("group 1 — the definition file"), "{report}");
            assert!(
                report.contains("\"template\": \"models/product.mustache\""),
                "{report}"
            );
            assert!(report.contains("\"dartType\": \"String\""), "{report}");
            assert!(!directory.join("lib").exists());
        },
    );
}

/// [typediagram.standalone]: the two predicates and the name convention, on
/// the spellings a real tree contains.
#[test]
fn the_conventions_are_what_they_say_they_are() {
    for (path, definition, template) in [
        ("models/a.td", true, false),
        ("models/a.TD", true, false),
        ("models/a.mustache", false, true),
        ("models/a.wire.mustache", false, true),
        ("lib/a.dart", false, false),
        ("docs/a.dmx.md", false, false),
    ] {
        assert_eq!(is_definition(Path::new(path)), definition, "{path}");
        assert_eq!(is_template(Path::new(path)), template, "{path}");
    }
    for (template, name) in [
        ("models/product.mustache", "product"),
        ("models/product.wire.mustache", "product_wire"),
        ("models/Product.WireNames.mustache", "product_wire_names"),
    ] {
        assert_eq!(
            output_name(Path::new(template)).expect(template),
            name,
            "{template}"
        );
    }
    // A Mustache file with no definition beside it is somebody else's — the
    // catalogue's previews are exactly that — and binds to nothing.
    assert_eq!(definition_of(Path::new("templates/model.mustache")), None);
    assert_eq!(definition_of(Path::new("models/a.td")), None);
}
