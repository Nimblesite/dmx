// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

/// Peg metadata, present only for pegged currencies
///
/// Generated from `components/schemas/CurrencyDetailPeg` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class CurrencyDetailPeg {
  /// `base` from the document.
  final String? base;

  /// `rate` from the document.
  final double? rate;

  /// `authority` from the document.
  final String? authority;

  /// `source` from the document.
  final Uri? source;

  /// Builds a `CurrencyDetailPeg`.
  const CurrencyDetailPeg({
    this.base,
    this.rate,
    this.authority,
    this.source,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'CurrencyDetailPeg';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['base', 'rate', 'authority', 'source'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<CurrencyDetailPeg, DecodeError> fromJson(
    Object? json, [
    String path = 'CurrencyDetailPeg',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'CurrencyDetailPeg', json));
    }
    final base = dmxNullable(json['base'], '$path.base', dmxString);
    final rate = dmxNullable(json['rate'], '$path.rate', dmxDouble);
    final authority = dmxNullable(json['authority'], '$path.authority', dmxString);
    final source = dmxNullable(json['source'], '$path.source', dmxUri);
    return switch ((
      base,
      rate,
      authority,
      source,
    )) {
      (
        Ok(value: final base),
        Ok(value: final rate),
        Ok(value: final authority),
        Ok(value: final source),
      ) =>
        Ok(CurrencyDetailPeg(
          base: base,
          rate: rate,
          authority: authority,
          source: source,
        )),
      _ => Err(dmxFirstError(
          [base, rate, authority, source],
          path,
          'CurrencyDetailPeg',
        )),
    };
  }
}
