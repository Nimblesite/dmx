//! E2E: user-defined macros in Dart [dartmacros].
//!
//! Black-box over the real binary and a real `tool/dmx/macros.dart` worker
//! [dartmacros.discovery]: dmx walks the CST, ships the invocation to the
//! Dart process, and splices what comes back through the ordinary pipeline
//! [dartmacros.pipeline]. The workers here are self-contained Dart scripts, so
//! the suite needs `dart` and nothing from pub. The `SQLite` case is the
//! point of the whole feature: the macro reads the declaration AND a live
//! database schema — computation no built-in could hard-code.

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
// A separate file only because this one is near the 500-line ceiling; the
// watch suite needs these fixtures, so it is a module rather than a binary.
#[path = "dartmacros/watch.rs"]
mod watch;
// Likewise: the render suite [dartmacros.render] shares these fixtures.
#[path = "dartmacros/render.rs"]
mod render;

use std::fs;
use std::process::{Command, Output};

use support::TempDirectory;

/// The protocol scaffold every fixture worker shares; only the `expand`
/// function differs. Kept as one template so the tests exercise the protocol
/// dmx speaks, not fifteen re-typings of it.
const WORKER_SCAFFOLD: &str = r"
import 'dart:convert';
import 'dart:io';

__EXPAND__

__FILES__

Future<void> main() async {
  final frames = stdin.transform(utf8.decoder).transform(const LineSplitter());
  await for (final frame in frames) {
    final Object? message = jsonDecode(frame);
    if (message is! Map<String, Object?>) {
      continue;
    }
    if (message['op'] == 'hello') {
      stdout.writeln(jsonEncode({
        'v': 1,
        'name': 'fixture',
        'version': '0.0.0',
        'contextVersion': 1,
        'ops': ['expand'],
        'macros': [__MACROS__],
      }));
      continue;
    }
    if (message['op'] == 'expand') {
      final Object? invocation = message['invocation'];
      final text =
          invocation is Map<String, Object?> ? expand(invocation) : '';
      final authored = invocation is Map<String, Object?>
          ? files(invocation)
          : <Map<String, String>>[];
      stdout.writeln(jsonEncode({
        'v': 1,
        'id': message['id'],
        'text': text,
        'introduced': <String>[],
        'files': authored,
        'diagnostics': <Object>[],
      }));
    }
  }
}
";

/// The `files` hook of a worker that authors none [dartmacros.files].
const NO_FILES: &str =
    "List<Map<String, String>> files(Map<String, Object?> invocation) => const [];";

/// A worker serving `names` that also authors whole files [dartmacros.files].
fn worker_with_files(names: &[&str], expand: &str, files: &str) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("'{n}'")).collect();
    WORKER_SCAFFOLD
        .replace("__EXPAND__", expand)
        .replace("__FILES__", files)
        .replace("__MACROS__", &quoted.join(", "))
}

/// A worker source serving `names`, expanding via the Dart `expand` function.
fn worker(names: &[&str], expand: &str) -> String {
    worker_with_files(names, expand, NO_FILES)
}

/// A project directory holding `lib/` sources and no worker at all.
fn project_without_worker(lib_file: &str, source: &str) -> TempDirectory {
    let dir = TempDirectory::create("dmx-dartmacros").expect("temp dir");
    fs::create_dir_all(dir.path.join("lib")).expect("lib");
    let _ = dir
        .write(&format!("lib/{lib_file}"), source)
        .expect("source");
    dir
}

/// The same, plus the conventional worker at `tool/dmx/macros.dart`.
fn project(worker_source: &str, lib_file: &str, source: &str) -> TempDirectory {
    let dir = project_without_worker(lib_file, source);
    fs::create_dir_all(dir.path.join("tool/dmx")).expect("tool/dmx");
    let _ = dir
        .write("tool/dmx/macros.dart", worker_source)
        .expect("worker");
    dir
}

/// The binary, run from the project so worker discovery is the real path
/// lookup [dartmacros.discovery].
fn dmx(dir: &TempDirectory, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dmx"))
        .args(args)
        .current_dir(&dir.path)
        .output()
        .expect("run dmx")
}

/// One successful `build --insert-regions` pass, returning the generated file.
fn build_and_read(dir: &TempDirectory, lib_file: &str) -> String {
    let output = dmx(dir, &["build", "lib", "--insert-regions"]);
    assert!(
        output.status.success(),
        "build failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    fs::read_to_string(dir.path.join("lib").join(lib_file)).expect("read output")
}

/// [dartmacros.protocol]: a Dart-defined macro reads the declaration's fields
/// off the CST and generates members dmx splices into the region.
#[test]
fn dart_macro_expands_from_the_declarations_fields() {
    let expand = r#"
String expand(Map<String, Object?> invocation) {
  final Object? declaration = invocation['declaration'];
  if (declaration is! Map<String, Object?>) {
    return '';
  }
  final entries = <String>[];
  final Object? fields = declaration['fields'];
  if (fields is List<Object?>) {
    for (final field in fields) {
      if (field is Map<String, Object?>) {
        final Object? name = field['name'];
        if (name is String) {
          entries.add("'$name': $name");
        }
      }
    }
  }
  return "  Map<String, Object?> get auditEntry => {${entries.join(', ')}};\n";
}
"#;
    let source = "@dmx('audit')\n\
                  class Order {\n\
                  \x20 final String id;\n\
                  \x20 final int total;\n\
                  \x20 const Order({required this.id, required this.total});\n\
                  }\n";
    let dir = project(&worker(&["audit"], expand), "order.dart", source);

    let generated = build_and_read(&dir, "order.dart");
    assert!(
        generated.contains("Map<String, Object?> get auditEntry => {'id': id, 'total': total};"),
        "generated members must come from the Dart macro:\n{generated}"
    );
    assert!(
        generated.contains("//#region"),
        "the fragment must land inside a machine-owned region:\n{generated}"
    );
}

/// [dartmacros.api]: the macro combines the CST with something external — a
/// live `SQLite` database — generating column constants from the real schema.
/// This is the capability no built-in can hard-code.
#[test]
fn dart_macro_reads_a_live_sqlite_schema() {
    let expand = r#"
String stringArg(Map<String, Object?> invocation, String label) {
  final Object? args = invocation['args'];
  if (args is! Map<String, Object?>) {
    return '';
  }
  final Object? raw = args[label];
  if (raw is! String) {
    return '';
  }
  final source = raw.trim();
  final quoted = (source.startsWith("'") && source.endsWith("'")) ||
      (source.startsWith('"') && source.endsWith('"'));
  return quoted && source.length >= 2
      ? source.substring(1, source.length - 1)
      : source;
}

String expand(Map<String, Object?> invocation) {
  final table = stringArg(invocation, 'table');
  final db = stringArg(invocation, 'db');
  final result = Process.runSync(
      'sqlite3', ['-json', db, "PRAGMA table_info('$table')"]);
  final Object? raw = result.stdout;
  final Object? rows = jsonDecode(raw is String ? raw : '[]');
  final columns = <String>[];
  if (rows is List<Object?>) {
    for (final row in rows) {
      if (row is Map<String, Object?>) {
        final Object? column = row['name'];
        if (column is String) {
          columns.add("'$column'");
        }
      }
    }
  }
  return "  static const List<String> columns = [${columns.join(', ')}];\n";
}
"#;
    let source = "@dmx('sqliteSchema', {'table': 'products', 'db': 'app.db'})\n\
                  class ProductRow {\n\
                  \x20 final String id;\n\
                  \x20 const ProductRow({required this.id});\n\
                  }\n";
    let dir = project(
        &worker(&["sqliteSchema"], expand),
        "product_row.dart",
        source,
    );
    let schema = Command::new("sqlite3")
        .arg(dir.path.join("app.db"))
        .arg(
            "CREATE TABLE products (\
               id TEXT NOT NULL PRIMARY KEY, \
               title TEXT NOT NULL, \
               price_cents INTEGER NOT NULL);",
        )
        .output()
        .expect("create schema");
    assert!(
        schema.status.success(),
        "sqlite3 must create the fixture db"
    );

    let generated = build_and_read(&dir, "product_row.dart");
    assert!(
        generated.contains("static const List<String> columns = ['id', 'title', 'price_cents'];"),
        "columns must mirror the database's actual schema:\n{generated}"
    );
}

/// [dartmacros.resolution]: a worker declaring a built-in's name fails loudly
/// with `DMX7005` — upgrading dmx can never silently change whose code
/// generates.
#[test]
fn dart_macro_shadowing_a_builtin_is_refused() {
    let expand = "String expand(Map<String, Object?> invocation) => '';\n";
    let source = "@dmx('anything')\n\
                  class Order {\n\
                  \x20 final String id;\n\
                  \x20 const Order({required this.id});\n\
                  }\n";
    let dir = project(&worker(&["model"], expand), "order.dart", source);

    let output = dmx(&dir, &["build", "lib", "--insert-regions"]);
    assert!(
        !output.status.success(),
        "shadowing a built-in must fail the build"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("DMX7005") && stderr.contains("model"),
        "the diagnostic must name the collision:\n{stderr}"
    );
}

/// [dartmacros.resolution]: without a worker, an unregistered `@dmx` name
/// stays inert and the build succeeds untouched — the default path never
/// spawns a Dart process.
#[test]
fn without_a_worker_an_unknown_macro_stays_inert() {
    let source = "@dmx('audit')\n\
                  class Order {\n\
                  \x20 final String id;\n\
                  \x20 const Order({required this.id});\n\
                  }\n";
    let dir = project_without_worker("order.dart", source);

    let output = dmx(&dir, &["build", "lib", "--insert-regions"]);
    assert!(
        output.status.success(),
        "an unserved name must not fail the build:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let untouched = fs::read_to_string(dir.path.join("lib/order.dart")).expect("read output");
    assert_eq!(untouched, source, "the file must be byte-identical");
}

/// A seed class whose macro authors sibling files [dartmacros.files].
const SEED: &str = "@dmx('tables')\nclass Schema {\n}\n";

/// An expand hook returning one manifest line for the seed's region.
const MANIFEST_EXPAND: &str =
    "String expand(Map<String, Object?> invocation) =>\n    '  static const int tables = 2;\\n';";

/// A project whose worker authors `files` beside the seed [dartmacros.files].
fn files_project(files: &str) -> TempDirectory {
    project(
        &worker_with_files(&["tables"], MANIFEST_EXPAND, files),
        "schema.dart",
        SEED,
    )
}

/// The exact ownership marker files generated from the seed carry
/// [dartmacros.files].
const SEED_MARKER: &str = "// dmx: generated from schema.dart — do not edit.";

/// [dartmacros.files]: one annotation, and the macro authors whole sibling
/// files it names itself — marker line prepended, content validated, and a
/// second pass writing nothing.
#[test]
fn a_macro_authors_whole_sibling_files() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) => [
           {'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'},
           {'name': 'order_row.dart', 'text': 'final class OrderRow {\\n  const OrderRow();\\n}\\n'},
         ];",
    );

    let seed = build_and_read(&dir, "schema.dart");
    assert!(
        seed.contains("static const int tables = 2;"),
        "the seed's own region must still fill:\n{seed}"
    );
    for (name, class) in [
        ("customer_row.dart", "final class CustomerRow {"),
        ("order_row.dart", "final class OrderRow {"),
    ] {
        let sibling = fs::read_to_string(dir.path.join("lib").join(name)).expect("sibling");
        assert!(
            sibling.starts_with(&format!("{SEED_MARKER}\n\n")),
            "`{name}` must open with the ownership marker:\n{sibling}"
        );
        assert!(sibling.contains(class), "`{name}` must hold its class");
    }

    let second = dmx(&dir, &["build", "lib", "--insert-regions"]);
    assert!(
        String::from_utf8_lossy(&second.stdout).contains("0 of 3 file(s) updated"),
        "an up-to-date pass must write nothing [emission.inline-backend.no-op-writes]"
    );
}

/// [dartmacros.files]: a file this seed wrote before and no longer produces is
/// collected — a dropped table means a dropped file — while a hand-written
/// neighbour without the marker is untouchable.
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
    assert!(
        !dir.path.join("lib/dropped_row.dart").exists(),
        "a sibling this pass no longer produces must be collected"
    );
    assert_eq!(
        fs::read_to_string(dir.path.join("lib/hand.dart")).expect("hand kept"),
        hand,
        "an unmarked neighbour is not dmx's to touch"
    );
}

/// [dartmacros.files]: the refusals — a name that would overwrite a
/// hand-written file, a path-shaped name, an unparseable file, and two claims
/// on one name all fail the build with nothing written.
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
    assert!(
        !refused.status.success(),
        "overwriting a human's file must fail"
    );
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("DMX7008"),
        "the refusal must carry its code"
    );
    assert_eq!(
        fs::read_to_string(overwrite.path.join("lib/customer_row.dart")).expect("kept"),
        hand,
        "the hand-written file must survive byte-identically"
    );

    for (files, code) in [
        (
            "List<Map<String, String>> files(Map<String, Object?> invocation) =>
               [{'name': '../escape.dart', 'text': 'class Escape {}\\n'}];",
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
            assert!(
                !dir.path.join(name).exists() && !dir.path.join("lib").join(name).exists(),
                "a refused pass must write nothing"
            );
        }
    }
}

/// [dartmacros.files] + [execution]: `--check` reports sibling drift on the
/// seed without writing, and a generated tree passes it clean.
#[test]
fn check_reports_sibling_drift_without_writing() {
    let dir = files_project(
        "List<Map<String, String>> files(Map<String, Object?> invocation) =>
           [{'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'}];",
    );

    let drift = dmx(&dir, &["build", "lib", "--insert-regions", "--check"]);
    assert_eq!(drift.status.code(), Some(2), "missing siblings are drift");
    assert!(
        !dir.path.join("lib/customer_row.dart").exists(),
        "`--check` must not write the sibling"
    );

    let _ = build_and_read(&dir, "schema.dart");
    let clean = dmx(&dir, &["build", "lib", "--insert-regions", "--check"]);
    assert_eq!(
        clean.status.code(),
        Some(0),
        "a generated tree must pass `--check`:\n{}",
        String::from_utf8_lossy(&clean.stderr)
    );
}
