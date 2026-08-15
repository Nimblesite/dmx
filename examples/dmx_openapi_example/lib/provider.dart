// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

/// The `Provider` schema.
///
/// Generated from `components/schemas/Provider` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class Provider {
  /// Provider identifier
  final String key;

  /// Full provider name
  final String name;

  /// ISO 3166-1 alpha-2 country code
  final String? countryCode;

  /// Official rate type as used by the source
  final String? rateType;

  /// Base currency for published rates
  final String? pivotCurrency;

  /// Link to the data source
  final Uri? dataUrl;

  /// Link to terms of use
  final Uri? termsUrl;

  /// Earliest available date
  final DateTime? startDate;

  /// Latest available date
  final DateTime? endDate;

  /// How often the provider publishes rates. Determines the unit of
  /// publishes_missed: a count of days, ISO weeks, or calendar months.
  /// Null for historical-only providers with no scheduled cadence.
  final String? publishCadence;

  /// Number of expected publishes missed since end_date, in units of
  /// publish_cadence. For daily providers, counts scheduled publish days
  /// strictly between end_date and today. For weekly and monthly
  /// providers, counts ISO weeks or calendar months between the latest
  /// imported bucket and the bucket whose publish window has already
  /// started. Null when the provider has no scheduled cadence or no
  /// imported data.
  final int? publishesMissed;

  /// Currency codes covered by this provider
  final List<String> currencies;

  /// Builds a `Provider`.
  const Provider({
    required this.key,
    required this.name,
    this.countryCode,
    this.rateType,
    this.pivotCurrency,
    this.dataUrl,
    this.termsUrl,
    this.startDate,
    this.endDate,
    this.publishCadence,
    this.publishesMissed,
    required this.currencies,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'Provider';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['key', 'name', 'country_code', 'rate_type', 'pivot_currency', 'data_url', 'terms_url', 'start_date', 'end_date', 'publish_cadence', 'publishes_missed', 'currencies'];

  /// The values the document allows for `publish_cadence`.
  static const List<String> publishCadenceValues = ['daily', 'weekly', 'monthly'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<Provider, DecodeError> fromJson(
    Object? json, [
    String path = 'Provider',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'Provider', json));
    }
    final key = dmxString(json['key'], '$path.key');
    final name = dmxString(json['name'], '$path.name');
    final countryCode = dmxNullable(json['country_code'], '$path.country_code', dmxString);
    final rateType = dmxNullable(json['rate_type'], '$path.rate_type', dmxString);
    final pivotCurrency = dmxNullable(json['pivot_currency'], '$path.pivot_currency', dmxString);
    final dataUrl = dmxNullable(json['data_url'], '$path.data_url', dmxUri);
    final termsUrl = dmxNullable(json['terms_url'], '$path.terms_url', dmxUri);
    final startDate = dmxNullable(json['start_date'], '$path.start_date', dmxDateTime);
    final endDate = dmxNullable(json['end_date'], '$path.end_date', dmxDateTime);
    final publishCadence = dmxNullable(json['publish_cadence'], '$path.publish_cadence', dmxString);
    final publishesMissed = dmxNullable(json['publishes_missed'], '$path.publishes_missed', dmxInt);
    final currencies = dmxListOf(dmxString)(json['currencies'], '$path.currencies');
    return switch ((
      key,
      name,
      countryCode,
      rateType,
      pivotCurrency,
      dataUrl,
      termsUrl,
      startDate,
      endDate,
      publishCadence,
      publishesMissed,
      currencies,
    )) {
      (
        Ok(value: final key),
        Ok(value: final name),
        Ok(value: final countryCode),
        Ok(value: final rateType),
        Ok(value: final pivotCurrency),
        Ok(value: final dataUrl),
        Ok(value: final termsUrl),
        Ok(value: final startDate),
        Ok(value: final endDate),
        Ok(value: final publishCadence),
        Ok(value: final publishesMissed),
        Ok(value: final currencies),
      ) =>
        Ok(Provider(
          key: key,
          name: name,
          countryCode: countryCode,
          rateType: rateType,
          pivotCurrency: pivotCurrency,
          dataUrl: dataUrl,
          termsUrl: termsUrl,
          startDate: startDate,
          endDate: endDate,
          publishCadence: publishCadence,
          publishesMissed: publishesMissed,
          currencies: currencies,
        )),
      _ => Err(dmxFirstError(
          [key, name, countryCode, rateType, pivotCurrency, dataUrl, termsUrl, startDate, endDate, publishCadence, publishesMissed, currencies],
          path,
          'Provider',
        )),
    };
  }
}
