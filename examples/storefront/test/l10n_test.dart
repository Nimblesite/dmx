/// [catalogue.strings]: localisation with the placeholders typed.
library;

import 'package:dmx_storefront_example/l10n.dart';
import 'package:test/test.dart';

void main() {
  test('a message with no placeholders is a constant string', () {
    expect(const AppStringsEn().addToCart(), 'Add to cart');
  });

  test('placeholders are interpolated in the order the template names them',
      () {
    expect(
      const AppStringsEn().shipsTo(city: 'London', country: 'GB'),
      'Ships to London, GB',
    );
  });

  test('a plural message picks the category from the count', () {
    const strings = AppStringsEn();
    expect(strings.cartCount(0), 'Your cart is empty');
    expect(strings.cartCount(1), '1 item in your cart');
    expect(strings.cartCount(7), '7 items in your cart');
  });

  test('a literal that looks like a placeholder survives', () {
    expect(
      const AppStringsEn().savings(name: 'Ada', amount: '12.50'),
      'Ada saved £12.50',
    );
  });

  group('translations', () {
    test('every locale implements the same interface', () {
      final locales = <AppStrings>[const AppStringsEn(), const AppStringsDe()];
      for (final strings in locales) {
        expect(strings.addToCart(), isNotEmpty);
        expect(strings.cartCount(2), isNotEmpty);
      }
    });

    test('a translation has its own wording for every category', () {
      const strings = AppStringsDe();
      expect(strings.cartCount(0), 'Ihr Warenkorb ist leer');
      expect(strings.cartCount(1), '1 Artikel in Ihrem Warenkorb');
      expect(strings.cartCount(4), '4 Artikel in Ihrem Warenkorb');
    });

    test('placeholders survive translation', () {
      expect(
        const AppStringsDe().shipsTo(city: 'Berlin', country: 'DE'),
        'Versand nach Berlin, DE',
      );
    });
  });

  group('lookup', () {
    test('an exact locale wins', () {
      expect(stringsFor('de'), isA<AppStringsDe>());
    });

    test('a regional locale falls back to its language', () {
      expect(stringsFor('de-AT'), isA<AppStringsDe>());
    });

    test('an unknown locale falls back to the reference locale', () {
      expect(stringsFor('ja'), isA<AppStringsEn>());
    });

    test('every registered locale answers to its own tag', () {
      for (final entry in stringsByLocale.entries) {
        expect(stringsFor(entry.key), same(entry.value));
      }
    });
  });
}
