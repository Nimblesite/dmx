// dmx: generated from schema.dart — do not edit.

import 'package:dmx/dmx.dart';

/// One row of `order_lines`, exactly as the database stores it.
///
/// Everything here follows from the live schema — even this file's
/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.
final class OrderLineRow {
  /// `order_lines.order_id` — TEXT NOT NULL, primary key, references `orders.id`.
  final String orderId;

  /// `order_lines.product_id` — TEXT NOT NULL, primary key, references `products.id`.
  final String productId;

  /// `order_lines.quantity` — INTEGER NOT NULL.
  final int quantity;

  /// `order_lines.unit_price_cents` — INTEGER NOT NULL.
  final int unitPriceCents;

  /// `order_lines.discount_ratio` — REAL.
  final double? discountRatio;

  /// Builds a row of `order_lines`. Every parameter is one of its columns.
  const OrderLineRow({
    required this.orderId,
    required this.productId,
    required this.quantity,
    required this.unitPriceCents,
    this.discountRatio,
  });

  /// The table these rows come from.
  static const String tableName = 'order_lines';

  /// Every column `order_lines` actually has, in schema order.
  static const List<String> columnNames = ['order_id', 'product_id', 'quantity', 'unit_price_cents', 'discount_ratio'];

  /// The primary key, in key order.
  static const List<String> primaryKeyColumns = ['order_id', 'product_id'];

  /// Which of these columns point at another table.
  static const Map<String, String> references = {
    'order_id': 'orders.id',
    'product_id': 'products.id',
  };

  /// Every row, naming its columns, so a schema change is a
  /// compile-time-visible change here rather than a runtime surprise.
  static const String selectAllSql =
      'SELECT order_id, product_id, quantity, unit_price_cents, discount_ratio FROM order_lines';

  /// One row by its primary key, for `keyValues` in that order.
  static const String selectByKeySql =
      'SELECT order_id, product_id, quantity, unit_price_cents, discount_ratio FROM order_lines WHERE order_id = ? AND product_id = ?';

  /// Every row pointing at one row of `orders`.
  static const String selectByOrderIdSql =
      'SELECT order_id, product_id, quantity, unit_price_cents, discount_ratio FROM order_lines WHERE order_id = ?';

  /// Every row pointing at one row of `products`.
  static const String selectByProductIdSql =
      'SELECT order_id, product_id, quantity, unit_price_cents, discount_ratio FROM order_lines WHERE product_id = ?';

  /// An INSERT of every column, for `insertValues` in that order.
  static const String insertSql =
      'INSERT INTO order_lines (order_id, product_id, quantity, unit_price_cents, discount_ratio) VALUES (?, ?, ?, ?, ?)';

  /// The values `insertSql` takes, in its parameter order.
  List<Object?> get insertValues => toRow().values.toList(growable: false);

  /// This row's primary key, for `selectByKeySql` in that order.
  List<Object?> get keyValues => [orderId, productId];

  /// This row as database values, keyed by real column name.
  Map<String, Object?> toRow() => {
    'order_id': orderId,
    'product_id': productId,
    'quantity': quantity,
    'unit_price_cents': unitPriceCents,
    'discount_ratio': discountRatio,
  };

  /// One database row decoded, or a [DecodeError] when it does not
  /// match the schema this class was generated from.
  static Result<OrderLineRow, DecodeError> fromRow(Map<String, Object?> row) =>
      switch (row) {
        {
          'order_id': final String orderId,
          'product_id': final String productId,
          'quantity': final int quantity,
          'unit_price_cents': final int unitPriceCents,
          'discount_ratio': final double? discountRatio,
        } =>
          Ok(OrderLineRow(
            orderId: orderId,
            productId: productId,
            quantity: quantity,
            unitPriceCents: unitPriceCents,
            discountRatio: discountRatio,
          )),
        _ => Err(DecodeError('OrderLineRow', 'a row of order_lines', row)),
      };
}
