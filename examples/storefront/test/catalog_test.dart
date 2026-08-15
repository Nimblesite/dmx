/// [model.json-codec], [model.copywith], [catalogue.diff]: the data class.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/catalog.dart';
import 'package:dmx_storefront_example/payments.dart';
import 'package:test/test.dart';

Map<String, dynamic> productJson() => <String, dynamic>{
      'id': 'p1',
      'title': 'Kettle',
      'variants': <dynamic>[
        <String, dynamic>{
          'sku': 'kettle-black',
          'price': <String, dynamic>{'amount': 2500, 'currency': 'GBP'},
          'options': <String, dynamic>{'colour': 'black'},
          'stock': 3,
        },
        <String, dynamic>{
          'sku': 'kettle-steel',
          'price': <String, dynamic>{'amount': 3200, 'currency': 'GBP'},
          'options': <String, dynamic>{'colour': 'steel'},
          'compare_at_price': <String, dynamic>{
            'amount': 3900,
            'currency': 'GBP',
          },
          'stock': 0,
        },
      ],
      'tags': <dynamic>['home', 'kitchen'],
      'accepted_methods': <dynamic>['card', 'apple_pay'],
      'published_at': '2024-03-01T00:00:00.000Z',
      'description': 'Boils water.',
    };

Product decoded() => switch (Product.fromJson(productJson())) {
      Ok(value: final product) => product,
      Err(error: final error) => fail('$error'),
    };

void main() {
  group('decoding', () {
    test('decodes a nested, collection-bearing payload', () {
      final product = decoded();
      expect(product.id, 'p1');
      expect(product.variants, hasLength(2));
      expect(product.variants.first.price,
          const Money(amount: 2500, currency: 'GBP'));
      expect(product.tags, <String>{'home', 'kitchen'});
      expect(product.acceptedMethods, <PaymentMethod>[
        PaymentMethod.card,
        PaymentMethod.applePay,
      ]);
      expect(product.publishedAt.year, 2024);
    });

    test('an absent nullable key decodes to null rather than failing', () {
      final json = productJson()..remove('description');
      expect(
        Product.fromJson(json),
        isA<Ok<Product, DecodeError>>()
            .having((r) => r.value.description, 'description', isNull),
      );
    });

    test('a failure names the path it was reached by, not the type', () {
      final json = productJson();
      final variants = json['variants'];
      expect(variants, isA<List<dynamic>>());
      if (variants case final List<dynamic> variants) {
        final second = variants[1];
        if (second case final Map<String, dynamic> second) {
          second['price'] = <String, dynamic>{'amount': 'lots'};
        }
      }
      expect(
        Product.fromJson(json),
        isA<Err<Product, DecodeError>>()
            .having((e) => e.error.path, 'path', 'Product.variants[1].price')
            .having((e) => e.error.expected, 'expected', 'Money'),
      );
    });

    test('a malformed date fails at the field, with the field name', () {
      final json = productJson()..['published_at'] = 'the ides of March';
      expect(
        Product.fromJson(json),
        isA<Err<Product, DecodeError>>()
            .having((e) => e.error.path, 'path', 'Product.published_at')
            .having((e) => e.error.expected, 'expected', 'DateTime'),
      );
    });

    test('a missing required key fails at the class, not with a null error',
        () {
      final json = productJson()..remove('title');
      expect(
        Product.fromJson(json),
        isA<Err<Product, DecodeError>>()
            .having((e) => e.error.expected, 'expected', 'Product'),
      );
    });

    test('a decoder composes over anything, including the wrong shape', () {
      expect(
          Product.fromJson('not an object'), isA<Err<Product, DecodeError>>());
      expect(Product.fromJson(null), isA<Err<Product, DecodeError>>());
    });
  });

  group('encoding', () {
    test('round-trips through JSON unchanged', () {
      final product = decoded();
      expect(Product.fromJson(product.toJson()),
          Ok<Product, DecodeError>(product));
    });

    test("@dmx('key', {'ignore': true}) keeps a derived field out of the wire format", () {
      expect(decoded().toJson().containsKey('featured_variant'), isFalse);
    });

    test('fieldRename applies to every key', () {
      expect(decoded().toJson().keys, contains('accepted_methods'));
      expect(decoded().toJson().keys, contains('published_at'));
    });
  });

  group('value semantics', () {
    test('two identical products are equal and hash alike', () {
      expect(decoded(), decoded());
      expect(decoded().hashCode, decoded().hashCode);
    });

    test('collections compare by content, not identity', () {
      final product = decoded();
      final rebuilt = product.copyWith(tags: <String>{...product.tags});
      expect(rebuilt, product);
      expect(rebuilt.hashCode, product.hashCode);
    });
  });

  group('copyWith', () {
    test('an omitted nullable field is kept', () {
      expect(decoded().copyWith().description, 'Boils water.');
    });

    test('DmxTo(null) clears it — which `null` alone could never express', () {
      expect(decoded().copyWith(description: const DmxTo(null)).description,
          isNull);
    });

    test('DmxTo(value) sets it', () {
      expect(
        decoded()
            .copyWith(description: const DmxTo('Now with a lid.'))
            .description,
        'Now with a lid.',
      );
    });
  });

  group("@dmx('diff')", () {
    test('reports only what changed, with both values', () {
      const before = Money(amount: 2500, currency: 'GBP');
      const after = Money(amount: 2600, currency: 'GBP');
      expect(before.diff(after), <DmxChange>[
        const DmxChange('amount', 2500, 2600),
      ]);
    });

    test('identical values produce no changes', () {
      const money = Money(amount: 2500, currency: 'GBP');
      expect(money.diff(money), isEmpty);
    });
  });

  test('hand-written members are untouched by generation', () {
    expect(decoded().lowestPrice, const Money(amount: 2500, currency: 'GBP'));
  });
}
