// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('table')` [catalogue.table] — the schema and the row mapper, from one class.
//
// Two things drift in every app with a local database: the `CREATE TABLE` you
// wrote in a migration, and the `toMap`/`fromMap` pair you wrote next to the
// model. They drift because they are two hand-written encodings of one fact.
// Here the class *is* the fact: the DDL, the indexes, the statements, the
// argument lists, and the row decoder all come out of the same field list, so
// adding a column is one line above the divider.
//
// The SQL is plain text and the statements are parameterised — no string
// interpolation of values, ever, because that is how injection happens.

import 'package:dmx/dmx.dart';

/// A product as it is stored.
@dmx('table', {'name': 'products', 'primaryKey': 'id'})
@dmx('model', {'json': false, 'copyWith': false})
class ProductRow {
  const ProductRow({
    required this.id,
    required this.title,
    required this.priceCents,
    required this.currency,
    required this.publishedAt,
    required this.inStock,
    this.description,
  });

  final String id;

  @dmx('column', {'indexed': true})
  final String title;

  final int priceCents;
  final String currency;
  final DateTime publishedAt;
  final bool inStock;
  final String? description;

  //#region
  static const String tableName = 'products';

  static const List<String> columns = <String>[
    'id',
    'title',
    'price_cents',
    'currency',
    'published_at',
    'in_stock',
    'description',
  ];

  static const String createTableSql = 'CREATE TABLE IF NOT EXISTS products (\n'
      '  id TEXT NOT NULL PRIMARY KEY,\n'
      '  title TEXT NOT NULL,\n'
      '  price_cents INTEGER NOT NULL,\n'
      '  currency TEXT NOT NULL,\n'
      '  published_at TEXT NOT NULL,\n'
      '  in_stock INTEGER NOT NULL,\n'
      '  description TEXT\n'
      ')';

  static const List<String> createIndexSql = <String>[
    'CREATE INDEX IF NOT EXISTS products_title_idx ON products (title)',
  ];

  static const String insertSql = 'INSERT INTO products '
      '(id, title, price_cents, currency, published_at, in_stock, description) '
      'VALUES (?, ?, ?, ?, ?, ?, ?)';

  static const String upsertSql = 'INSERT INTO products '
      '(id, title, price_cents, currency, published_at, in_stock, description) '
      'VALUES (?, ?, ?, ?, ?, ?, ?) '
      'ON CONFLICT(id) DO UPDATE SET '
      'title = excluded.title, '
      'price_cents = excluded.price_cents, '
      'currency = excluded.currency, '
      'published_at = excluded.published_at, '
      'in_stock = excluded.in_stock, '
      'description = excluded.description'
      ;

  static const String selectAllSql = 'SELECT '
      'id, title, price_cents, currency, published_at, in_stock, description '
      'FROM products';

  static const String selectByIdSql = '$selectAllSql WHERE id = ?';

  static const String deleteByIdSql =
      'DELETE FROM products WHERE id = ?';

  /// Positional arguments for [insertSql] and [upsertSql], in column order.
  /// Values are bound, never interpolated — that is how injection happens.
  List<Object?> get insertArgs => <Object?>[
        id,
        title,
        priceCents,
        currency,
        publishedAt.toIso8601String(),
        inStock ? 1 : 0,
        description,
      ];

  Map<String, Object?> toRow() => <String, Object?>{
        'id': id,
        'title': title,
        'price_cents': priceCents,
        'currency': currency,
        'published_at': publishedAt.toIso8601String(),
        'in_stock': inStock ? 1 : 0,
        'description': description,
      };

  /// A row is data from outside the program, exactly like JSON is, so it is
  /// decoded exactly like JSON is: patterns, a path, and a `Result`.
  static Result<ProductRow, DecodeError> fromRow(
    Map<String, Object?> row, [
    String path = 'ProductRow',
  ]) =>
      switch (row) {
        {
          'id': final String id,
          'title': final String title,
          'price_cents': final int priceCents,
          'currency': final String currency,
          'published_at': final String publishedAt,
          'in_stock': final int inStock,
        } =>
          switch ((
            dmxDateTime(publishedAt, '$path.published_at'),
            dmxNullable<String>(dmxKey(row, 'description'), '$path.description', dmxString),
          )) {
            (
              Ok(value: final publishedAt),
              Ok(value: final description),
            ) =>
              Ok(ProductRow(
                id: id,
                title: title,
                priceCents: priceCents,
                currency: currency,
                publishedAt: publishedAt,
                inStock: inStock != 0,
                description: description,
              )),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'ProductRow', row)),
      };

  /// Decodes a whole result set, failing on the first bad row and saying
  /// which one it was.
  static Result<List<ProductRow>, DecodeError> fromRows(
    List<Map<String, Object?>> rows, [
    String path = 'ProductRow',
  ]) =>
      dmxList<ProductRow>(
        rows,
        path,
        (value, path) => switch (value) {
          final Map<String, Object?> row => fromRow(row, path),
          _ => Err(DecodeError(path, 'ProductRow', value)),
        },
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ProductRow &&
          other.id == id &&
          other.title == title &&
          other.priceCents == priceCents &&
          other.currency == currency &&
          other.publishedAt == publishedAt &&
          other.inStock == inStock &&
          other.description == description);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        title,
        priceCents,
        currency,
        publishedAt,
        inStock,
        description,
      );

  @override
  String toString() => 'ProductRow(id: $id, title: $title, priceCents: $priceCents, currency: $currency, publishedAt: $publishedAt, inStock: $inStock, description: $description)';
  //#endregion
}

/// A line of an order, pointing back at its order and its product.
@dmx('table', {'name': 'order_lines', 'primaryKey': 'id'})
@dmx('model', {'json': false, 'copyWith': false})
class OrderLineRow {
  const OrderLineRow({
    required this.id,
    required this.orderId,
    required this.productId,
    required this.quantity,
    required this.unitPriceCents,
  });

  final int id;

  @dmx('column', {'references': 'orders(id)', 'indexed': true})
  final String orderId;

  @dmx('column', {'references': 'products(id)'})
  final String productId;

  final int quantity;
  final int unitPriceCents;

  //#region
  static const String tableName = 'order_lines';

  static const List<String> columns = <String>[
    'id',
    'order_id',
    'product_id',
    'quantity',
    'unit_price_cents',
  ];

  static const String createTableSql = 'CREATE TABLE IF NOT EXISTS order_lines (\n'
      '  id INTEGER NOT NULL PRIMARY KEY,\n'
      '  order_id TEXT NOT NULL REFERENCES orders(id),\n'
      '  product_id TEXT NOT NULL REFERENCES products(id),\n'
      '  quantity INTEGER NOT NULL,\n'
      '  unit_price_cents INTEGER NOT NULL\n'
      ')';

  static const List<String> createIndexSql = <String>[
    'CREATE INDEX IF NOT EXISTS order_lines_order_id_idx ON order_lines (order_id)',
  ];

  static const String insertSql = 'INSERT INTO order_lines '
      '(id, order_id, product_id, quantity, unit_price_cents) '
      'VALUES (?, ?, ?, ?, ?)';

  static const String upsertSql = 'INSERT INTO order_lines '
      '(id, order_id, product_id, quantity, unit_price_cents) '
      'VALUES (?, ?, ?, ?, ?) '
      'ON CONFLICT(id) DO UPDATE SET '
      'order_id = excluded.order_id, '
      'product_id = excluded.product_id, '
      'quantity = excluded.quantity, '
      'unit_price_cents = excluded.unit_price_cents'
      ;

  static const String selectAllSql = 'SELECT '
      'id, order_id, product_id, quantity, unit_price_cents '
      'FROM order_lines';

  static const String selectByIdSql = '$selectAllSql WHERE id = ?';

  static const String deleteByIdSql =
      'DELETE FROM order_lines WHERE id = ?';

  /// Positional arguments for [insertSql] and [upsertSql], in column order.
  /// Values are bound, never interpolated — that is how injection happens.
  List<Object?> get insertArgs => <Object?>[
        id,
        orderId,
        productId,
        quantity,
        unitPriceCents,
      ];

  Map<String, Object?> toRow() => <String, Object?>{
        'id': id,
        'order_id': orderId,
        'product_id': productId,
        'quantity': quantity,
        'unit_price_cents': unitPriceCents,
      };

  /// A row is data from outside the program, exactly like JSON is, so it is
  /// decoded exactly like JSON is: patterns, a path, and a `Result`.
  static Result<OrderLineRow, DecodeError> fromRow(
    Map<String, Object?> row, [
    String path = 'OrderLineRow',
  ]) =>
      switch (row) {
        {
          'id': final int id,
          'order_id': final String orderId,
          'product_id': final String productId,
          'quantity': final int quantity,
          'unit_price_cents': final int unitPriceCents,
        } =>
          Ok(OrderLineRow(
            id: id,
            orderId: orderId,
            productId: productId,
            quantity: quantity,
            unitPriceCents: unitPriceCents,
          )),
        _ => Err(DecodeError(path, 'OrderLineRow', row)),
      };

  /// Decodes a whole result set, failing on the first bad row and saying
  /// which one it was.
  static Result<List<OrderLineRow>, DecodeError> fromRows(
    List<Map<String, Object?>> rows, [
    String path = 'OrderLineRow',
  ]) =>
      dmxList<OrderLineRow>(
        rows,
        path,
        (value, path) => switch (value) {
          final Map<String, Object?> row => fromRow(row, path),
          _ => Err(DecodeError(path, 'OrderLineRow', value)),
        },
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is OrderLineRow &&
          other.id == id &&
          other.orderId == orderId &&
          other.productId == productId &&
          other.quantity == quantity &&
          other.unitPriceCents == unitPriceCents);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        orderId,
        productId,
        quantity,
        unitPriceCents,
      );

  @override
  String toString() => 'OrderLineRow(id: $id, orderId: $orderId, productId: $productId, quantity: $quantity, unitPriceCents: $unitPriceCents)';
  //#endregion
}

/// The migration, assembled from the tables in this file — hand-written,
/// because the *order* statements run in is a decision, not a derivation.
/// Everything it is made of came out of a region.
const List<String> schemaStatements = <String>[
  ProductRow.createTableSql,
  ...ProductRow.createIndexSql,
  OrderLineRow.createTableSql,
  ...OrderLineRow.createIndexSql,
];
