/// The macro controls the file names [dartmacros.files].
///
/// lib/ holds exactly one hand-written file — the seed — and everything else
/// was authored AND named by the macro, one file per table. These tests hold
/// the directory itself to that: the file set is a pure function of the live
/// schema, every generated file opens with the driver's ownership marker, and
/// the seed's manifest says where everything went.
library;

import 'dart:io';

import 'package:dmx_sqlite_schema_example/schema.dart';
import 'package:test/test.dart';

import '../tool/dmx/src/database.dart';
import '../tool/dmx/src/naming.dart';
import 'support.dart';

/// The marker line every macro-authored file must open with — the driver
/// writes it, naming the seed [dartmacros.files].
const String marker = '// dmx: generated from schema.dart — do not edit.';

/// Every Dart file in lib/, by bare name.
Map<String, String> libFiles() => {
      for (final entity in Directory('lib').listSync())
        if (entity is File && entity.path.endsWith('.dart'))
          entity.uri.pathSegments.last: entity.readAsStringSync(),
    };

void main() {
  test('one file per table, named after it, plus the barrel and the seed', () {
    final expected = {
      for (final table in tableNames(databasePath)) fileNameFor(table),
      'rows.dart',
      'schema.dart',
    };
    expect(
      libFiles().keys.toSet(),
      expected,
      reason: 'lib/ must hold exactly what the schema implies — no more, '
          'no fewer, no hand-added strays',
    );
  });

  test(
      'every generated file opens with the ownership marker; the seed with '
      'none', () {
    for (final MapEntry(key: name, value: content) in libFiles().entries) {
      if (name == 'schema.dart') {
        expect(
          content.startsWith(marker),
          isFalse,
          reason: 'the seed is the author\'s file',
        );
      } else {
        expect(
          content.startsWith('$marker\n'),
          isTrue,
          reason: '`$name` must open with the marker, or dmx cannot own it',
        );
      }
    }
  });

  test('the seed manifest matches the live schema exactly', () {
    final live = [
      for (final name in tableNames(databasePath))
        readTable(databasePath, name),
    ].whereType<SchemaTable>().toList();
    expect(Schema.tables, [
      for (final table in live)
        if (!table.isView) table.name,
    ]);
    expect(Schema.views, [
      for (final table in live)
        if (table.isView) table.name,
    ]);
    expect(Schema.rowFiles, {
      for (final table in live) table.name: fileNameFor(table.name),
    });
  });

  test('the barrel exports every row file and nothing else', () {
    final barrel = libFiles()['rows.dart'] ?? '';
    final exports = [
      for (final line in barrel.split('\n'))
        if (line.startsWith('export ')) line,
    ];
    expect(exports, [
      for (final file in Schema.rowFiles.values) "export '$file';",
    ]);
  });

  test('tables become class and file names without anyone spelling them', () {
    // The rule the macro applies, pinned: strip one plural ending, pascal-case
    // the words, add the suffix. `customer_spend` is already one row's name.
    const cases = {
      'customers': ('CustomerRow', 'customer_row.dart'),
      'products': ('ProductRow', 'product_row.dart'),
      'orders': ('OrderRow', 'order_row.dart'),
      'order_lines': ('OrderLineRow', 'order_line_row.dart'),
      'customer_spend': ('CustomerSpendRow', 'customer_spend_row.dart'),
      'boxes': ('BoxRow', 'box_row.dart'),
      'categories': ('CategoryRow', 'category_row.dart'),
      'statuses': ('StatusRow', 'status_row.dart'),
      'address': ('AddressRow', 'address_row.dart'),
    };
    for (final MapEntry(key: table, value: (className, file))
        in cases.entries) {
      expect(classNameFor(table), className);
      expect(fileNameFor(table), file);
    }
  });

  test('the column-to-field rule, pinned', () {
    expect(camelCase('price_cents'), 'priceCents');
    expect(camelCase('id'), 'id');
    expect(camelCase('unit_price_cents'), 'unitPriceCents');
  });
}
