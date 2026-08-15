/// Naming the generated classes and files after the tables [dartmacros.files].
///
/// The database already knows what everything is called, so nothing here asks
/// the author to say it: `products` becomes `ProductRow` in
/// `product_row.dart`, `order_lines` becomes `OrderLineRow` in
/// `order_line_row.dart`, and the `customer_spend` view keeps its own name.
///
/// The casing itself comes from `package:dmx/macros.dart`, which mirrors the
/// Rust helpers the built-in catalogue names things with — so a column named
/// by this macro and a field named by `@dmx('model')` are spelled the same
/// way. What is left here is the one rule dmx could not know: how this project
/// turns a table name into a class name.
library;

import 'package:dmx/macros.dart';

/// The suffix every generated row class and file carries.
const String rowSuffix = 'Row';

/// The Dart field name for a column: `price_cents` becomes `priceCents`.
String camelCase(String column) => dmxCamelCase(column);

/// The class a table's rows become: `order_lines` is `OrderLineRow`.
String classNameFor(String table) =>
    '${dmxPascalCase(_singular(table))}$rowSuffix';

/// The file that class lives in: `order_lines` is `order_line_row.dart`.
String fileNameFor(String table) => '${_singular(table)}_row.dart';

/// One row of a plural table: `customers` to `customer`, `boxes` to `box`,
/// `categories` to `category`. A name that is not plural — `customer_spend` —
/// is already the answer.
///
/// Deliberately unclever — a handful of English plural endings, nothing more —
/// because it only ever runs on names read out of the live schema, never on
/// guesses.
String _singular(String table) {
  if (table.endsWith('ies')) {
    return '${table.substring(0, table.length - 3)}y';
  }
  const sibilants = ['ses', 'xes', 'zes', 'ches', 'shes'];
  if (sibilants.any(table.endsWith)) {
    return table.substring(0, table.length - 2);
  }
  return table.endsWith('s') && !table.endsWith('ss')
      ? table.substring(0, table.length - 1)
      : table;
}
