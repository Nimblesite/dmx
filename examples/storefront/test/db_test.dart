/// [catalogue.table]: DDL, statements, and row mapping from one field list.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/db.dart';
import 'package:test/test.dart';

final row = ProductRow(
  id: 'p1',
  title: 'Kettle',
  priceCents: 2500,
  currency: 'GBP',
  publishedAt: DateTime.utc(2024, 3),
  inStock: true,
  description: 'Boils water.',
);

void main() {
  group('schema', () {
    test('every field becomes a column, in declaration order', () {
      expect(ProductRow.columns, <String>[
        'id',
        'title',
        'price_cents',
        'currency',
        'published_at',
        'in_stock',
        'description',
      ]);
    });

    test('Dart nullability becomes SQL nullability', () {
      expect(ProductRow.createTableSql, contains('title TEXT NOT NULL'));
      expect(ProductRow.createTableSql, contains('description TEXT\n'));
      expect(
          ProductRow.createTableSql, isNot(contains('description TEXT NOT')));
    });

    test('Dart types become SQL types', () {
      expect(ProductRow.createTableSql, contains('price_cents INTEGER'));
      expect(ProductRow.createTableSql, contains('in_stock INTEGER'));
      expect(ProductRow.createTableSql, contains('published_at TEXT'));
    });

    test('the primary key is declared, not assumed', () {
      expect(
        ProductRow.createTableSql,
        contains('id TEXT NOT NULL PRIMARY KEY'),
      );
    });

    test("@dmx('column', {'indexed': true}) emits an index statement", () {
      expect(ProductRow.createIndexSql, <String>[
        'CREATE INDEX IF NOT EXISTS products_title_idx ON products (title)',
      ]);
    });

    test("@dmx('column', {'references': ...}) emits a foreign key", () {
      expect(
        OrderLineRow.createTableSql,
        contains('order_id TEXT NOT NULL REFERENCES orders(id)'),
      );
    });

    test('the migration is assembled from the tables in the file', () {
      expect(schemaStatements, hasLength(4));
      expect(schemaStatements.first, ProductRow.createTableSql);
    });
  });

  group('statements', () {
    test('values are bound, never interpolated', () {
      expect(ProductRow.insertSql, endsWith('VALUES (?, ?, ?, ?, ?, ?, ?)'));
      expect(ProductRow.insertSql, isNot(contains('Kettle')));
    });

    test('the placeholder count matches the column count', () {
      expect(
        '?'.allMatches(ProductRow.insertSql).length,
        ProductRow.columns.length,
      );
      expect(row.insertArgs, hasLength(ProductRow.columns.length));
    });

    test('the upsert updates every column except the key', () {
      expect(ProductRow.upsertSql, contains('ON CONFLICT(id) DO UPDATE SET'));
      expect(ProductRow.upsertSql, contains('title = excluded.title'));
      expect(ProductRow.upsertSql, isNot(contains('id = excluded.id')));
    });

    test('select and delete are keyed on the declared primary key', () {
      expect(ProductRow.selectByIdSql, endsWith('WHERE id = ?'));
      expect(ProductRow.deleteByIdSql, 'DELETE FROM products WHERE id = ?');
    });
  });

  group('row mapping', () {
    test('encodes to storage types, not Dart types', () {
      expect(row.toRow()['in_stock'], 1);
      expect(row.toRow()['published_at'], '2024-03-01T00:00:00.000Z');
    });

    test('insertArgs is toRow in column order', () {
      expect(
        row.insertArgs,
        ProductRow.columns.map((column) => row.toRow()[column]),
      );
    });

    test('round-trips through a row', () {
      expect(ProductRow.fromRow(row.toRow()), Ok<ProductRow, DecodeError>(row));
    });

    test('a bad row is a decode failure naming the path', () {
      final bad = row.toRow()..['published_at'] = 'whenever';
      expect(
        ProductRow.fromRow(bad),
        isA<Err<ProductRow, DecodeError>>()
            .having((e) => e.error.path, 'path', 'ProductRow.published_at'),
      );
    });

    test('a missing column fails at the class', () {
      final bad = row.toRow()..remove('currency');
      expect(
        ProductRow.fromRow(bad),
        isA<Err<ProductRow, DecodeError>>()
            .having((e) => e.error.expected, 'expected', 'ProductRow'),
      );
    });

    test('a result set decodes row by row and says which one failed', () {
      expect(
        ProductRow.fromRows(<Map<String, Object?>>[
          row.toRow(),
          row.toRow()..['title'] = 7,
        ]),
        isA<Err<List<ProductRow>, DecodeError>>()
            .having((e) => e.error.path, 'path', 'ProductRow[1]'),
      );
    });

    test('a clean result set decodes whole', () {
      expect(
        ProductRow.fromRows(<Map<String, Object?>>[row.toRow(), row.toRow()]),
        isA<Ok<List<ProductRow>, DecodeError>>()
            .having((r) => r.value, 'value', hasLength(2)),
      );
    });

    test('a table with no nullable columns needs no sequencing at all', () {
      const line = OrderLineRow(
        id: 1,
        orderId: 'o-1',
        productId: 'p1',
        quantity: 2,
        unitPriceCents: 2500,
      );
      expect(
        OrderLineRow.fromRow(line.toRow()),
        Ok<OrderLineRow, DecodeError>(line),
      );
    });
  });
}
