// The ONLY hand-written file in lib/. Every `*_row.dart` beside it — and
// `rows.dart` — is authored, NAMED, and kept current by `tool/dmx/macros.dart`
// from the live database: one file per table, one class per file, nobody
// types a table name anywhere. Drop a table and dmx collects its file.
//
// tool/dmx/schema.sql is the source of truth. Never edit a generated file.

import 'package:dmx/dmx.dart';

/// The whole database, as one annotation.
///
/// The macro reads every table and view out of the live schema, writes one
/// row class per table in a file it names itself, and leaves the manifest —
/// what exists, and where it went — between the dividers below.
@dmx('sqliteSchema')
class Schema {
  //#region
  /// Every table the database has, in name order.
  static const List<String> tables = ['customers', 'order_lines', 'orders', 'products'];

  /// Every view. A view generates a read-only row class.
  static const List<String> views = ['customer_spend'];

  /// The generated file each row class lives in, keyed by table.
  static const Map<String, String> rowFiles = {
    'customer_spend': 'customer_spend_row.dart',
    'customers': 'customer_row.dart',
    'order_lines': 'order_line_row.dart',
    'orders': 'order_row.dart',
    'products': 'product_row.dart',
  };
  //#endregion
}
