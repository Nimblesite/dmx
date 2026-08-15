/// The project's macro worker [dartmacros.discovery].
///
/// `dmx build` finds this file by convention, runs it once, and asks it to
/// expand every `@dmx` name it serves. This one serves `sqliteSchema`, which
/// reads a LIVE SQLite database — the one beside this worker — and authors
/// **one whole Dart file per table** [dartmacros.files]:
///
///   - `products` becomes `ProductRow` in `product_row.dart`
///   - `order_lines` becomes `OrderLineRow` in `order_line_row.dart`
///   - the `customer_spend` view becomes a read-only `CustomerSpendRow`
///   - `rows.dart` exports the lot
///
/// The user writes ONE annotated class, with no arguments, and never types a
/// table name, a class name, or a file name: the macro controls the file
/// names, from the tables. Add a table to `tool/dmx/schema.sql` and a file
/// appears; drop the table and dmx collects the file. The annotated class
/// itself receives the manifest — which tables exist, which are views, and
/// where every row class went.
///
/// That combination is the point. No built-in macro can hard-code it, because
/// the answer lives in a database file that only this project has.
library;

import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';

import 'src/columns.dart';
import 'src/database.dart';
import 'src/emit.dart';
import 'src/naming.dart';

/// Generates one row class per table from the live database schema.
final class SqliteSchema extends DmxMacro {
  /// Reads the schema through the `sqlite3` CLI, so the example needs no
  /// native package and no `pub get` beyond `test`.
  const SqliteSchema();

  @override
  String get name => 'sqliteSchema';

  @override
  DmxOutput expand(DmxInvocation invocation) =>
      switch (_database(invocation.stringArg('db'))) {
        Err(:final error) => error,
        Ok(value: final database) => _schema(invocation, database),
      };

  /// Every table and view, read while the database is in front of us.
  DmxOutput _schema(DmxInvocation invocation, String database) {
    final tables = <SchemaTable>[];
    for (final name in tableNames(database)) {
      switch (readTable(database, name)) {
        case null:
          return DmxRefusal('DMX3912', '`$name` has no columns to read.');
        case final SchemaTable table:
          tables.add(table);
      }
    }
    return tables.isEmpty
        ? const DmxRefusal(
            'DMX3913',
            'the database has no tables, so there is nothing to generate. '
                'Run `make example-sqlite` to build it from '
                'tool/dmx/schema.sql.',
          )
        : _files(invocation, tables);
  }

  /// One authored file per table, the barrel, and the seed's manifest.
  DmxOutput _files(DmxInvocation invocation, List<SchemaTable> tables) {
    final files = <DmxGeneratedFile>[];
    for (final table in tables) {
      switch (mapColumns(table)) {
        case Err(:final error):
          return error;
        case Ok(value: final columns):
          files.add(
            DmxGeneratedFile(fileNameFor(table.name), rowFile(table, columns)),
          );
      }
    }
    final seed = manifest(tables);
    return DmxFragment(
      indented(invocation.memberIndent, seed.lines),
      introduced: seed.names,
      files: [...files, DmxGeneratedFile(barrelName, barrelFile(tables))],
    );
  }
}

/// The database to read, without being told where it is.
///
/// The project keeps its database beside this worker, so that is where the
/// macro looks. Two of them is the one case the convention cannot settle, and
/// it says so rather than picking.
Result<String, DmxRefusal> _database(String? override) {
  if (override != null) {
    return File(override).existsSync()
        ? Ok(override)
        : Err(
            DmxRefusal(
              'DMX3911',
              "`@dmx('sqliteSchema', {'db': '$override'})`: no database there. "
                  'Paths are relative to the directory `dmx` runs in.',
            ),
          );
  }
  final directory = Directory.fromUri(Platform.script.resolve('.'));
  final found = [
    for (final entity in directory.listSync())
      if (entity is File && entity.path.endsWith('.db')) entity.path,
  ]..sort();
  return switch (found) {
    [final only] => Ok(only),
    [] => Err(
        DmxRefusal(
          'DMX3911',
          'no database beside `${directory.path}` — run `make example-sqlite` '
              'to build one from tool/dmx/schema.sql.',
        ),
      ),
    _ => Err(
        DmxRefusal(
          'DMX3910',
          '${found.length} databases beside `${directory.path}`, so the '
              "convention cannot pick one. Say which with {'db': '…'}.",
        ),
      ),
  };
}

/// Serves the macro until `dmx` closes the pipe [dartmacros.protocol].
Future<void> main() => dmxServeMacros([const SqliteSchema()]);
