// dmx: generated from frankfurter.dart — do not edit.

import 'package:dmx/dmx.dart';

/// An object the document declares inline, named after where it was
/// found.
///
/// Generated from `components/schemas/RateProvider` of the vendored OpenAPI
/// document. Change the document and rebuild; never edit this file.
final class RateProvider {
  /// Provider key
  final String key;

  /// Provider observation date used for this entry
  final DateTime date;

  /// Provider's rate, rebased to the row's base
  final double rate;

  /// Present and true when this entry did not contribute to the blended
  /// rate
  final bool? excluded;

  /// Builds a `RateProvider`.
  const RateProvider({
    required this.key,
    required this.date,
    required this.rate,
    this.excluded,
  });

  /// The schema in the document this class was generated from.
  static const String schemaName = 'RateProvider';

  /// Every property the schema declares, in document order.
  static const List<String> propertyNames = ['key', 'date', 'rate', 'excluded'];

  /// One JSON value decoded, or a [DecodeError] naming the first property
  /// that did not match the schema.
  static Result<RateProvider, DecodeError> fromJson(
    Object? json, [
    String path = 'RateProvider',
  ]) {
    if (json is! Map<String, Object?>) {
      return Err(DecodeError(path, 'RateProvider', json));
    }
    final key = dmxString(json['key'], '$path.key');
    final date = dmxDateTime(json['date'], '$path.date');
    final rate = dmxDouble(json['rate'], '$path.rate');
    final excluded = dmxNullable(json['excluded'], '$path.excluded', dmxBool);
    return switch ((
      key,
      date,
      rate,
      excluded,
    )) {
      (
        Ok(value: final key),
        Ok(value: final date),
        Ok(value: final rate),
        Ok(value: final excluded),
      ) =>
        Ok(RateProvider(
          key: key,
          date: date,
          rate: rate,
          excluded: excluded,
        )),
      _ => Err(dmxFirstError(
          [key, date, rate, excluded],
          path,
          'RateProvider',
        )),
    };
  }
}
