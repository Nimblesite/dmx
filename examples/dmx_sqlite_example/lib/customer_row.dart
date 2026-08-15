// dmx: generated from schema.dart — do not edit.

import 'package:dmx/dmx.dart';

/// One row of `customers`, exactly as the database stores it.
///
/// Everything here follows from the live schema — even this file's
/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.
final class CustomerRow {
  /// `customers.id` — TEXT NOT NULL, primary key.
  final String id;

  /// `customers.email` — TEXT NOT NULL.
  final String email;

  /// `customers.display_name` — TEXT NOT NULL.
  final String displayName;

  /// `customers.signed_up_at` — TEXT NOT NULL.
  final String signedUpAt;

  /// `customers.marketing_opt_in` — BOOLEAN NOT NULL.
  final bool marketingOptIn;

  /// `customers.loyalty_points` — INTEGER.
  final int? loyaltyPoints;

  /// Builds a row of `customers`. Every parameter is one of its columns.
  const CustomerRow({
    required this.id,
    required this.email,
    required this.displayName,
    required this.signedUpAt,
    required this.marketingOptIn,
    this.loyaltyPoints,
  });

  /// The table these rows come from.
  static const String tableName = 'customers';

  /// Every column `customers` actually has, in schema order.
  static const List<String> columnNames = ['id', 'email', 'display_name', 'signed_up_at', 'marketing_opt_in', 'loyalty_points'];

  /// The primary key, in key order.
  static const List<String> primaryKeyColumns = ['id'];

  /// Every row, naming its columns, so a schema change is a
  /// compile-time-visible change here rather than a runtime surprise.
  static const String selectAllSql =
      'SELECT id, email, display_name, signed_up_at, marketing_opt_in, loyalty_points FROM customers';

  /// One row by its primary key, for `keyValues` in that order.
  static const String selectByKeySql =
      'SELECT id, email, display_name, signed_up_at, marketing_opt_in, loyalty_points FROM customers WHERE id = ?';

  /// An INSERT of every column, for `insertValues` in that order.
  static const String insertSql =
      'INSERT INTO customers (id, email, display_name, signed_up_at, marketing_opt_in, loyalty_points) VALUES (?, ?, ?, ?, ?, ?)';

  /// The values `insertSql` takes, in its parameter order.
  List<Object?> get insertValues => toRow().values.toList(growable: false);

  /// This row's primary key, for `selectByKeySql` in that order.
  List<Object?> get keyValues => [id];

  /// This row as database values, keyed by real column name.
  Map<String, Object?> toRow() => {
    'id': id,
    'email': email,
    'display_name': displayName,
    'signed_up_at': signedUpAt,
    'marketing_opt_in': marketingOptIn ? 1 : 0,
    'loyalty_points': loyaltyPoints,
  };

  /// One database row decoded, or a [DecodeError] when it does not
  /// match the schema this class was generated from.
  static Result<CustomerRow, DecodeError> fromRow(Map<String, Object?> row) =>
      switch (row) {
        {
          'id': final String id,
          'email': final String email,
          'display_name': final String displayName,
          'signed_up_at': final String signedUpAt,
          'marketing_opt_in': final int marketingOptIn,
          'loyalty_points': final int? loyaltyPoints,
        } =>
          Ok(CustomerRow(
            id: id,
            email: email,
            displayName: displayName,
            signedUpAt: signedUpAt,
            marketingOptIn: marketingOptIn != 0,
            loyaltyPoints: loyaltyPoints,
          )),
        _ => Err(DecodeError('CustomerRow', 'a row of customers', row)),
      };
}
