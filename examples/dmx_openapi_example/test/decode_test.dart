/// [dartmacros.pipeline]: the generated decoders read real payloads.
///
/// Black-box over the generated code, against bytes the live API actually
/// returned — captured into `test/fixtures/` so this suite is hermetic and
/// deterministic. `live_api_test.dart` proves the same code against the API
/// itself.
///
/// The failure cases matter as much as the successes. A generated decoder that
/// accepts a payload it should reject is worse than one that does not compile,
/// because nothing tells you.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:dmx_openapi_example/api.dart';
import 'package:test/test.dart';

/// One captured payload, decoded from JSON.
Object? fixture(String name) =>
    jsonDecode(File('test/fixtures/$name.json').readAsStringSync());

void main() {
  group('Rate', () {
    test('decodes the rate the API returned for 1999-01-04', () {
      final decoded = Rate.fromJson(fixture('rate_usd_eur_1999'));
      expect(decoded, isA<Ok<Rate, DecodeError>>());
      switch (decoded) {
        case Ok(value: final rate):
          expect(rate.base, 'USD');
          expect(rate.quote, 'EUR');
          expect(rate.rate, closeTo(0.85053, 0.000001));
          // `format: date` in the document, so the generated field is a
          // DateTime rather than the string the wire carried. A date-only
          // string carries no zone, and `DateTime.parse` reads that as local.
          expect(rate.date, DateTime(1999, 1, 4));
          expect(rate.date.isUtc, isFalse);
          expect(rate.providers, isNull);
        case Err(:final error):
          fail('expected a rate, got $error');
      }
    });

    test('names the property that is wrong', () {
      final decoded = Rate.fromJson({
        'date': '1999-01-04',
        'base': 'USD',
        'quote': 'EUR',
        'rate': 'not a number',
      });
      switch (decoded) {
        case Ok(value: final rate):
          fail('expected a failure, got $rate');
        case Err(:final error):
          expect(error.path, 'Rate.rate');
          expect(error.expected, 'double');
      }
    });

    test('refuses a payload missing a required property', () {
      final decoded = Rate.fromJson({'base': 'USD', 'quote': 'EUR'});
      expect(decoded, isA<Err<Rate, DecodeError>>());
    });

    test('refuses a value that is not an object at all', () {
      expect(Rate.fromJson('nope'), isA<Err<Rate, DecodeError>>());
      expect(Rate.fromJson(null), isA<Err<Rate, DecodeError>>());
    });

    test('decodes the inline provider objects the document declares', () {
      final decoded = Rate.fromJson({
        'date': '2024-01-15',
        'base': 'EUR',
        'quote': 'USD',
        'rate': 1.09,
        'providers': [
          {'key': 'ECB', 'date': '2024-01-15', 'rate': 1.09},
          {'key': 'FED', 'date': '2024-01-15', 'rate': 1.10, 'excluded': true},
        ],
      });
      switch (decoded) {
        case Ok(value: final rate):
          // RateProvider is a class the document never named — the macro
          // named it after where it was found.
          expect(rate.providers, hasLength(2));
          expect(rate.providers?.first.key, 'ECB');
          expect(rate.providers?.first.excluded, isNull);
          expect(rate.providers?.last.excluded, isTrue);
        case Err(:final error):
          fail('expected a rate, got $error');
      }
    });

    test('a bad element fails the whole list, naming its index', () {
      final decoded = Rate.fromJson({
        'date': '2024-01-15',
        'base': 'EUR',
        'quote': 'USD',
        'rate': 1.09,
        'providers': [
          {'key': 'ECB', 'date': '2024-01-15', 'rate': 1.09},
          {'key': 'FED', 'date': '2024-01-15'},
        ],
      });
      switch (decoded) {
        case Ok(value: final rate):
          fail('expected a failure, got $rate');
        case Err(:final error):
          expect(error.path, contains('[1]'));
          expect(error.path, contains('rate'));
      }
    });
  });

  group('CurrencyDetail', () {
    test('decodes the currency the API returned for EUR', () {
      final decoded = CurrencyDetail.fromJson(fixture('currency_eur'));
      switch (decoded) {
        case Ok(value: final currency):
          expect(currency.isoCode, 'EUR');
          expect(currency.name, 'Euro');
          expect(currency.symbol, '€');
          expect(currency.isoNumeric, '978');
          expect(currency.providers, isNotEmpty);
          // `peg` is absent for the euro, and absent is a value here.
          expect(currency.peg, isNull);
        case Err(:final error):
          fail('expected a currency, got $error');
      }
    });

    test('a null in a nullable property decodes as null', () {
      final decoded = CurrencyDetail.fromJson({
        'iso_code': 'XYZ',
        'name': 'Test',
        'iso_numeric': null,
        'symbol': null,
      });
      switch (decoded) {
        case Ok(value: final currency):
          expect(currency.isoNumeric, isNull);
          expect(currency.symbol, isNull);
        case Err(:final error):
          fail('expected a currency, got $error');
      }
    });

    test('decodes the inline peg object', () {
      final decoded = CurrencyDetail.fromJson({
        'iso_code': 'BAM',
        'name': 'Bosnia-Herzegovina Convertible Mark',
        'peg': {
          'base': 'EUR',
          'rate': 1.95583,
          'authority': 'Central Bank of Bosnia and Herzegovina',
          'source': 'https://www.cbbh.ba/',
        },
      });
      switch (decoded) {
        case Ok(value: final currency):
          expect(currency.peg?.base, 'EUR');
          expect(currency.peg?.rate, closeTo(1.95583, 0.00001));
          // `format: uri` in the document, so the generated field is a Uri.
          expect(currency.peg?.source, Uri.parse('https://www.cbbh.ba/'));
        case Err(:final error):
          fail('expected a currency, got $error');
      }
    });
  });

  group('Provider', () {
    test('decodes every provider the API returned', () {
      final decoded =
          dmxListOf(Provider.fromJson)(fixture('providers'), 'list');
      switch (decoded) {
        case Ok(value: final providers):
          expect(providers, isNotEmpty);
          for (final provider in providers) {
            expect(provider.key, isNotEmpty);
            expect(provider.name, isNotEmpty);
            expect(provider.currencies, isNotEmpty);
            // Nullable in the document, and genuinely null in the data.
            expect(
              provider.publishCadence,
              anyOf(isNull, isIn(Provider.publishCadenceValues)),
            );
          }
        case Err(:final error):
          fail('expected providers, got $error');
      }
    });

    test('carries the enum the document declares', () {
      expect(
        Provider.publishCadenceValues,
        containsAll(<String>['daily', 'weekly', 'monthly']),
      );
    });
  });
}
