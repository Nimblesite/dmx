/// What the example's tests share: the generated classes as data, sample rows,
/// and a real database to run generated SQL against [dartmacros.pipeline].
///
/// The five classes have no common supertype — they are five independent
/// generated classes — so [generated] is how a test says the same thing about
/// all of them at once. A member missing from one of them is a compile error
/// here, which is the point.
library;

import 'dart:io';

import 'package:dmx/dmx.dart';
// The barrel is generated too — importing it is the test: one line reaches
// every class the macro authored [dartmacros.files].
import 'package:dmx_sqlite_schema_example/rows.dart';

/// The database the macro generated from.
const String databasePath = 'tool/dmx/app.db';

/// The schema that database was built from.
const String schemaPath = 'tool/dmx/schema.sql';

/// One generated class, reduced to what every one of them has.
typedef Generated = ({
  /// The Dart class, for a failure message.
  String className,

  /// What the class says its table is.
  String tableName,

  /// What the class says its columns are.
  List<String> columnNames,

  /// The primary key it generated, empty when the schema gave it none.
  List<String> primaryKeyColumns,

  /// The foreign keys it generated, empty when the schema gave it none.
  Map<String, String> references,

  /// The SELECT it generated.
  String selectAllSql,

  /// The INSERT it generated, absent for a view.
  String? insertSql,

  /// A sample row's database values.
  Map<String, Object?> row,

  /// Its decoder, which any of these tests may call with anything.
  Result<Object, DecodeError> Function(Map<String, Object?>) fromRow,
});

/// A customer with everything filled in.
const CustomerRow sampleCustomer = CustomerRow(
  id: 'c-1',
  email: 'ada@example.com',
  displayName: 'Ada',
  signedUpAt: '2026-01-02T00:00:00Z',
  marketingOptIn: true,
  loyaltyPoints: 120,
);

/// A published product, with both of its nullable columns present.
const ProductRow samplePan = ProductRow(
  id: 'p-1',
  title: 'Cast iron pan',
  priceCents: 4950,
  inStock: true,
  publishedAt: '2026-01-31T00:00:00Z',
  weightGrams: 482.5,
);

/// A draft product: every optional parameter left off, which only compiles
/// because the schema declares those columns nullable.
const ProductRow sampleMug = ProductRow(
  id: 'p-2',
  title: 'Enamel mug',
  priceCents: 1200,
  inStock: false,
);

/// An order belonging to [sampleCustomer].
const OrderRow sampleOrder = OrderRow(
  id: 'o-1',
  customerId: 'c-1',
  placedAt: '2026-02-01T00:00:00Z',
  status: 'placed',
);

/// Two of the pan, at its full price.
const OrderLineRow samplePanLine = OrderLineRow(
  orderId: 'o-1',
  productId: 'p-1',
  quantity: 2,
  unitPriceCents: 4950,
);

/// Three mugs, discounted.
const OrderLineRow sampleMugLine = OrderLineRow(
  orderId: 'o-1',
  productId: 'p-2',
  quantity: 3,
  unitPriceCents: 1200,
  discountRatio: 0.1,
);

/// What the view should say about [sampleCustomer] once those lines exist.
const CustomerSpendRow sampleSpend = CustomerSpendRow(
  customerId: 'c-1',
  displayName: 'Ada',
  orderCount: 1,
  spentCents: 13500,
);

/// Every generated class, as the facts each one claims about its table.
final List<Generated> generated = [
  (
    className: 'CustomerRow',
    tableName: CustomerRow.tableName,
    columnNames: CustomerRow.columnNames,
    primaryKeyColumns: CustomerRow.primaryKeyColumns,
    references: const {},
    selectAllSql: CustomerRow.selectAllSql,
    insertSql: CustomerRow.insertSql,
    row: sampleCustomer.toRow(),
    fromRow: CustomerRow.fromRow,
  ),
  (
    className: 'ProductRow',
    tableName: ProductRow.tableName,
    columnNames: ProductRow.columnNames,
    primaryKeyColumns: ProductRow.primaryKeyColumns,
    references: const {},
    selectAllSql: ProductRow.selectAllSql,
    insertSql: ProductRow.insertSql,
    row: samplePan.toRow(),
    fromRow: ProductRow.fromRow,
  ),
  (
    className: 'OrderRow',
    tableName: OrderRow.tableName,
    columnNames: OrderRow.columnNames,
    primaryKeyColumns: OrderRow.primaryKeyColumns,
    references: OrderRow.references,
    selectAllSql: OrderRow.selectAllSql,
    insertSql: OrderRow.insertSql,
    row: sampleOrder.toRow(),
    fromRow: OrderRow.fromRow,
  ),
  (
    className: 'OrderLineRow',
    tableName: OrderLineRow.tableName,
    columnNames: OrderLineRow.columnNames,
    primaryKeyColumns: OrderLineRow.primaryKeyColumns,
    references: OrderLineRow.references,
    selectAllSql: OrderLineRow.selectAllSql,
    insertSql: OrderLineRow.insertSql,
    row: samplePanLine.toRow(),
    fromRow: OrderLineRow.fromRow,
  ),
  (
    className: 'CustomerSpendRow',
    tableName: CustomerSpendRow.tableName,
    columnNames: CustomerSpendRow.columnNames,
    primaryKeyColumns: const [],
    references: const {},
    selectAllSql: CustomerSpendRow.selectAllSql,
    // A view. SQLite will not insert into one, so nothing was generated.
    insertSql: null,
    row: sampleSpend.toRow(),
    fromRow: CustomerSpendRow.fromRow,
  ),
];

/// A fresh database with this project's schema, deleted when the test ends.
String freshDatabase(String name, void Function(void Function()) onTearDown) {
  final directory = Directory.systemTemp.createTempSync('dmx-sqlite-$name');
  onTearDown(() => directory.deleteSync(recursive: true));
  final database = '${directory.path}/$name.db';
  // `.read` rather than the SQL as an argument: schema.sql opens with a `--`
  // comment, which sqlite3 would parse as a command-line option.
  final created = Process.runSync('sqlite3', [
    database,
    '.read ${File(schemaPath).absolute.path}',
  ]);
  return created.exitCode == 0 ? database : '';
}

/// Runs a statement, returning its stderr — empty when SQLite accepted it.
String execute(String database, String sql) {
  final run = Process.runSync('sqlite3', [database, sql]);
  final Object? error = run.stderr;
  return run.exitCode == 0 ? '' : '${error is String ? error : run.exitCode}';
}

/// Generated SQL with its `?` placeholders filled in.
///
/// The `sqlite3` CLI has no parameter binding, so the test does what a real
/// binder would. Nothing generated here contains a `?` that is not a
/// placeholder.
String bound(String sql, List<Object?> values) {
  final parts = sql.split('?');
  return [
    for (var index = 0; index < parts.length; index++) ...[
      parts[index],
      if (index < values.length) literal(values[index]),
    ],
  ].join();
}

/// One Dart value as the SQL literal SQLite reads it back as.
String literal(Object? value) => switch (value) {
      null => 'NULL',
      final int number => '$number',
      final double number => '$number',
      final Object other => "'${other.toString().replaceAll("'", "''")}'",
    };
