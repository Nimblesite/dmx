/// Leaf decoders [model.json-codec].
///
/// Every one of these is a `DmxDecode<T>` — `(Object? value, String path)` in,
/// `Result<T, DecodeError>` out. That single shape is what lets a generated
/// `Foo.fromJson` tear-off drop into `dmxList`, `dmxMap`, or `dmxNullable`
/// without a wrapper, and it is why the generator never emits a cast.
library;

import '../dmx.dart';

/// Reads one key out of a value that may not even be a map.
///
/// Generated code needs this for nullable fields: an absent key and a present
/// null are the same value, and neither may take down the decode.
Object? dmxKey(Object? json, String key) => switch (json) {
      final Map<String, dynamic> map => map[key],
      _ => null,
    };

Result<String, DecodeError> dmxString(Object? value, String path) =>
    switch (value) {
      final String value => Ok(value),
      _ => Err(DecodeError(path, 'String', value)),
    };

Result<int, DecodeError> dmxInt(Object? value, String path) => switch (value) {
      final int value => Ok(value),
      _ => Err(DecodeError(path, 'int', value)),
    };

/// Accepts an `int` as well as a `double`: JSON has one number type, and a
/// price of exactly 5 arrives as `5`, not `5.0`.
Result<double, DecodeError> dmxDouble(Object? value, String path) =>
    switch (value) {
      final double value => Ok(value),
      final int value => Ok(value.toDouble()),
      _ => Err(DecodeError(path, 'double', value)),
    };

Result<num, DecodeError> dmxNum(Object? value, String path) => switch (value) {
      final num value => Ok(value),
      _ => Err(DecodeError(path, 'num', value)),
    };

Result<bool, DecodeError> dmxBool(Object? value, String path) =>
    switch (value) {
      final bool value => Ok(value),
      _ => Err(DecodeError(path, 'bool', value)),
    };

/// ISO-8601 in, `DateTime` out. `DateTime.parse` throws; `tryParse` returns
/// null, and null is a decode failure like any other.
Result<DateTime, DecodeError> dmxDateTime(Object? value, String path) =>
    switch (value) {
      final String value => switch (DateTime.tryParse(value)) {
          final DateTime parsed => Ok(parsed),
          null => Err(DecodeError(path, 'DateTime', value)),
        },
      _ => Err(DecodeError(path, 'DateTime', value)),
    };

Result<Uri, DecodeError> dmxUri(Object? value, String path) => switch (value) {
      final String value => switch (Uri.tryParse(value)) {
          final Uri parsed => Ok(parsed),
          null => Err(DecodeError(path, 'Uri', value)),
        },
      _ => Err(DecodeError(path, 'Uri', value)),
    };

/// A duration on the wire is whole milliseconds; anything else is a lie about
/// precision somebody will eventually depend on.
Result<Duration, DecodeError> dmxDuration(Object? value, String path) =>
    switch (value) {
      final int value => Ok(Duration(milliseconds: value)),
      _ => Err(DecodeError(path, 'Duration', value)),
    };

/// Passes any JSON value through untouched, for the one field in a payload
/// that genuinely has no schema.
Result<Object?, DecodeError> dmxAny(Object? value, String path) => Ok(value);

/// [dmxList] as a `DmxDecode`, so a list is composable like a leaf
/// [model.json-codec].
///
/// `dmxList` takes a `List<dynamic>` because it has already been narrowed.
/// This narrows, which is what a generator needs to drop a list straight into
/// `dmxNullable` or another `dmxListOf` without emitting the `is List` test —
/// and an unnarrowed value at that position is a decode failure, not a crash.
DmxDecode<List<T>> dmxListOf<T>(DmxDecode<T> item) =>
    (value, path) => switch (value) {
          final List<dynamic> source => dmxList(source, path, item),
          _ => Err(DecodeError(path, 'List', value)),
        };

/// [dmxMap] as a `DmxDecode`, narrowing first [model.json-codec].
DmxDecode<Map<String, V>> dmxMapOf<V>(DmxDecode<V> value) =>
    (source, path) => switch (source) {
          final Map<String, dynamic> map => dmxMap(map, path, value),
          _ => Err(DecodeError(path, 'Map', source)),
        };

/// The first failure among `results` [model.json-codec].
///
/// Generated code decodes every property, then reports one error. Asking each
/// result in order means the error names the property a reader would look at
/// first — the earliest one in the schema that did not match.
///
/// The fallback is unreachable from generated code, which only asks for a
/// first error once it knows at least one result failed. It is here so the
/// function is total without a throw.
DecodeError dmxFirstError(
  List<Result<Object?, DecodeError>> results,
  String path,
  String expected,
) {
  for (final result in results) {
    if (result case Err(:final error)) {
      return error;
    }
  }
  return DecodeError(path, expected, null);
}
