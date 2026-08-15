// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('model')` [model] — the immutable data class, and `@dmx('diff')` [catalogue.diff].
//
// The decoder is the interesting part. It takes `Object?`, not
// `Map<String, dynamic>`, so a nested model's `fromJson` *is* a decoder and
// composes without a cast at the call site. It carries a `path`, so a failure
// six levels down reports `Product.variants[2].price.amount` instead of
// "type 'Null' is not a subtype of type 'int'". And it returns a `Result`, so
// a malformed payload is a value you can handle rather than an exception you
// remembered to catch.

import 'package:dmx/dmx.dart';

import 'payments.dart';

/// Money in integer minor units, because floating-point money is a bug waiting
/// for a customer to find it.
///
/// `@dmx('diff')` is the second macro on this class: two macros, one region, fragments
/// in the order the annotations were written.
@dmx('model')
@dmx('diff')
class Money {
  const Money({required this.amount, required this.currency});

  final int amount;
  final String currency;

  //#region
  static Result<Money, DecodeError> fromJson(Object? json, [String path = 'Money']) =>
      switch (json) {
        {
          'amount': final int amount,
          'currency': final String currency,
        } =>
          Ok(Money(
            amount: amount,
            currency: currency,
          )),
        _ => Err(DecodeError(path, 'Money', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'amount': amount,
        'currency': currency,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Money &&
          other.amount == amount &&
          other.currency == currency);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        amount,
        currency,
      );

  @override
  String toString() => 'Money(amount: $amount, currency: $currency)';

  Money copyWith({
    int? amount,
    String? currency,
  }) =>
      Money(
        amount: amount ?? this.amount,
        currency: currency ?? this.currency,
      );

  /// Every field that differs, in field order. Collection fields compare by
  /// content, so this agrees with `==` rather than with identity.
  ///
  /// Nothing here is reflective: the field list is fixed at generation time,
  /// so adding a field adds a line on the next build and cannot be forgotten.
  List<DmxChange> diff(Money other) => <DmxChange>[
        if (other.amount != amount)
          DmxChange('amount', amount, other.amount),
        if (other.currency != currency)
          DmxChange('currency', currency, other.currency),
      ];

  /// The names alone, for a "3 unsaved changes" badge that does not need the
  /// values behind them.
  List<String> changedFields(Money other) =>
      <String>[for (final change in diff(other)) change.field];

  bool differsFrom(Money other) => diff(other).isNotEmpty;
  //#endregion
}

/// One buyable configuration of a product.
@dmx('model', {'fieldRename': 'snake'})
class Variant {
  const Variant({
    required this.sku,
    required this.price,
    required this.options,
    this.compareAtPrice,
    this.stock = 0,
  });

  final String sku;
  final Money price;
  final Map<String, String> options;
  final Money? compareAtPrice;
  final int stock;

  //#region
  static Result<Variant, DecodeError> fromJson(Object? json, [String path = 'Variant']) =>
      switch (json) {
        {
          'sku': final String sku,
          'price': final Object? price,
          'options': final Map<String, dynamic> options,
          'stock': final int stock,
        } =>
          switch ((
            Money.fromJson(price, '$path.price'),
            dmxMap<String>(options, '$path.options', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
            dmxNullable<Money>(dmxKey(json, 'compare_at_price'), '$path.compare_at_price', Money.fromJson),
          )) {
            (
              Ok(value: final price),
              Ok(value: final options),
              Ok(value: final compareAtPrice),
            ) =>
              Ok(Variant(
                sku: sku,
                price: price,
                options: options,
                compareAtPrice: compareAtPrice,
                stock: stock,
              )),
            (Err(error: final e), _, _) => Err(e),
            (_, Err(error: final e), _) => Err(e),
            (_, _, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'Variant', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'sku': sku,
        'price': price.toJson(),
        'options': options,
        'compare_at_price': compareAtPrice?.toJson(),
        'stock': stock,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Variant &&
          other.sku == sku &&
          other.price == price &&
          dmxDeepEquals(other.options, options) &&
          other.compareAtPrice == compareAtPrice &&
          other.stock == stock);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        sku,
        price,
        dmxDeepHash(options),
        compareAtPrice,
        stock,
      );

  @override
  String toString() => 'Variant(sku: $sku, price: $price, options: $options, compareAtPrice: $compareAtPrice, stock: $stock)';

  Variant copyWith({
    String? sku,
    Money? price,
    Map<String, String>? options,
    DmxPatch<Money?> compareAtPrice = const DmxKeep(),
    int? stock,
  }) =>
      Variant(
        sku: sku ?? this.sku,
        price: price ?? this.price,
        options: options ?? this.options,
        compareAtPrice: switch (compareAtPrice) { DmxKeep() => this.compareAtPrice, DmxTo(value: final value) => value },
        stock: stock ?? this.stock,
      );
  //#endregion
}

/// A catalogue entry: nested models, a list of models, an enum, a `DateTime`,
/// a `Set`, and a nullable — every decode shape this generator knows, in one
/// class, so the golden output is worth reading.
@dmx('model', {'fieldRename': 'snake'})
class Product {
  const Product({
    required this.id,
    required this.title,
    required this.variants,
    required this.tags,
    required this.acceptedMethods,
    required this.publishedAt,
    this.description,
    this.featuredVariant,
  });

  final String id;
  final String title;
  final List<Variant> variants;
  final Set<String> tags;
  final List<PaymentMethod> acceptedMethods;
  final DateTime publishedAt;
  final String? description;

  /// Excluded from the codec entirely — it is derived, and round-tripping it
  /// would let the two copies disagree.
  @dmx('key', {'ignore': true})
  final Variant? featuredVariant;

  /// Hand-written members live above the divider and are never rewritten.
  Money? get lowestPrice => variants.isEmpty
      ? null
      : variants
          .map((variant) => variant.price)
          .reduce((a, b) => a.amount <= b.amount ? a : b);

  //#region
  static Result<Product, DecodeError> fromJson(Object? json, [String path = 'Product']) =>
      switch (json) {
        {
          'id': final String id,
          'title': final String title,
          'variants': final List<dynamic> variants,
          'tags': final List<dynamic> tags,
          'accepted_methods': final List<dynamic> acceptedMethods,
          'published_at': final String publishedAt,
        } =>
          switch ((
            dmxList<Variant>(variants, '$path.variants', Variant.fromJson),
            dmxSet<String>(tags, '$path.tags', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
            dmxList<PaymentMethod>(acceptedMethods, '$path.accepted_methods', PaymentMethod.fromJson),
            switch (DateTime.tryParse(publishedAt)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError('$path.published_at', 'DateTime', publishedAt)) },
            dmxNullable<String>(dmxKey(json, 'description'), '$path.description', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final variants),
              Ok(value: final tags),
              Ok(value: final acceptedMethods),
              Ok(value: final publishedAt),
              Ok(value: final description),
            ) =>
              Ok(Product(
                id: id,
                title: title,
                variants: variants,
                tags: tags,
                acceptedMethods: acceptedMethods,
                publishedAt: publishedAt,
                description: description,
              )),
            (Err(error: final e), _, _, _, _) => Err(e),
            (_, Err(error: final e), _, _, _) => Err(e),
            (_, _, Err(error: final e), _, _) => Err(e),
            (_, _, _, Err(error: final e), _) => Err(e),
            (_, _, _, _, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'Product', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'title': title,
        'variants': variants.map((e0) => e0.toJson()).toList(),
        'tags': tags.toList(),
        'accepted_methods': acceptedMethods.map((e0) => e0.toJson()).toList(),
        'published_at': publishedAt.toIso8601String(),
        'description': description,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Product &&
          other.id == id &&
          other.title == title &&
          dmxDeepEquals(other.variants, variants) &&
          dmxDeepEquals(other.tags, tags) &&
          dmxDeepEquals(other.acceptedMethods, acceptedMethods) &&
          other.publishedAt == publishedAt &&
          other.description == description);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        title,
        dmxDeepHash(variants),
        dmxDeepHash(tags),
        dmxDeepHash(acceptedMethods),
        publishedAt,
        description,
      );

  @override
  String toString() => 'Product(id: $id, title: $title, variants: $variants, tags: $tags, acceptedMethods: $acceptedMethods, publishedAt: $publishedAt, description: $description)';

  Product copyWith({
    String? id,
    String? title,
    List<Variant>? variants,
    Set<String>? tags,
    List<PaymentMethod>? acceptedMethods,
    DateTime? publishedAt,
    DmxPatch<String?> description = const DmxKeep(),
  }) =>
      Product(
        id: id ?? this.id,
        title: title ?? this.title,
        variants: variants ?? this.variants,
        tags: tags ?? this.tags,
        acceptedMethods: acceptedMethods ?? this.acceptedMethods,
        publishedAt: publishedAt ?? this.publishedAt,
        description: switch (description) { DmxKeep() => this.description, DmxTo(value: final value) => value },
      );
  //#endregion
}
