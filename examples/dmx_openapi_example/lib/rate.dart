// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

import 'rate_provider.dart';

/// The `Rate` schema.
///
/// Generated from `components/schemas/Rate` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class Rate {
  /// The date of the rate
  final DateTime date;

  /// Base currency code
  final String base;

  /// Quote currency code
  final String quote;

  /// Exchange rate value
  final double rate;

  /// Per-provider rates for this pair. Present only when
  /// `expand=providers` is set. Each entry has the provider's observation
  /// date and published rate (rebased to the row's base). Entries with
  /// `excluded: true` did not contribute to the blended `rate` — either
  /// flagged as outliers by the consensus filter, or overridden by a
  /// currency peg. Omitted on synthesized peg rows where no provider
  /// published the quote.
  final List<RateProvider>? providers;

  /// Builds a `Rate`.
  const Rate({
    required this.date,
    required this.base,
    required this.quote,
    required this.rate,
    this.providers,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'Rate';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['date', 'base', 'quote', 'rate', 'providers'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<Rate, DecodeError> fromJson(
    Object? json, [
    String path = 'Rate',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'Rate', json));
    }
    final date = dmxDateTime(json['date'], '$path.date');
    final base = dmxString(json['base'], '$path.base');
    final quote = dmxString(json['quote'], '$path.quote');
    final rate = dmxDouble(json['rate'], '$path.rate');
    final providers = dmxNullable(json['providers'], '$path.providers', dmxListOf(RateProvider.fromJson));
    return switch ((
      date,
      base,
      quote,
      rate,
      providers,
    )) {
      (
        Ok(value: final date),
        Ok(value: final base),
        Ok(value: final quote),
        Ok(value: final rate),
        Ok(value: final providers),
      ) =>
        Ok(Rate(
          date: date,
          base: base,
          quote: quote,
          rate: rate,
          providers: providers,
        )),
      _ => Err(dmxFirstError(
          [date, base, quote, rate, providers],
          path,
          'Rate',
        )),
    };
  }
}
