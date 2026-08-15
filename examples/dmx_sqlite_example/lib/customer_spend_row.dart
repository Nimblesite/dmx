// dmx: generated from schema.dart — do not edit.

import 'package:dmx/dmx.dart';

/// One row of `customer_spend`, exactly as the database computes it.
///
/// Everything here follows from the live schema — even this file's
/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.
final class CustomerSpendRow {
  /// `customer_spend.customer_id` — TEXT.
  final String? customerId;

  /// `customer_spend.display_name` — TEXT.
  final String? displayName;

  /// `customer_spend.order_count` — INT.
  final int? orderCount;

  /// `customer_spend.spent_cents` — INT.
  final int? spentCents;

  /// Builds a row of `customer_spend`. Every parameter is one of its columns.
  const CustomerSpendRow({
    this.customerId,
    this.displayName,
    this.orderCount,
    this.spentCents,
  });

  /// The view these rows come from.
  static const String tableName = 'customer_spend';

  /// Every column `customer_spend` actually has, in schema order.
  static const List<String> columnNames = ['customer_id', 'display_name', 'order_count', 'spent_cents'];

  /// Every row, naming its columns, so a schema change is a
  /// compile-time-visible change here rather than a runtime surprise.
  static const String selectAllSql =
      'SELECT customer_id, display_name, order_count, spent_cents FROM customer_spend';

  /// This row as database values, keyed by real column name.
  Map<String, Object?> toRow() => {
    'customer_id': customerId,
    'display_name': displayName,
    'order_count': orderCount,
    'spent_cents': spentCents,
  };

  /// One database row decoded, or a [DecodeError] when it does not
  /// match the schema this class was generated from.
  static Result<CustomerSpendRow, DecodeError> fromRow(Map<String, Object?> row) =>
      switch (row) {
        {
          'customer_id': final String? customerId,
          'display_name': final String? displayName,
          'order_count': final int? orderCount,
          'spent_cents': final int? spentCents,
        } =>
          Ok(CustomerSpendRow(
            customerId: customerId,
            displayName: displayName,
            orderCount: orderCount,
            spentCents: spentCents,
          )),
        _ => Err(DecodeError('CustomerSpendRow', 'a row of customer_spend', row)),
      };
}
