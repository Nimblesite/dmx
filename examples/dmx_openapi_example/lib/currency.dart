// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

/// The `Currency` schema.
///
/// Generated from `components/schemas/Currency` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class Currency {
  /// ISO 4217 currency code
  final String isoCode;

  /// ISO 4217 numeric code
  final String? isoNumeric;

  /// Full currency name
  final String name;

  /// Currency symbol
  final String? symbol;

  /// Earliest available date
  final DateTime? startDate;

  /// Latest available date
  final DateTime? endDate;

  /// Builds a `Currency`.
  const Currency({
    required this.isoCode,
    this.isoNumeric,
    required this.name,
    this.symbol,
    this.startDate,
    this.endDate,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'Currency';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['iso_code', 'iso_numeric', 'name', 'symbol', 'start_date', 'end_date'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<Currency, DecodeError> fromJson(
    Object? json, [
    String path = 'Currency',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'Currency', json));
    }
    final isoCode = dmxString(json['iso_code'], '$path.iso_code');
    final isoNumeric = dmxNullable(json['iso_numeric'], '$path.iso_numeric', dmxString);
    final name = dmxString(json['name'], '$path.name');
    final symbol = dmxNullable(json['symbol'], '$path.symbol', dmxString);
    final startDate = dmxNullable(json['start_date'], '$path.start_date', dmxDateTime);
    final endDate = dmxNullable(json['end_date'], '$path.end_date', dmxDateTime);
    return switch ((
      isoCode,
      isoNumeric,
      name,
      symbol,
      startDate,
      endDate,
    )) {
      (
        Ok(value: final isoCode),
        Ok(value: final isoNumeric),
        Ok(value: final name),
        Ok(value: final symbol),
        Ok(value: final startDate),
        Ok(value: final endDate),
      ) =>
        Ok(Currency(
          isoCode: isoCode,
          isoNumeric: isoNumeric,
          name: name,
          symbol: symbol,
          startDate: startDate,
          endDate: endDate,
        )),
      _ => Err(dmxFirstError(
          [isoCode, isoNumeric, name, symbol, startDate, endDate],
          path,
          'Currency',
        )),
    };
  }
}
