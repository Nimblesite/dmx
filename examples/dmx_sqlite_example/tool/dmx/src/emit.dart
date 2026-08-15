/// The Dart the macro writes [dartmacros.pipeline] — whole files of it
/// [dartmacros.files].
///
/// Every member here is a consequence of something in the schema, and the ones
/// the schema does not support are simply absent: no primary key means no
/// keyed SELECT and no key values, a view means no INSERT, and a table with no
/// foreign keys carries no lookups. Nothing is emitted that the database
/// cannot back.
///
/// Each section returns its lines together with the names those lines bind, so
/// the fragment's `introduced` list cannot drift from what it actually
/// declares [hygiene].
library;

import 'columns.dart';
import 'database.dart';
import 'naming.dart';

/// Lines of generated Dart, with the identifiers they declare.
typedef Section = ({List<String> lines, List<String> names});

/// The generated barrel's file name.
const String barrelName = 'rows.dart';

/// The complete file holding one table's row class, marker aside — the
/// driver prepends that [dartmacros.files].
String rowFile(SchemaTable table, List<MappedColumn> columns) {
  final className = classNameFor(table.name);
  final members = membersFor(className, table, columns).lines;
  while (members.isNotEmpty && members.last.isEmpty) {
    members.removeLast();
  }
  return [
    "import 'package:dmx/dmx.dart';",
    '',
    '/// One row of `${table.name}`, exactly as the database '
        '${table.isView ? 'computes' : 'stores'} it.',
    '///',
    '/// Everything here follows from the live schema — even this file\'s',
    '/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.',
    'final class $className {',
    indented('  ', members),
    '}',
  ].join('\n');
}

/// One `export` per generated row file, so consumers import one thing.
String barrelFile(List<SchemaTable> tables) => [
      '/// Every generated row class, one per table the database has.',
      'library;',
      '',
      for (final table in tables) "export '${fileNameFor(table.name)}';",
    ].join('\n');

/// What the seed class holds: the schema's shape, and where every row class
/// went — the region names the files the macro authored beside it.
Section manifest(List<SchemaTable> tables) => (
      lines: [
        '/// Every table the database has, in name order.',
        'static const List<String> tables = '
            '${_quoted([
              for (final table in tables)
                if (!table.isView) table.name
            ])};',
        '',
        '/// Every view. A view generates a read-only row class.',
        'static const List<String> views = '
            '${_quoted([
              for (final table in tables)
                if (table.isView) table.name
            ])};',
        '',
        '/// The generated file each row class lives in, keyed by table.',
        'static const Map<String, String> rowFiles = {',
        for (final table in tables)
          "  '${table.name}': '${fileNameFor(table.name)}',",
        '};',
      ],
      names: ['tables', 'views', 'rowFiles'],
    );

/// The whole class body for one table.
Section membersFor(
  String className,
  SchemaTable table,
  List<MappedColumn> columns,
) {
  final keys = [
    for (final column in columns)
      if (column.column.isPrimaryKey) column,
  ]..sort((a, b) => a.column.keyPosition.compareTo(b.column.keyPosition));
  final parents = [
    for (final column in columns)
      if (column.column.references != null) column,
  ];
  final insertable = !table.isView;
  return _joined([
    _fields(table.name, columns),
    _constructor(className, table.name, columns),
    _metadata(table, columns, keys, parents),
    _selects(table.name, columns, keys, parents),
    if (insertable) _insert(table.name, columns),
    if (keys.isNotEmpty) _keyValues(keys),
    _toRow(columns),
    _fromRow(className, table.name, columns),
  ]);
}

/// One field per column — the reason this macro exists.
Section _fields(String table, List<MappedColumn> columns) => (
      lines: [
        for (final column in columns) ...[
          '/// ${column.doc(table)}',
          'final ${column.fieldType} ${column.field};',
          '',
        ],
      ],
      names: [for (final column in columns) column.field],
    );

/// A constructor whose parameters are the columns, required exactly when the
/// database requires a value.
Section _constructor(
  String className,
  String table,
  List<MappedColumn> columns,
) =>
    (
      lines: [
        '/// Builds a row of `$table`. Every parameter is one of its columns.',
        'const $className({',
        for (final column in columns)
          '  ${column.notNull ? 'required ' : ''}this.${column.field},',
        '});',
        '',
      ],
      names: ['$className.new'],
    );

/// The table's identity, as constants callers can build queries from.
Section _metadata(
  SchemaTable table,
  List<MappedColumn> columns,
  List<MappedColumn> keys,
  List<MappedColumn> parents,
) =>
    (
      lines: [
        '/// The ${table.isView ? 'view' : 'table'} these rows come from.',
        "static const String tableName = '${table.name}';",
        '',
        '/// Every column `${table.name}` actually has, in schema order.',
        'static const List<String> columnNames = ${_quoted([
              for (final column in columns) column.name
            ])};',
        '',
        if (keys.isNotEmpty) ...[
          '/// The primary key, in key order.',
          'static const List<String> primaryKeyColumns = ${_quoted([
                for (final key in keys) key.name
              ])};',
          '',
        ],
        if (parents.isNotEmpty) ...[
          '/// Which of these columns point at another table.',
          'static const Map<String, String> references = {',
          for (final column in parents)
            "  '${column.name}': '${column.column.references}',",
          '};',
          '',
        ],
      ],
      names: [
        'tableName',
        'columnNames',
        if (keys.isNotEmpty) 'primaryKeyColumns',
        if (parents.isNotEmpty) 'references',
      ],
    );

/// One SELECT naming its columns, plus one per way the schema says these rows
/// are looked up: by their key, and by each foreign key.
Section _selects(
  String table,
  List<MappedColumn> columns,
  List<MappedColumn> keys,
  List<MappedColumn> parents,
) {
  final projection = 'SELECT ${columns.map((c) => c.name).join(', ')} '
      'FROM $table';
  return (
    lines: [
      '/// Every row, naming its columns, so a schema change is a',
      '/// compile-time-visible change here rather than a runtime surprise.',
      'static const String selectAllSql =',
      "    '$projection';",
      '',
      if (keys.isNotEmpty) ...[
        '/// One row by its primary key, for `keyValues` in that order.',
        'static const String selectByKeySql =',
        "    '$projection WHERE ${_conjunction(keys)}';",
        '',
      ],
      for (final parent in parents) ...[
        '/// Every row pointing at one row of `${parent.column.referencesTable}`.',
        'static const String ${_selectByName(parent)} =',
        "    '$projection WHERE ${parent.name} = ?';",
        '',
      ],
    ],
    names: [
      'selectAllSql',
      if (keys.isNotEmpty) 'selectByKeySql',
      for (final parent in parents) _selectByName(parent),
    ],
  );
}

/// An INSERT and the values it takes — tables only, since SQLite will not
/// insert into a view.
Section _insert(String table, List<MappedColumn> columns) => (
      lines: [
        '/// An INSERT of every column, for `insertValues` in that order.',
        'static const String insertSql =',
        "    'INSERT INTO $table (${columns.map((c) => c.name).join(', ')}) "
            "VALUES (${List.filled(columns.length, '?').join(', ')})';",
        '',
        '/// The values `insertSql` takes, in its parameter order.',
        'List<Object?> get insertValues => toRow().values.toList(growable: false);',
        '',
      ],
      names: ['insertSql', 'insertValues'],
    );

/// This row's primary key, for `selectByKeySql`.
Section _keyValues(List<MappedColumn> keys) => (
      lines: [
        '/// This row\'s primary key, for `selectByKeySql` in that order.',
        'List<Object?> get keyValues => [${keys.map((c) => c.encoded).join(', ')}];',
        '',
      ],
      names: ['keyValues'],
    );

/// Dart values out to database values.
Section _toRow(List<MappedColumn> columns) => (
      lines: [
        '/// This row as database values, keyed by real column name.',
        'Map<String, Object?> toRow() => {',
        for (final column in columns) "  '${column.name}': ${column.encoded},",
        '};',
        '',
      ],
      names: ['toRow'],
    );

/// Database values back in, as a `Result` — a row that does not match the
/// schema is a value, never an exception.
Section _fromRow(String className, String table, List<MappedColumn> columns) =>
    (
      lines: [
        '/// One database row decoded, or a [DecodeError] when it does not',
        '/// match the schema this class was generated from.',
        'static Result<$className, DecodeError> fromRow(Map<String, Object?> row) =>',
        '    switch (row) {',
        '      {',
        for (final column in columns)
          "        '${column.name}': final ${column.patternType} ${column.field},",
        '      } =>',
        '        Ok($className(',
        for (final column in columns)
          '          ${column.field}: ${column.decoded},',
        '        )),',
        "      _ => Err(DecodeError('$className', 'a row of $table', row)),",
        '    };',
        '',
      ],
      names: ['fromRow'],
    );

/// Every section, in order, as one section.
Section _joined(List<Section> sections) => (
      lines: [for (final section in sections) ...section.lines],
      names: [for (final section in sections) ...section.names],
    );

/// `selectByCustomerIdSql`, from the column that does the pointing.
String _selectByName(MappedColumn column) =>
    'selectBy${column.field.substring(0, 1).toUpperCase()}'
    '${column.field.substring(1)}Sql';

/// `id = ?`, or `order_id = ? AND product_id = ?` for a composite key.
String _conjunction(List<MappedColumn> keys) =>
    keys.map((column) => '${column.name} = ?').join(' AND ');

/// A Dart list literal of quoted names.
String _quoted(List<String> names) =>
    '[${names.map((name) => "'$name'").join(', ')}]';

/// Puts the member indent on every non-blank line, so the section builders
/// stay readable.
String indented(String indent, List<String> lines) =>
    lines.map((line) => line.isEmpty ? '' : '$indent$line').join('\n');
