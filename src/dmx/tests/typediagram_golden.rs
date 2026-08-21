//! Every corpus definition, rendered to Dart by the shipped binary [typediagram.output].
//!
//! `tests/typediagram/corpus/*.td` is the parity corpus: `typediagram_model`
//! holds the Rust parser to typeDiagram's own model, fixture by fixture. That
//! proves the *model* is right and says nothing about the *code*, and this repo
//! holds that emitting Dart which does not compile is the worst failure
//! available to it.
//!
//! So the whole corpus is laid out as standalone `models/<name>.td` files
//! [typediagram.standalone] with nothing beside them, run through the real
//! `dmx` binary in one `dmx build`, and compared byte for byte with
//! `tests/typediagram/golden/lib/<name>.dart`. Those files are committed, and
//! `make corpus` runs `dart analyze --fatal-infos` over the package holding
//! them — so the corpus is checked as source, not just as JSON.
//!
//! Nothing is wrapped, assembled, or extracted, and no template is written
//! here at all: the `.td` files are copied out of the parity corpus byte for
//! byte and render through the canonical model template dmx ships
//! [typediagram.canonical]. That makes this suite the canonical template's
//! own gate — every shape typeDiagram can express, held to Dart the analyzer
//! accepts.
//!
//! Hygiene is not re-asserted here. The binary refuses to write source
//! carrying `throw`, an `as` cast or a `!` assertion at all, and
//! `typediagram_cli` proves that refusal (DMX4003) against a template written
//! to trip it. A substring scan of the goldens could only re-check it worse —
//! `as` occurs in English — so the check lives where it can be made properly.
//!
//! To accept a deliberate change to the emitted shape:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test --test typediagram_golden
//! ```

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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use support::TempDirectory;

/// Every fixture the parity corpus carries, in the order a reader meets them.
const FIXTURES: &[&str] = &[
    "scalars",
    "records",
    "unions",
    "aliases-and-functions",
    "targeting",
];

/// The version token the goldens are written with.
///
/// A release build injects `DMX_VERSION`, so the marker's version field is the
/// one thing in the file that is not a function of the fixture. Both sides are
/// normalised to this before comparing; that the field carries the running
/// build's version is asserted by the `typediagram_cli` suite, which reads it
/// out of a marker directly.
const PINNED_VERSION: &str = "0.0.0";

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/typediagram/golden")
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/typediagram/corpus")
}

/// The Dart file name a fixture generates, which is its own name spelled the
/// way Dart spells a source file [typediagram.standalone].
fn dart_name(name: &str) -> String {
    name.replace('-', "_")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// The same bytes with the running build's version replaced by the pinned one.
fn normalised(source: &str) -> String {
    source.replace(
        &format!("dmx {}.", dmx::VERSION),
        &format!("dmx {PINNED_VERSION}."),
    )
}

/// Runs the binary over a throwaway package holding the whole corpus as
/// standalone definitions, and returns the Dart it wrote for each fixture.
///
/// One package and one invocation: every `.td` is found by the same recursive
/// sweep a real project gets, and every one of them renders through the
/// canonical model template, because nothing sits beside it
/// [typediagram.canonical].
fn generate() -> BTreeMap<&'static str, String> {
    let workspace = TempDirectory::create("dmx-td-golden").expect("scratch directory");
    let _ = workspace
        .write("pubspec.yaml", &read(&golden_dir().join("pubspec.yaml")))
        .expect("pubspec");
    for name in FIXTURES {
        let _ = workspace
            .write(
                &format!("models/{name}.td"),
                &read(&corpus_dir().join(format!("{name}.td"))),
            )
            .expect("definition");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_dmx"))
        .args(["build", "models", "lib"])
        .current_dir(&workspace.path)
        .output()
        .expect("run dmx");
    assert!(
        output.status.success(),
        "dmx build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    FIXTURES
        .iter()
        .map(|name| {
            let written = workspace.at(&format!("lib/{}.dart", dart_name(name)));
            assert!(
                written.exists(),
                "{name}: nothing was written to lib/{}.dart\nstdout:\n{}",
                dart_name(name),
                String::from_utf8_lossy(&output.stdout)
            );
            (*name, normalised(&read(&written)))
        })
        .collect()
}

/// [typediagram.output]: every corpus definition renders to the committed Dart,
/// byte for byte, through the shipped binary.
#[test]
fn every_corpus_fixture_generates_its_golden_dart() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();

    for (name, actual) in generate() {
        let file = format!("lib/{}.dart", dart_name(name));
        let expected_path = golden_dir().join(&file);

        if updating {
            fs::write(&expected_path, &actual).expect("write golden");
            continue;
        }

        let expected = read(&expected_path);
        assert_eq!(
            actual, expected,
            "{name}: generated Dart no longer matches tests/typediagram/golden/{file}. \
             Re-run with UPDATE_GOLDEN=1 if the change is deliberate."
        );
    }
}

/// [typediagram.output]: the shapes the corpus exists to reach are actually in
/// the generated Dart, so a golden emptied by a template mistake cannot pass.
///
/// The byte comparison above proves the output is *stable*; it cannot notice
/// that a section stopped matching and quietly rendered nothing. These are the
/// constructs no other suite in the repo generates.
/// Every construct the corpus exists to cover: the golden that must carry it,
/// what it is, and the text that proves it survived. A row that stops matching
/// names the construct that went missing rather than a line number.
const CONSTRUCTS: &[(&str, &str, &[&str])] = &[
    (
        "lib/unions.dart",
        "a tuple variant, under a name Dart can compile",
        &["const Triple({required this.value1, required this.value2, required this.value3})"],
    ),
    (
        "lib/unions.dart",
        "a generic union, with its cases parameterised by the union's own list",
        &[
            "final class Some<T> extends Option<T>",
            "final class Err<T, E> extends Result<T, E>",
        ],
    ),
    (
        "lib/unions.dart",
        "explicit discriminants, the digit-separated one included",
        &[
            "static const int discriminant = -32700;",
            "static const int discriminant = 1_000;",
        ],
    ),
    (
        "lib/unions.dart",
        "the untagged union, marked as told apart by shape",
        &["told apart by shape"],
    ),
    (
        "lib/unions.dart",
        "cases named as the diagram names them [typediagram.canonical.names]",
        &[
            "final class Circle extends Shape {",
            "final class Left extends Loose {",
            "final class Number extends RequestId {",
        ],
    ),
    (
        "lib/unions.dart",
        "a colliding case name qualified by its union — `Ok` belongs to two \
         unions here, and `String` is Dart's own",
        &[
            "final class ErrorCodeOk extends ErrorCode {",
            "final class ResultOk<T, E> extends Result<T, E> {",
            "final class RequestIdString extends RequestId {",
        ],
    ),
    (
        "lib/aliases_and_functions.dart",
        "the generic function typedef",
        &["typedef Fetch<T> = Response Function(Request request, T? fallback);"],
    ),
    (
        "lib/aliases_and_functions.dart",
        "overloads, written out one typedef each",
        &[
            "typedef Read0 = List<int> Function(String path);",
            "typedef Read1 = Future<List<int>> Function(String path, double timeout);",
        ],
    ),
    (
        "lib/aliases_and_functions.dart",
        "an async single-signature function, which is a Future",
        &["typedef Store = Future<void> Function(Request item);"],
    ),
    (
        "lib/aliases_and_functions.dart",
        "the generic alias",
        &["typedef Index<K> = Map<K, List<Email>>;"],
    ),
    (
        "lib/scalars.dart",
        "the scalar mapping table, field by field",
        &[
            "final bool flag;",
            "final int count;",
            "final double ratio;",
            "final List<int> blob;",
            "final void nothing;",
            "final DateTime at;",
            "final Object anything;",
            "final Map<String, List<String?>> index;",
            "final Map<Uuid, List<Object>>? deep;",
        ],
    ),
    (
        "lib/scalars.dart",
        "a declaration shadowing a primitive, with the field on the declared name",
        &["typedef Uuid = String;", "final Uuid id;"],
    ),
    (
        "lib/records.dart",
        "an empty record, which takes no parameter list",
        &["const Empty();"],
    ),
    (
        "lib/records.dart",
        "generic records",
        &["final class Pair<A, B> {"],
    ),
    (
        "lib/targeting.dart",
        "every declaration the dart target selects",
        &["class OnlyDartAndRust", "class NotGo", "sealed class Both"],
    ),
];

/// The shapes that must never appear: typeDiagram's positional member names
/// are not Dart identifiers, and a case qualifies only on a collision.
const REFUSED: &[(&str, &str, &str)] = &[
    (
        "lib/unions.dart",
        "this._0",
        "a private member reached Dart",
    ),
    (
        "lib/unions.dart",
        "ShapeCircle",
        "a case was qualified for no reason",
    ),
];

#[test]
fn the_goldens_cover_the_shapes_the_corpus_exists_for() {
    for (file, construct, needles) in CONSTRUCTS {
        let source = read(&golden_dir().join(file));
        for needle in *needles {
            assert!(
                source.contains(needle),
                "{file} lost {construct}: no `{needle}`"
            );
        }
    }

    for (file, refused, why) in REFUSED {
        assert!(
            !read(&golden_dir().join(file)).contains(refused),
            "{file}: {why}"
        );
    }
}

/// [typediagram.canonical]: the classes the canonical template writes are
/// values, and their JSON is beside them rather than in them.
///
/// This is the whole point of there being one model template. A record and a
/// union case are the same kind of thing — an immutable value — so both get
/// `==`, `hashCode`, `toString` and `copyWith`; and neither carries a codec,
/// because a class the diagram described should read as what the diagram said
/// and nothing else.
#[test]
fn every_generated_class_is_a_value_with_its_json_beside_it() {
    let records = read(&golden_dir().join("lib/records.dart"));
    for expected in [
        "bool operator ==(Object other) =>",
        "          dmx.dmxDeepEquals(other.roles, roles) &&",
        "int get hashCode => Object.hash(",
        "        dmx.dmxDeepHash(roles),",
        "String toString() => 'User(id: $id, name: $name, email: $email, roles: $roles, \
         address: $address)';",
        "  User copyWith({",
        "extension UserJson on User {",
        "  static dmx.Result<User, dmx.DecodeError> fromJson(Object? json, [String path = 'User']) =>",
        "  Map<String, Object?> toJson() => <String, Object?>{",
        // The nested decode reaches the *extension*, not the class.
        "AddressJson.fromJson(address, '$path.address')",
    ] {
        assert!(
            records.contains(expected),
            "records.dart is missing `{expected}`"
        );
    }
    for (file, class) in [
        ("lib/records.dart", "final class User {"),
        ("lib/unions.dart", "final class Circle extends Shape {"),
        ("lib/targeting.dart", "final class OnlyDartAndRust {"),
    ] {
        let body = class_body(&read(&golden_dir().join(file)), class);
        assert!(
            !body.contains("Json") && !body.contains("toJson"),
            "{file}: `{class}` carries JSON members:\n{body}"
        );
    }

    // A case decodes by its tag, and the union it belongs to dispatches on one.
    let unions = read(&golden_dir().join("lib/unions.dart"));
    assert!(
        unions.contains("extension ShapeJson on Shape {")
            && unions.contains("'circle' => CircleJson.fromJson(json, path),")
            && unions.contains("            'type': 'circle',"),
        "the union's own codec is missing from unions.dart"
    );
    // The diagram declares `Result`, `Ok` and `Err` itself. A prefixed import
    // is what stops the codec resolving to them [typediagram.canonical].
    assert!(
        unions.contains("import 'package:dmx/dmx.dart' as dmx;")
            && unions.contains("sealed class Result<T, E> {")
            && unions.contains("dmx.Result<Circle, dmx.DecodeError>"),
        "unions.dart does not keep the runtime and the diagram's own names apart"
    );

    // `Unit` is Dart's `void`, which is not a value: it takes part in no
    // comparison, no `toString`, no `copyWith`, and no codec.
    let scalars = read(&golden_dir().join("lib/scalars.dart"));
    assert!(
        scalars.contains("final void nothing;")
            && !scalars.contains("nothing: $nothing")
            && !scalars.contains("extension"),
        "scalars.dart tried to give `void` value semantics"
    );

    // A generic declaration has no codec, because a codec for `T` is not known
    // until `T` is. It is still a value.
    assert!(
        records.contains("(other is Pair<A, B> &&") && !records.contains("extension PairJson"),
        "records.dart got the generic case wrong"
    );
}

/// The text between a class header and the brace that closes it at column
/// zero, which is what generated Dart puts there.
fn class_body(source: &str, header: &str) -> String {
    source
        .split_once(header)
        .and_then(|(_, rest)| rest.split_once("\n}\n"))
        .map_or_else(
            || panic!("no class body for `{header}`"),
            |(body, _)| body.to_owned(),
        )
}

/// [typediagram.output]: every generated file carries the ownership marker the
/// emitter refuses to overwrite without.
#[test]
fn every_golden_is_marked_as_generated() {
    for name in FIXTURES {
        let source = read(&golden_dir().join(format!("lib/{}.dart", dart_name(name))));
        let mut lines = source.lines();
        assert_eq!(
            lines.next().unwrap_or_default(),
            format!("// dmx: generated from models/{name}.td — do not edit.")
        );
        let identity = lines.next().unwrap_or_default();
        assert!(
            identity
                .starts_with("// dmx: rendered through the canonical model template, definition "),
            "{name}: the identity line does not name the template: {identity}"
        );
        assert!(
            identity.contains("context v1"),
            "{name}: the identity line is missing the context version"
        );
    }
}
