/// The generated SQL runs, against the real thing [dartmacros.pipeline].
///
/// A fresh database, real INSERTs through the generated statements, then every
/// generated way back in: by key, by each foreign key, and through the view.
/// No mocks anywhere — if the SQL were wrong, SQLite would say so here.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_sqlite_schema_example/rows.dart';
import 'package:test/test.dart';

import '../tool/dmx/src/database.dart';
import 'support.dart';

void main() {
  late String database;

  setUp(() {
    database = freshDatabase('round-trip', addTearDown);
    expect(database, isNotEmpty, reason: 'sqlite3 must build the fixture');
    for (final (sql, values) in [
      (CustomerRow.insertSql, sampleCustomer.insertValues),
      (ProductRow.insertSql, samplePan.insertValues),
      (ProductRow.insertSql, sampleMug.insertValues),
      (OrderRow.insertSql, sampleOrder.insertValues),
      (OrderLineRow.insertSql, samplePanLine.insertValues),
      (OrderLineRow.insertSql, sampleMugLine.insertValues),
    ]) {
      expect(execute(database, '${bound(sql, values)};'), isEmpty);
    }
  });

  /// The decoded rows of one query, failing the test on any decode error.
  List<T> decoded<T>(
    String sql,
    Result<T, DecodeError> Function(Map<String, Object?>) fromRow,
  ) =>
      [
        for (final row in queryRows(database, sql))
          switch (fromRow(row)) {
            Ok(value: final value) => value,
            Err(error: final error) => fail('$error'),
          },
      ];

  test('a row inserted through insertSql comes back by primary key', () {
    final rows = decoded(
      bound(ProductRow.selectByKeySql, samplePan.keyValues),
      ProductRow.fromRow,
    );
    expect(rows, hasLength(1));
    expect(rows.first.toRow(), samplePan.toRow());
  });

  test('a draft with NULLs survives the trip untouched', () {
    final rows = decoded(
      bound(ProductRow.selectByKeySql, sampleMug.keyValues),
      ProductRow.fromRow,
    );
    expect(rows.first.publishedAt, isNull);
    expect(rows.first.weightGrams, isNull);
    expect(rows.first.inStock, isFalse);
  });

  test('the foreign-key lookup finds a customer\'s orders', () {
    final rows = decoded(
      bound(OrderRow.selectByCustomerIdSql, [sampleCustomer.id]),
      OrderRow.fromRow,
    );
    expect(rows, hasLength(1));
    expect(rows.first.toRow(), sampleOrder.toRow());
  });

  test('a composite key: both lines of the order, then one line exactly', () {
    final lines = decoded(
      bound(OrderLineRow.selectByOrderIdSql, [sampleOrder.id]),
      OrderLineRow.fromRow,
    );
    expect(lines, hasLength(2), reason: 'the order has two lines');
    final one = decoded(
      bound(OrderLineRow.selectByKeySql, sampleMugLine.keyValues),
      OrderLineRow.fromRow,
    );
    expect(one, hasLength(1));
    expect(one.first.toRow(), sampleMugLine.toRow());
  });

  test('the view computes what the generated class reads back', () {
    final rows = decoded(
      CustomerSpendRow.selectAllSql,
      CustomerSpendRow.fromRow,
    );
    expect(rows, hasLength(1));
    expect(rows.first.toRow(), sampleSpend.toRow());
    expect(
      rows.first.spentCents,
      2 * 4950 + 3 * 1200,
      reason: 'two pans and three mugs',
    );
  });
}
