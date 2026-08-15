// dmx: generated from schema.dart — do not edit.

import 'package:dmx/dmx.dart';

/// One row of `orders`, exactly as the database stores it.
///
/// Everything here follows from the live schema — even this file's
/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.
final class OrderRow {
  /// `orders.id` — TEXT NOT NULL, primary key.
  final String id;

  /// `orders.customer_id` — TEXT NOT NULL, references `customers.id`.
  final String customerId;

  /// `orders.placed_at` — TEXT NOT NULL.
  final String placedAt;

  /// `orders.status` — TEXT NOT NULL.
  final String status;

  /// `orders.note` — TEXT.
  final String? note;

  /// Builds a row of `orders`. Every parameter is one of its columns.
  const OrderRow({
    required this.id,
    required this.customerId,
    required this.placedAt,
    required this.status,
    this.note,
  });

  /// The table these rows come from.
  static const String tableName = 'orders';

  /// Every column `orders` actually has, in schema order.
  static const List<String> columnNames = ['id', 'customer_id', 'placed_at', 'status', 'note'];

  /// The primary key, in key order.
  static const List<String> primaryKeyColumns = ['id'];

  /// Which of these columns point at another table.
  static const Map<String, String> references = {
    'customer_id': 'customers.id',
  };

  /// Every row, naming its columns, so a schema change is a
  /// compile-time-visible change here rather than a runtime surprise.
  static const String selectAllSql =
      'SELECT id, customer_id, placed_at, status, note FROM orders';

  /// One row by its primary key, for `keyValues` in that order.
  static const String selectByKeySql =
      'SELECT id, customer_id, placed_at, status, note FROM orders WHERE id = ?';

  /// Every row pointing at one row of `customers`.
  static const String selectByCustomerIdSql =
      'SELECT id, customer_id, placed_at, status, note FROM orders WHERE customer_id = ?';

  /// An INSERT of every column, for `insertValues` in that order.
  static const String insertSql =
      'INSERT INTO orders (id, customer_id, placed_at, status, note) VALUES (?, ?, ?, ?, ?)';

  /// The values `insertSql` takes, in its parameter order.
  List<Object?> get insertValues => toRow().values.toList(growable: false);

  /// This row's primary key, for `selectByKeySql` in that order.
  List<Object?> get keyValues => [id];

  /// This row as database values, keyed by real column name.
  Map<String, Object?> toRow() => {
    'id': id,
    'customer_id': customerId,
    'placed_at': placedAt,
    'status': status,
    'note': note,
  };

  /// One database row decoded, or a [DecodeError] when it does not
  /// match the schema this class was generated from.
  static Result<OrderRow, DecodeError> fromRow(Map<String, Object?> row) =>
      switch (row) {
        {
          'id': final String id,
          'customer_id': final String customerId,
          'placed_at': final String placedAt,
          'status': final String status,
          'note': final String? note,
        } =>
          Ok(OrderRow(
            id: id,
            customerId: customerId,
            placedAt: placedAt,
            status: status,
            note: note,
          )),
        _ => Err(DecodeError('OrderRow', 'a row of orders', row)),
      };
}
