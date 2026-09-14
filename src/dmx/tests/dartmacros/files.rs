//! Whole-file outputs authored by Dart macros [dartmacros.files].

use super::*;

const SEED: &str = "@dmx('tables')\nclass Schema {\n}\n";
const MANIFEST_EXPAND: &str =
    "String expand(Map<String, Object?> invocation) =>\n    '  static const int tables = 2;\\n';";
pub(super) const SEED_MARKER: &str = "// dmx: generated from schema.dart — do not edit.";

pub(super) fn files_project(files: &str) -> TempDirectory {
    project(
        &worker_with_files(&["tables"], MANIFEST_EXPAND, files),
        "schema.dart",
        SEED,
    )
}

/// [dartmacros.files]: safe package-relative output paths are owned by the
/// Dart macro, so one seed can produce one ordinary public library file.
#[test]
fn a_macro_authors_a_package_relative_file() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'lib/src/generated/models.dart', 'text': 'final class Models {}\\n'}];",
    );
    let _ = dir
        .write("pubspec.yaml", "name: fixture\n")
        .expect("pubspec");

    let _ = build_and_read(&dir, "schema.dart");
    let output = fs::read_to_string(dir.path.join("lib/src/generated/models.dart"))
        .expect("package-relative output");
    assert!(output.starts_with("// dmx: generated from lib/schema.dart — do not edit.\n\n"));
    assert!(output.contains("final class Models {}"));
}

/// [dartmacros.files]: package-relative outputs participate in the same
/// ownership cleanup as siblings when a later macro pass drops them.
#[test]
fn stale_package_relative_files_are_collected() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'lib/src/generated/models.dart', 'text': 'final class Models {}\\n'}];",
    );
    let _ = dir
        .write("pubspec.yaml", "name: fixture\n")
        .expect("pubspec");
    let _ = build_and_read(&dir, "schema.dart");
    assert!(dir.path.join("lib/src/generated/models.dart").is_file());

    let worker = worker_with_files(&["tables"], MANIFEST_EXPAND, NO_FILES);
    let _ = dir.write("tool/dmx/macros.dart", &worker).expect("worker");
    let _ = build_and_read(&dir, "schema.dart");
    assert!(!dir.path.join("lib/src/generated/models.dart").exists());
}

/// [dartmacros.files]: whole sibling files carry markers and settle after one
/// generation pass.
#[test]
fn a_macro_authors_whole_sibling_files() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) => [
           {'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'},
           {'name': 'order_row.dart', 'text': 'final class OrderRow {\\n  const OrderRow();\\n}\\n'},
         ];",
    );

    let seed = build_and_read(&dir, "schema.dart");
    assert!(seed.contains("static const int tables = 2;"));
    for (name, class) in [
        ("customer_row.dart", "final class CustomerRow {"),
        ("order_row.dart", "final class OrderRow {"),
    ] {
        let sibling = fs::read_to_string(dir.path.join("lib").join(name)).expect("sibling");
        assert!(sibling.starts_with(&format!("{SEED_MARKER}\n\n")));
        assert!(sibling.contains(class), "`{name}` must hold its class");
    }

    let second = dmx(&dir, &["build", "lib", "--insert-regions"]);
    assert!(
        String::from_utf8_lossy(&second.stdout).contains("0 of 3 file(s) updated"),
        "an up-to-date pass must write nothing"
    );
}

/// [dartmacros.files]: stale owned siblings are collected while human files
/// remain untouched.
#[test]
fn stale_macro_files_are_collected_and_hand_written_ones_kept() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'}];",
    );
    let stale = format!("{SEED_MARKER}\n\nfinal class DroppedRow {{\n  const DroppedRow();\n}}\n");
    let _ = dir.write("lib/dropped_row.dart", &stale).expect("stale");
    let hand = "class Hand {\n  const Hand();\n}\n";
    let _ = dir.write("lib/hand.dart", hand).expect("hand");

    let _ = build_and_read(&dir, "schema.dart");
    assert!(!dir.path.join("lib/dropped_row.dart").exists());
    assert_eq!(
        fs::read_to_string(dir.path.join("lib/hand.dart")).expect("hand kept"),
        hand
    );
}

/// [dartmacros.files]: overwrites, traversal, invalid Dart, and duplicate
/// claims all fail with nothing written.
#[test]
fn dangerous_macro_files_are_refused() {
    let overwrite = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'}];",
    );
    let hand = "class CustomerRow {\n  const CustomerRow();\n}\n";
    let _ = overwrite
        .write("lib/customer_row.dart", hand)
        .expect("hand");
    let refused = dmx(&overwrite, &["build", "lib", "--insert-regions"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("DMX7008"));
    assert_eq!(
        fs::read_to_string(overwrite.path.join("lib/customer_row.dart")).expect("kept"),
        hand
    );

    for (files, code) in [
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) =>
               [{'name': '../escape.dart', 'text': 'class Escape {}\\n'}];",
            "DMX7007",
        ),
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) =>
               [{'name': '/tmp/escape.dart', 'text': 'class Escape {}\\n'}];",
            "DMX7007",
        ),
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) =>
               [{'name': 'lib/../escape.dart', 'text': 'class Escape {}\\n'}];",
            "DMX7007",
        ),
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) =>
               [{'name': 'broken_row.dart', 'text': 'final class {\\n'}];",
            "macro-authored file",
        ),
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) => [
               {'name': 'twice_row.dart', 'text': 'final class TwiceRow {}\\n'},
               {'name': 'twice_row.dart', 'text': 'final class TwiceRow {}\\n'},
             ];",
            "DMX7008",
        ),
    ] {
        let dir = files_project(files);
        let output = dmx(&dir, &["build", "lib", "--insert-regions"]);
        assert!(
            !output.status.success(),
            "the reply must be refused: {code}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "diagnostic `{code}` missing:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for name in ["escape.dart", "broken_row.dart", "twice_row.dart"] {
            assert!(!dir.path.join(name).exists() && !dir.path.join("lib").join(name).exists());
        }
    }
}

/// [dartmacros.files] + [execution]: check reports drift without writing and
/// accepts a generated tree.
#[test]
fn check_reports_sibling_drift_without_writing() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'}];",
    );

    let drift = dmx(&dir, &["build", "lib", "--insert-regions", "--check"]);
    assert_eq!(drift.status.code(), Some(2), "missing siblings are drift");
    assert!(!dir.path.join("lib/customer_row.dart").exists());

    let _ = build_and_read(&dir, "schema.dart");
    let clean = dmx(&dir, &["build", "lib", "--insert-regions", "--check"]);
    assert_eq!(
        clean.status.code(),
        Some(0),
        "a generated tree must pass `--check`:\n{}",
        String::from_utf8_lossy(&clean.stderr)
    );
}
