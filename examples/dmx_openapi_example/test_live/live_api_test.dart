/// The generated client against the real Frankfurter API.
///
/// Tagged `live` and excluded from the default run, because a test that needs
/// the internet is not the same kind of test as the rest of this suite: it can
/// fail for reasons that have nothing to do with this repository, and a
/// release gate that depends on somebody else's uptime is a release gate that
/// blocks a tag on their bad afternoon.
///
/// Run it deliberately:
///
/// ```sh
/// make example-openapi-live      # or: dart test --tags live
/// ```
///
/// The assertions are still deterministic. Frankfurter serves European Central
/// Bank reference rates, and a rate that was published for a date in the past
/// does not change — `USD/EUR` on 1999-01-04 is a fact, not a reading. What is
/// checked here is therefore exact where the data is settled, and structural
/// where it is not.
@Tags(['live'])
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_openapi_example/api.dart';
import 'package:dmx_openapi_example/http_transport.dart';
import 'package:test/test.dart';

void main() {
  late HttpTransport transport;
  late FrankfurterClient client;

  setUp(() {
    transport = HttpTransport();
    client = FrankfurterClient(transport);
  });

  tearDown(() => transport.close());

  /// Fails with the API error rather than a null dereference, so a broken run
  /// says what the API did.
  T value<T>(Result<T, ApiError> result) => switch (result) {
        Ok(value: final value) => value,
        Err(error: final error) => fail('the API call failed: $error'),
      };

  test('getRate returns the rate the ECB published on 1999-01-04', () async {
    // The first day the euro had a reference rate. It will never change.
    final rate = value(
      await client.getRate(base: 'USD', quote: 'EUR', date: '1999-01-04'),
    );
    expect(rate.base, 'USD');
    expect(rate.quote, 'EUR');
    expect(rate.date, DateTime(1999, 1, 4));
    expect(rate.rate, closeTo(0.85053, 0.00001));
  });

  test('getRates returns a range, one rate per published day', () async {
    final rates = value(
      await client.getRates(
        from: '1999-01-04',
        to: '1999-01-08',
        base: 'USD',
        quotes: 'EUR',
      ),
    );
    expect(rates, isNotEmpty);
    for (final rate in rates) {
      expect(rate.base, 'USD');
      expect(rate.quote, 'EUR');
      expect(rate.rate, greaterThan(0));
      expect(rate.date.isBefore(DateTime(1999, 1, 9)), isTrue);
      expect(rate.date.isAfter(DateTime(1999, 1, 3)), isTrue);
    }
  });

  test('getCurrency returns the euro, symbol and all', () async {
    final currency = value(await client.getCurrency(code: 'EUR'));
    expect(currency.isoCode, 'EUR');
    expect(currency.name, 'Euro');
    expect(currency.isoNumeric, '978');
    expect(currency.symbol, '€');
    expect(currency.providers, isNotEmpty);
  });

  test('getCurrencies returns a list including the majors', () async {
    final currencies = value(await client.getCurrencies());
    expect(currencies, isNotEmpty);
    final codes = [for (final currency in currencies) currency.isoCode];
    expect(codes, containsAll(<String>['EUR', 'USD', 'GBP', 'JPY']));
    for (final currency in currencies) {
      expect(currency.isoCode, hasLength(3));
      expect(currency.name, isNotEmpty);
    }
  });

  test('getProviders returns providers, the ECB among them', () async {
    final providers = value(await client.getProviders());
    expect(providers, isNotEmpty);
    final keys = [for (final provider in providers) provider.key];
    expect(keys, contains('ECB'));
    for (final provider in providers) {
      expect(provider.currencies, isNotEmpty);
      expect(
        provider.publishCadence,
        anyOf(isNull, isIn(Provider.publishCadenceValues)),
        reason: '`${provider.key}` published a cadence the document forbids',
      );
    }
  });

  test('a currency that does not exist is a status failure, not a crash',
      () async {
    // The generated client classifies rather than throws: a 404 is an
    // ApiStatusFailure, distinct from a transport or a decode failure.
    switch (await client.getCurrency(code: 'ZZZ')) {
      case Ok(value: final currency):
        fail('expected a failure, got $currency');
      case Err(error: final ApiStatusFailure failure):
        expect(failure.status, 404);
      case Err(error: final error):
        fail('expected a status failure, got $error');
    }
  });
}
