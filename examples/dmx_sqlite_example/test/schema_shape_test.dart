/// The generated classes ARE the database's shape [dartmacros.pipeline].
///
/// Not one of the five row classes exists as source anyone wrote: the macro
/// authored every file [dartmacros.files]. These tests read the same live
/// database the macro read, through the same reader, and check every claim
/// the generated code makes against it. If the macro ever drifts from the
/// schema, they fail.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_sqlite_schema_example/rows.dart';
import 'package:test/test.dart';

import '../tool/dmx/src/database.dart';
import 'support.dart';

/// One table as the database describes it right now, or a failure naming it.
SchemaTable live(String table) => switch (readTable(databasePath, table)) {
      final SchemaTable found => found,
      null => fail('the database has no `$table` to check against'),
    };

void main() {
  test('every generated class names a table the database actually has', () {
    final tables = tableNames(databasePath);
    expect(tables, hasLength(5), reason: 'four tables and one view');
    for (final row in generated) {
      expect(
        tables,
        contains(row.tableName),
        reason: '${row.className} generated a table that does not exist',
      );
    }
  });

  test('no class was told its table — the class name was the question', () {
    // The names below are what the macro RESOLVED, and nothing in lib/ says
    // any of them. `CustomerSpendRow` reaching a view is the interesting one:
    // it matched exactly, with no plural, because that is what the database
    // had.
    expect(generated.map((row) => row.tableName), [
      'customers',
      'products',
      'orders',
      'order_lines',
      'customer_spend',
    ]);
  });

  test('columns mirror the live schema exactly, in order', () {
    for (final row in generated) {
      expect(
        row.columnNames,
        [for (final column in live(row.tableName).columns) column.name],
        reason: '${row.className} disagrees with its table',
      );
    }
    expect(ProductRow.columnNames, [
      'id',
      'title',
      'price_cents',
      'in_stock',
      'published_at',
      'weight_grams',
    ]);
  });

  test('there is one field per column, and no field without one', () {
    // toRow() is keyed by column and written one entry per generated field, so
    // its key set IS the generated field set. A hand-written field would show
    // up as an extra key; a column the macro skipped, as a missing one.
    for (final row in generated) {
      expect(
        row.row.keys,
        row.columnNames,
        reason: '${row.className} has a field that is not a column, or misses '
            'one that is',
      );
      expect(row.row, hasLength(live(row.tableName).columns.length));
    }
  });

  test('the primary key is the schema key, in key order', () {
    for (final row in generated) {
      expect(
        row.primaryKeyColumns,
        [for (final column in live(row.tableName).primaryKey) column.name],
        reason: '${row.className} disagrees about its key',
      );
    }
    // A composite key is two columns in the order the schema declares, which
    // is the order `keyValues` and `selectByKeySql` both use.
    expect(OrderLineRow.primaryKeyColumns, ['order_id', 'product_id']);
    expect(samplePanLine.keyValues, ['o-1', 'p-1']);
    expect(ProductRow.primaryKeyColumns, ['id']);
  });

  test('the foreign keys are the schema foreign keys', () {
    for (final row in generated) {
      expect(
          row.references,
          {
            for (final column in live(row.tableName).foreignKeys)
              column.name:
                  '${column.referencesTable}.${column.referencesColumn}',
          },
          reason: '${row.className} disagrees about what it points at');
    }
    expect(OrderRow.references, {'customer_id': 'customers.id'});
    expect(OrderLineRow.references, {
      'order_id': 'orders.id',
      'product_id': 'products.id',
    });
  });

  test('a REFERENCES becomes a lookup by that column', () {
    expect(
      OrderRow.selectByCustomerIdSql,
      endsWith('FROM orders WHERE customer_id = ?'),
    );
    expect(
      OrderLineRow.selectByOrderIdSql,
      endsWith('FROM order_lines WHERE order_id = ?'),
    );
    expect(
      OrderLineRow.selectByProductIdSql,
      endsWith('FROM order_lines WHERE product_id = ?'),
    );
    expect(
      OrderLineRow.selectByKeySql,
      endsWith('WHERE order_id = ? AND product_id = ?'),
    );
  });

  test('every generated SELECT is SQL this database accepts', () {
    for (final row in generated) {
      expect(
        execute(databasePath, '${row.selectAllSql};'),
        isEmpty,
        reason: '${row.className} generated SQL the database rejected',
      );
    }
  });

  test('a view gets no INSERT; a table gets one naming every column', () {
    for (final row in generated) {
      final expected = live(row.tableName).isView ? isNull : isNotNull;
      expect(
        row.insertSql,
        expected,
        reason:
            '${row.className} is a ${live(row.tableName).isView ? 'view' : 'table'}',
      );
      for (final column
          in row.insertSql == null ? <String>[] : row.columnNames) {
        expect(row.insertSql, contains(column));
      }
    }
    expect(
      ProductRow.insertSql,
      'INSERT INTO products (id, title, price_cents, in_stock, published_at, '
      'weight_grams) VALUES (?, ?, ?, ?, ?, ?)',
    );
  });

  test('each field has the Dart type its declared column type maps to', () {
    // The static types are what the generator chose; reading them back off a
    // value proves the choice, and the live schema names the reason for it.
    final declared = {
      for (final column in live('products').columns)
        column.name: column.declaredType,
    };
    expect(declared['price_cents'], 'INTEGER');
    expect(samplePan.priceCents, isA<int>());
    expect(declared['in_stock'], 'BOOLEAN');
    expect(samplePan.inStock, isA<bool>());
    expect(declared['weight_grams'], 'REAL');
    expect(samplePan.weightGrams, isA<double>());
    expect(declared['title'], 'TEXT');
    expect(samplePan.title, isA<String>());
  });

  test('a NOT NULL column is a required parameter, a nullable one is not', () {
    // That `sampleMug` compiles without `publishedAt` or `weightGrams` is the
    // assertion; omitting `id` would not compile at all.
    expect(sampleMug.publishedAt, isNull);
    expect(sampleMug.weightGrams, isNull);
    for (final column in live('products').columns) {
      expect(
        samplePan.toRow()[column.name],
        column.notNull ? isNotNull : anything,
        reason: '${column.name} is NOT NULL, so no row may omit it',
      );
    }
  });

  test('a bool column stores as the integer SQLite keeps it as', () {
    expect(samplePan.toRow()['in_stock'], 1);
    expect(sampleMug.toRow()['in_stock'], 0);
    expect(sampleCustomer.toRow()['marketing_opt_in'], 1);
  });

  test('toRow keys are database column names, not Dart field names', () {
    expect(samplePan.toRow().keys, containsAll(['price_cents', 'in_stock']));
    expect(samplePan.toRow().containsKey('priceCents'), isFalse);
  });

  test('insertValues are the row values in the INSERT parameter order', () {
    expect(samplePan.insertValues, [
      for (final column in ProductRow.columnNames) samplePan.toRow()[column],
    ]);
    expect(samplePan.insertValues.first, 'p-1');
  });

  test('every fromRow refuses a row that is not its table, as a value', () {
    for (final row in generated) {
      switch (row.fromRow(const {'nope': 1})) {
        case Ok(value: final decoded):
          fail('${row.className} decoded a foreign row to $decoded');
        case Err(error: final error):
          expect(error.expected, 'a row of ${row.tableName}');
      }
    }
  });

  test('a view decodes what a view can hold: anything, or nothing', () {
    // Every view column is nullable because SQLite guarantees nothing about
    // one, and the generated class says exactly that rather than claiming
    // more than the database will honour.
    for (final column in live('customer_spend').columns) {
      expect(column.notNull, isFalse);
    }
    switch (CustomerSpendRow.fromRow(const {
      'customer_id': null,
      'display_name': null,
      'order_count': null,
      'spent_cents': null,
    })) {
      case Ok(value: final decoded):
        expect(decoded.spentCents, isNull);
      case Err(error: final error):
        fail('an all-null view row is legal, but did not decode: $error');
    }
  });
}
