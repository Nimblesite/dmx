/// The live database, read through the `sqlite3` CLI [dartmacros].
///
/// This is the macro's second input, and the one no built-in could ever have:
/// the real tables, their real columns, which of those columns are the primary
/// key and which point at another table. Everything here is a query against a
/// database file that exists on disk right now.
///
/// The CLI is used rather than a native package so the example needs no build
/// step of its own, and `-json` rather than the default output because it is
/// the only mode that keeps an integer an integer and a real a double — which
/// is exactly what generated `fromRow` code matches on.
library;

import 'dart:convert';
import 'dart:io';

/// One column, as the database describes it.
final class SchemaColumn {
  /// The column name, as the schema spells it.
  final String name;

  /// The type word the schema declared, e.g. `INTEGER` or `BOOLEAN`. SQLite
  /// keeps this verbatim, which is what makes `BOOLEAN` a usable signal.
  final String declaredType;

  /// Whether the database requires a value.
  final bool notNull;

  /// Position in the primary key, 1-based, or 0 when this column is not part
  /// of it. A composite key is just a second column with a position.
  final int keyPosition;

  /// The table this column references, when it is a foreign key.
  final String? referencesTable;

  /// The column it references, in that table.
  final String? referencesColumn;

  /// Builds a column description.
  const SchemaColumn({
    required this.name,
    required this.declaredType,
    required this.notNull,
    required this.keyPosition,
    this.referencesTable,
    this.referencesColumn,
  });

  /// Whether this column is part of the primary key.
  bool get isPrimaryKey => keyPosition > 0;

  /// `customers.id`, when this column points at one.
  String? get references => switch ((referencesTable, referencesColumn)) {
        (final String table, final String column) => '$table.$column',
        _ => null,
      };
}

/// One table or view, in schema order.
final class SchemaTable {
  /// The name the database knows it by.
  final String name;

  /// Whether this is a view — readable, but nothing SQLite will insert into.
  final bool isView;

  /// Its columns, in the order the schema declares them.
  final List<SchemaColumn> columns;

  /// Builds a table description.
  const SchemaTable({
    required this.name,
    required this.isView,
    required this.columns,
  });

  /// The primary key columns, in key order. Empty for a view, and for a table
  /// declared without one.
  List<SchemaColumn> get primaryKey {
    final keys = [
      for (final column in columns)
        if (column.isPrimaryKey) column,
    ];
    keys.sort((a, b) => a.keyPosition.compareTo(b.keyPosition));
    return keys;
  }

  /// The foreign key columns, in schema order.
  List<SchemaColumn> get foreignKeys => [
        for (final column in columns)
          if (column.references != null) column,
      ];
}

/// Rows of a query, typed: `-json` keeps integers integers and reals doubles.
///
/// A query that returns nothing prints nothing, so an empty result and a
/// missing table are both the empty list here; the caller distinguishes them.
List<Map<String, Object?>> queryRows(String database, String sql) {
  final probe = Process.runSync('sqlite3', ['-json', database, sql]);
  final Object? raw = probe.stdout;
  if (raw is! String || raw.trim().isEmpty) {
    return const [];
  }
  final Object? rows = jsonDecode(raw);
  return [
    if (rows is List<Object?>)
      for (final row in rows)
        if (row is Map<String, Object?>) row,
  ];
}

/// Every table and view the database has, in name order — the set a class name
/// is resolved against [dartmacros].
List<String> tableNames(String database) => [
      for (final row in queryRows(
        database,
        "SELECT name FROM sqlite_master WHERE type IN ('table', 'view') "
        "AND name NOT LIKE 'sqlite_%' ORDER BY name",
      ))
        if (row['name'] case final String name) name,
    ];

/// One table read in full, or null when the database has no such table.
SchemaTable? readTable(String database, String table) {
  final kind = queryRows(
    database,
    'SELECT type FROM sqlite_master WHERE name = ${_literal(table)}',
  );
  if (kind.isEmpty) {
    return null;
  }
  final parents = _foreignKeys(database, table);
  final columns = [
    for (final row in queryRows(
      database,
      'PRAGMA table_info(${_literal(table)})',
    ))
      if (row['name'] case final String name)
        SchemaColumn(
          name: name,
          declaredType: switch (row['type']) {
            final String declared => declared,
            _ => '',
          },
          notNull: row['notnull'] == 1,
          keyPosition: switch (row['pk']) {
            final int position => position,
            _ => 0,
          },
          referencesTable: parents[name]?.table,
          referencesColumn: parents[name]?.column,
        ),
  ];
  return columns.isEmpty
      ? null
      : SchemaTable(
          name: table,
          isView: kind.first['type'] == 'view',
          columns: columns,
        );
}

/// Which column points where, keyed by the referencing column.
Map<String, ({String table, String column})> _foreignKeys(
  String database,
  String table,
) =>
    {
      for (final row in queryRows(
        database,
        'PRAGMA foreign_key_list(${_literal(table)})',
      ))
        if (row case {'from': final String from, 'table': final String parent})
          from: (
            table: parent,
            column: switch (row['to']) {
              final String to => to,
              // An FK written without a target column means the parent's primary
              // key; SQLite reports that as null.
              _ => 'rowid',
            },
          ),
    };

/// A SQL string literal. Identifiers reach here from `sqlite_master` or from
/// an author's `table:` argument, so the quote doubling is what keeps a name
/// containing a quote from becoming syntax.
String _literal(String value) => "'${value.replaceAll("'", "''")}'";
