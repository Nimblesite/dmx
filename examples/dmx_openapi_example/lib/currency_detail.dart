// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

import 'currency_detail_peg.dart';

/// The `CurrencyDetail` schema.
///
/// Generated from `components/schemas/CurrencyDetail` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class CurrencyDetail {
  /// ISO 4217 currency code
  final String isoCode;

  /// ISO 4217 numeric code
  final String? isoNumeric;

  /// Full currency name
  final String name;

  /// Currency symbol
  final String? symbol;

  /// Provider keys that publish this currency
  final List<String>? providers;

  /// Peg metadata, present only for pegged currencies
  final CurrencyDetailPeg? peg;

  /// Builds a `CurrencyDetail`.
  const CurrencyDetail({
    required this.isoCode,
    this.isoNumeric,
    required this.name,
    this.symbol,
    this.providers,
    this.peg,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'CurrencyDetail';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['iso_code', 'iso_numeric', 'name', 'symbol', 'providers', 'peg'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<CurrencyDetail, DecodeError> fromJson(
    Object? json, [
    String path = 'CurrencyDetail',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'CurrencyDetail', json));
    }
    final isoCode = dmxString(json['iso_code'], '$path.iso_code');
    final isoNumeric = dmxNullable(json['iso_numeric'], '$path.iso_numeric', dmxString);
    final name = dmxString(json['name'], '$path.name');
    final symbol = dmxNullable(json['symbol'], '$path.symbol', dmxString);
    final providers = dmxNullable(json['providers'], '$path.providers', dmxListOf(dmxString));
    final peg = dmxNullable(json['peg'], '$path.peg', CurrencyDetailPeg.fromJson);
    return switch ((
      isoCode,
      isoNumeric,
      name,
      symbol,
      providers,
      peg,
    )) {
      (
        Ok(value: final isoCode),
        Ok(value: final isoNumeric),
        Ok(value: final name),
        Ok(value: final symbol),
        Ok(value: final providers),
        Ok(value: final peg),
      ) =>
        Ok(CurrencyDetail(
          isoCode: isoCode,
          isoNumeric: isoNumeric,
          name: name,
          symbol: symbol,
          providers: providers,
          peg: peg,
        )),
      _ => Err(dmxFirstError(
          [isoCode, isoNumeric, name, symbol, providers, peg],
          path,
          'CurrencyDetail',
        )),
    };
  }
}
