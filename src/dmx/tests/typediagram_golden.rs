//! Every corpus definition, rendered to Dart by the shipped binary [typediagram.output].
//!
//! `tests/typediagram/corpus/*.td` is the parity corpus: `typediagram_model`
//! holds the Rust parser to typeDiagram's own model, fixture by fixture. That
//! proves the *model* is right and says nothing about the *code*, and this repo
//! holds that emitting Dart which does not compile is the worst failure
//! available to it.
//!
//! So each fixture is wrapped in a real `*.dmx.md` document over one shared
//! template, run through the real `dmx` binary, and compared byte for byte with
//! `tests/typediagram/golden/lib/<name>.dart`. Those files are committed, and
//! `make corpus` runs `dart analyze --fatal-infos` over the package holding
//! them — so the corpus is checked as source, not just as JSON.
//!
//! The definitions are never copied. The document is assembled from the `.td`
//! file at test time, so the parity corpus stays the one place a definition is
//! written and the two suites can never drift apart.
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

/// The `*.dmx.md` document one fixture is generated from.
///
/// The definition is the `.td` file verbatim and the template is
/// `golden/template.mustache` verbatim, so neither is written twice.
fn document(name: &str, definition: &str, template: &str) -> String {
    format!(
        "# {name}\n\nGenerated from the parity corpus fixture of the same name.\n\n\
         ```typeDiagram\n{definition}```\n\n\
         ```mustache {{\"dmx\":{{\"output\":\"lib/{name}.dart\"}}}}\n{template}```\n"
    )
}

/// Runs the binary over a throwaway package holding one fixture's document and
/// returns the Dart it wrote.
fn generate(name: &str, template: &str) -> String {
    let workspace = TempDirectory::create("dmx-td-golden").expect("scratch directory");
    let _ = workspace
        .write(
            "pubspec.yaml",
            "name: dmx_typediagram_golden\npublish_to: none\nenvironment:\n  sdk: ^3.6.0\n",
        )
        .expect("pubspec");
    let definition = read(&corpus_dir().join(format!("{name}.td")));
    let _ = workspace
        .write(
            &format!("docs/{name}.dmx.md"),
            &document(name, &definition, template),
        )
        .expect("document");

    let output = Command::new(env!("CARGO_BIN_EXE_dmx"))
        .args(["build", "docs", "lib"])
        .current_dir(&workspace.path)
        .output()
        .expect("run dmx");
    assert!(
        output.status.success(),
        "{name}: dmx build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let written = workspace.at(&format!("lib/{name}.dart"));
    assert!(
        written.exists(),
        "{name}: nothing was written to lib/{name}.dart\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    normalised(&read(&written))
}

/// [typediagram.output]: every corpus definition renders to the committed Dart,
/// byte for byte, through the shipped binary.
#[test]
fn every_corpus_fixture_generates_its_golden_dart() {
    let template = read(&golden_dir().join("template.mustache"));
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();

    for name in FIXTURES {
        let actual = generate(name, &template);
        let expected_path = golden_dir().join(format!("lib/{name}.dart"));

        if updating {
            fs::write(&expected_path, &actual).expect("write golden");
            continue;
        }

        let expected = read(&expected_path);
        assert_eq!(
            actual, expected,
            "{name}: generated Dart no longer matches tests/typediagram/golden/lib/{name}.dart. \
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
#[test]
fn the_goldens_cover_the_shapes_the_corpus_exists_for() {
    let unions = read(&golden_dir().join("lib/unions.dart"));
    // A tuple variant, under a name Dart can compile — see
    // `tuple_members_are_named_for_the_target_not_for_the_model`.
    assert!(
        unions.contains("const RequestIdTriple({required this.value1, required this.value2, required this.value3})"),
        "tuple variants missing from unions.dart"
    );
    assert!(!unions.contains("this._0"), "a private member reached Dart");
    // A generic union, with its cases parameterised by the union's own list.
    assert!(
        unions.contains("final class OptionSome<T> extends Option<T>"),
        "generic union cases missing from unions.dart"
    );
    assert!(
        unions.contains("final class ResultErr<T, E> extends Result<T, E>"),
        "multi-parameter generic union cases missing from unions.dart"
    );
    // Explicit discriminants, including the digit-separated one.
    assert!(
        unions.contains("static const int discriminant = -32700;")
            && unions.contains("static const int discriminant = 1_000;"),
        "discriminants missing from unions.dart"
    );
    assert!(
        unions.contains("told apart by shape"),
        "the untagged union is not marked in unions.dart"
    );

    let functions = read(&golden_dir().join("lib/aliases-and-functions.dart"));
    assert!(
        functions.contains("typedef Fetch<T> = Response Function(Request request, T? fallback);"),
        "the generic function typedef is missing"
    );
    assert!(
        functions.contains("typedef Read0 = List<int> Function(String path);")
            && functions.contains(
                "typedef Read1 = Future<List<int>> Function(String path, double timeout);"
            ),
        "overloads are not written out one typedef each"
    );
    assert!(
        functions.contains("typedef Store = Future<void> Function(Request item);"),
        "an async single-signature function is not a Future"
    );
    assert!(
        functions.contains("typedef Index<K> = Map<K, List<Email>>;"),
        "the generic alias is missing"
    );

    let scalars = read(&golden_dir().join("lib/scalars.dart"));
    for expected in [
        "final bool flag;",
        "final int count;",
        "final double ratio;",
        "final List<int> blob;",
        "final void nothing;",
        "final DateTime at;",
        "final Object anything;",
        "final Map<String, List<String?>> index;",
        "final Map<Uuid, List<Object>>? deep;",
    ] {
        assert!(
            scalars.contains(expected),
            "scalars.dart is missing `{expected}`"
        );
    }
    // A declaration shadows a primitive, and the field takes the declared name.
    assert!(
        scalars.contains("typedef Uuid = String;") && scalars.contains("final Uuid id;"),
        "the shadowing alias is not honoured in scalars.dart"
    );

    let records = read(&golden_dir().join("lib/records.dart"));
    assert!(
        records.contains("const Empty();"),
        "an empty record must take no parameter list"
    );
    assert!(
        records.contains("final class Pair<A, B> {"),
        "generic records are missing"
    );

    let targeting = read(&golden_dir().join("lib/targeting.dart"));
    for expected in ["class OnlyDartAndRust", "class NotGo", "sealed class Both"] {
        assert!(
            targeting.contains(expected),
            "targeting.dart dropped `{expected}`, which the dart target selects"
        );
    }
}

/// [typediagram.output]: every generated file carries the ownership marker the
/// emitter refuses to overwrite without.
#[test]
fn every_golden_is_marked_as_generated() {
    for name in FIXTURES {
        let source = read(&golden_dir().join(format!("lib/{name}.dart")));
        let first = source.lines().next().unwrap_or_default();
        assert_eq!(
            first,
            format!("// dmx: generated from docs/{name}.dmx.md — do not edit.")
        );
        assert!(
            source
                .lines()
                .nth(1)
                .unwrap_or_default()
                .contains("context v1"),
            "{name}: the identity line is missing"
        );
    }
}
