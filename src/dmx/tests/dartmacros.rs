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
// Whole-file emission has its own module so this integration target stays
// below the repository's 500-line ceiling [dartmacros.files].
#[path = "dartmacros/files.rs"]
mod files;

use files::{SEED_MARKER, files_project};

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
