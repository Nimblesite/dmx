// dmx: generated from schema.dart — do not edit.

import 'package:dmx/dmx.dart';

/// One row of `products`, exactly as the database stores it.
///
/// Everything here follows from the live schema — even this file's
/// name. Change `tool/dmx/schema.sql` and rebuild; never edit this.
final class ProductRow {
  /// `products.id` — TEXT NOT NULL, primary key.
  final String id;

  /// `products.title` — TEXT NOT NULL.
  final String title;

  /// `products.price_cents` — INTEGER NOT NULL.
  final int priceCents;

  /// `products.in_stock` — BOOLEAN NOT NULL.
  final bool inStock;

  /// `products.published_at` — TEXT.
  final String? publishedAt;

  /// `products.weight_grams` — REAL.
  final double? weightGrams;

  /// Builds a row of `products`. Every parameter is one of its columns.
  const ProductRow({
    required this.id,
    required this.title,
    required this.priceCents,
    required this.inStock,
    this.publishedAt,
    this.weightGrams,
  });

  /// The table these rows come from.
  static const String tableName = 'products';

  /// Every column `products` actually has, in schema order.
  static const List<String> columnNames = ['id', 'title', 'price_cents', 'in_stock', 'published_at', 'weight_grams'];

  /// The primary key, in key order.
  static const List<String> primaryKeyColumns = ['id'];

  /// Every row, naming its columns, so a schema change is a
  /// compile-time-visible change here rather than a runtime surprise.
  static const String selectAllSql =
      'SELECT id, title, price_cents, in_stock, published_at, weight_grams FROM products';

  /// One row by its primary key, for `keyValues` in that order.
  static const String selectByKeySql =
      'SELECT id, title, price_cents, in_stock, published_at, weight_grams FROM products WHERE id = ?';

  /// An INSERT of every column, for `insertValues` in that order.
  static const String insertSql =
      'INSERT INTO products (id, title, price_cents, in_stock, published_at, weight_grams) VALUES (?, ?, ?, ?, ?, ?)';

  /// The values `insertSql` takes, in its parameter order.
  List<Object?> get insertValues => toRow().values.toList(growable: false);

  /// This row's primary key, for `selectByKeySql` in that order.
  List<Object?> get keyValues => [id];

  /// This row as database values, keyed by real column name.
  Map<String, Object?> toRow() => {
    'id': id,
    'title': title,
    'price_cents': priceCents,
    'in_stock': inStock ? 1 : 0,
    'published_at': publishedAt,
    'weight_grams': weightGrams,
  };

  /// One database row decoded, or a [DecodeError] when it does not
  /// match the schema this class was generated from.
  static Result<ProductRow, DecodeError> fromRow(Map<String, Object?> row) =>
      switch (row) {
        {
          'id': final String id,
          'title': final String title,
          'price_cents': final int priceCents,
          'in_stock': final int inStock,
          'published_at': final String? publishedAt,
          'weight_grams': final double? weightGrams,
        } =>
          Ok(ProductRow(
            id: id,
            title: title,
            priceCents: priceCents,
            inStock: inStock != 0,
            publishedAt: publishedAt,
            weightGrams: weightGrams,
          )),
        _ => Err(DecodeError('ProductRow', 'a row of products', row)),
      };
}
