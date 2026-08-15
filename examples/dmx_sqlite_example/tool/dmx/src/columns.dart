/// A database column as a Dart field [dartmacros].
///
/// The declared type decides the Dart type, the NOT NULL decides nullability,
/// and a type this example cannot map is refused rather than guessed at — a
/// silently wrong Dart type is a runtime bug in every caller.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';

import 'database.dart';
import 'naming.dart';

/// One column, with the Dart type it maps to worked out.
final class MappedColumn {
  /// The column as the database describes it.
  final SchemaColumn column;

  /// The non-nullable Dart type this column's values have.
  final String dartType;

  /// Builds a mapped column.
  const MappedColumn(this.column, {required this.dartType});

  /// The column name as the database spells it.
  String get name => column.name;

  /// Whether the database requires a value.
  bool get notNull => column.notNull;

  /// The Dart field name: `price_cents` becomes `priceCents`.
  String get field => camelCase(column.name);

  /// The field's declared type, nullable exactly when the column is.
  String get fieldType => notNull ? dartType : '$dartType?';

  /// The type SQLite actually stores — a bool lives there as an integer.
  String get storedType => dartType == 'bool' ? 'int' : dartType;

  /// The pattern type `fromRow` matches this column's stored value with.
  String get patternType => notNull ? storedType : '$storedType?';

  /// The expression `toRow` writes for this field.
  String get encoded => switch ((dartType, notNull)) {
        ('bool', true) => '$field ? 1 : 0',
        ('bool', false) =>
          'switch ($field) { null => null, final value => value ? 1 : 0 }',
        _ => field,
      };

  /// The expression `fromRow` passes to the constructor for this column.
  String get decoded => switch ((dartType, notNull)) {
        ('bool', true) => '$field != 0',
        ('bool', false) =>
          'switch ($field) { null => null, final value => value != 0 }',
        _ => field,
      };

  /// The doc comment, stating what this column is — including the parts that
  /// are not its type, because those are the parts a caller cannot see.
  String doc(String table) => [
        '`$table.$name` — ${column.declaredType}',
        if (notNull) ' NOT NULL',
        if (column.isPrimaryKey) ', primary key',
        if (column.references case final String parent)
          ', references `$parent`',
        '.',
      ].join();
}

/// Every column of [table] mapped, or the first one with no Dart type.
Result<List<MappedColumn>, DmxRefusal> mapColumns(SchemaTable table) {
  final mapped = <MappedColumn>[];
  for (final column in table.columns) {
    final dartType = dartTypeFor(column.declaredType);
    if (dartType == null) {
      return Err(DmxRefusal('DMX3914', _unmappable(table, column)));
    }
    mapped.add(MappedColumn(column, dartType: dartType));
  }
  return Ok(mapped);
}

/// The Dart type a declared SQLite type holds, or null when this example has
/// no mapping for it.
///
/// SQLite keeps the declared word verbatim rather than reducing it to a
/// storage class, so `BOOLEAN` survives `PRAGMA table_info` and is a usable
/// signal even though what it stores is a number.
String? dartTypeFor(String declared) =>
    switch (declared.split('(').first.trim().toUpperCase()) {
      'TEXT' || 'VARCHAR' || 'CHARACTER' || 'CLOB' => 'String',
      'INTEGER' || 'INT' || 'BIGINT' || 'SMALLINT' => 'int',
      'REAL' || 'DOUBLE' || 'FLOAT' || 'NUMERIC' => 'double',
      'BOOLEAN' || 'BOOL' => 'bool',
      _ => null,
    };

/// Why a column could not be mapped, in terms of the thing to change.
String _unmappable(SchemaTable table, SchemaColumn column) => column
        .declaredType
        .trim()
        .isEmpty
    ? '`${table.name}.${column.name}` has no declared type. A computed view '
        'column needs a `CAST(… AS INTEGER)` to have one, and without it '
        'there is nothing to derive a Dart type from.'
    : '`${table.name}.${column.name}` is declared `${column.declaredType}`, '
        'which this example has no Dart type for. Use TEXT, INTEGER, REAL '
        'or BOOLEAN.';
