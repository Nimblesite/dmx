// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('strings')` [catalogue.strings] — localisation with the placeholders typed.
//
// ARB files and their generators exist, and they work, and they are also a
// separate file format, a separate build step, and a separate place to be
// wrong. Here the message *is* a Dart method signature: `{count}` in the
// template must correspond to a parameter called `count`, and a template that
// mentions a placeholder the method does not have is a generation-time error
// rather than a `{count}` printed to a customer.
//
// Pluralisation is a `switch` on the count, which is all pluralisation is in
// the languages this file covers — and the shape scales to the ones where it
// is not, because the arms are generated from the categories the locale
// declares.

import 'package:dmx/dmx.dart';

/// Every string in the app. Hand-written, and the only thing UI code imports.
abstract interface class AppStrings {
  @dmx('message', {'template': 'Add to cart'})
  String addToCart();

  @dmx('message', {'template': 'Ships to {city}, {country}'})
  String shipsTo({required String city, required String country});

  @dmx('message.plural', {'zero': 'Your cart is empty', 'one': '1 item in your cart', 'other': '{count} items in your cart', 'description': 'Shown on the cart button.'})
  String cartCount(int count);

  @dmx('message', {'template': '{name} saved £{amount}'})
  String savings({required String name, required String amount});
}

/// The reference locale.
@dmx('strings', {'locale': 'en'})
class AppStringsEn implements AppStrings {
  const AppStringsEn();

  //#region
  static const String locale = 'en';

  @override
  String addToCart() => 'Add to cart';

  @override
  String shipsTo({required String city, required String country}) =>
      'Ships to $city, $country';

  @override
  String cartCount(int count) => switch (count) {
        0 => 'Your cart is empty',
        1 => '1 item in your cart',
        _ => '$count items in your cart',
      };

  @override
  String savings({required String name, required String amount}) =>
      '$name saved £$amount';
  //#endregion
}

/// A translation. The signatures are not repeated here — they come from the
/// interface, so a message added upstream is a compile error in every locale
/// that has not caught up, which is the correct time to find out.
@dmx('strings', {'locale': 'de'})
class AppStringsDe implements AppStrings {
  const AppStringsDe();

  //#region
  static const String locale = 'de';

  @override
  String addToCart() => 'In den Warenkorb';

  @override
  String shipsTo({required String city, required String country}) =>
      'Versand nach $city, $country';

  @override
  String cartCount(int count) => switch (count) {
        0 => 'Ihr Warenkorb ist leer',
        1 => '1 Artikel in Ihrem Warenkorb',
        _ => '$count Artikel in Ihrem Warenkorb',
      };

  @override
  String savings({required String name, required String amount}) =>
      '$name hat £$amount gespart';
  //#endregion
}

/// Hand-written: which locale to use is an application decision, and the
/// fallback chain is exactly the sort of thing that should not be generated
/// behind somebody's back.
const Map<String, AppStrings> stringsByLocale = <String, AppStrings>{
  'en': AppStringsEn(),
  'de': AppStringsDe(),
};

AppStrings stringsFor(String locale) =>
    stringsByLocale[locale] ??
    stringsByLocale[locale.split('-').first] ??
    const AppStringsEn();
