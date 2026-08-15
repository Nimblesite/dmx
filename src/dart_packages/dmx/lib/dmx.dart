/// The `@dmx` trigger and runtime for `dmx` [surface.annotations].
///
/// One annotation is the whole authoring surface: `@dmx('macro')` names what
/// to generate and its map carries the macro's arguments as plain data. The
/// binary reads both out of the CST at generation time — nothing here runs,
/// and there is no catalogue of annotation classes to keep in sync. The
/// runtime is what generated code composes with: a `Result` type
/// [model.json-codec], collection combinators, and the `copyWith` option
/// type [model.copywith]. Nothing here throws; nothing here casts.
library;

export 'src/decode.dart';
export 'src/support.dart';
export 'src/transport.dart';
export 'src/version.dart';

/// The single generation trigger [surface.annotations].
///
/// `macro` names what is generated — `@dmx('model')`, `@dmx('union')`,
/// `@dmx('key')` — and `args` carries that macro's configuration as data:
///
/// ```dart
/// @dmx('union', {'discriminator': 'kind'})
/// sealed class Shape { ... }
///
/// @dmx('model', {'fieldRename': 'snake'})
/// class Order {
///   @dmx('key', {'name': 'order_id'})
///   final String id;
///   ...
/// }
/// ```
///
/// The type carries no behaviour whatsoever: the `dmx` binary reads the macro
/// name and the map out of the CST, and hands the declaration's own structure
/// — fields, types, variants, members — to the macro. Deliberately lowercase,
/// the way `@override` and `@pragma` are: it is an instruction to a tool, not
/// a type a program uses.
class dmx {
  /// Which macro this declaration or member opts into.
  final String macro;

  /// The macro's arguments, as plain data.
  final Map<String, Object?> args;

  const dmx(this.macro, [this.args = const {}]);
}

// ---------------------------------------------------------------------------
// Result [model.json-codec]
// ---------------------------------------------------------------------------

/// The outcome of a fallible operation: [Ok] carrying a `T`, or [Err] carrying
/// an `E`. Never an exception. Sealed, so a `switch` over it is exhaustive at
/// compile time.
///
/// The failure is a type parameter, not a fixed shape: decoding uses
/// `Result<T, DecodeError>`, and anything else you compose with generated code
/// picks its own `E`.
sealed class Result<T, E> {
  const Result();
}

/// Success, carrying its [value].
final class Ok<T, E> extends Result<T, E> {
  const Ok(this.value);

  final T value;

  /// A `Result` is a value, so two successes carrying equal values are equal.
  /// Without this, every test that compares one has to unwrap it first, and
  /// the type stops behaving like the data it is.
  @override
  bool operator ==(Object other) =>
      identical(this, other) || (other is Ok<T, E> && other.value == value);

  @override
  int get hashCode => Object.hash(Ok, value);

  @override
  String toString() => 'Ok($value)';
}

/// Failure, carrying its [error].
final class Err<T, E> extends Result<T, E> {
  const Err(this.error);

  final E error;

  @override
  bool operator ==(Object other) =>
      identical(this, other) || (other is Err<T, E> && other.error == error);

  @override
  int get hashCode => Object.hash(Err, error);

  @override
  String toString() => 'Err($error)';
}

/// Why a decode failed, and where.
///
/// [path] is a dotted field path such as `User.address.street`, with `[i]` for
/// list indices and `[key]` for map keys, so a failure deep inside a nested
/// structure still names its exact location. The error travels unchanged as an
/// enclosing decode re-wraps it — it describes where the failure *happened*,
/// not where it was observed.
final class DecodeError {
  const DecodeError(this.path, this.expected, this.actual);

  /// Where the failure happened, e.g. `User.tags[2]`.
  final String path;

  /// The Dart type that was required, e.g. `String`.
  final String expected;

  /// The offending JSON value.
  final Object? actual;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is DecodeError &&
          other.path == path &&
          other.expected == expected &&
          other.actual == actual);

  @override
  int get hashCode => Object.hash(runtimeType, path, expected, actual);

  @override
  String toString() => '$path: expected $expected, got ${_brief(actual)}';
}

String _brief(Object? value) => switch (value) {
      null => 'null',
      final String v => '"$v"',
      _ => '$value (${value.runtimeType})',
    };

/// Decodes one JSON value at [path]. The unit generated code composes.
typedef DmxDecode<T> = Result<T, DecodeError> Function(
    Object? value, String path);

/// Decodes a JSON list, failing at the first bad element [model.json-codec].
Result<List<T>, DecodeError> dmxList<T>(
  List<dynamic> source,
  String path,
  DmxDecode<T> item,
) {
  final out = <T>[];
  for (var i = 0; i < source.length; i++) {
    switch (item(source[i], '$path[$i]')) {
      case Ok(value: final v):
        out.add(v);
      case Err(error: final e):
        return Err(e);
    }
  }
  return Ok(List<T>.unmodifiable(out));
}

/// Decodes a JSON list into a `Set` [model.json-codec].
Result<Set<T>, DecodeError> dmxSet<T>(
  List<dynamic> source,
  String path,
  DmxDecode<T> item,
) =>
    switch (dmxList(source, path, item)) {
      Ok(value: final v) => Ok(Set<T>.unmodifiable(v)),
      Err(error: final e) => Err(e),
    };

/// Decodes a JSON object into a `Map`, failing at the first bad value [model.json-codec].
Result<Map<String, V>, DecodeError> dmxMap<V>(
  Map<String, dynamic> source,
  String path,
  DmxDecode<V> value,
) {
  final out = <String, V>{};
  for (final entry in source.entries) {
    switch (value(entry.value, '$path[${entry.key}]')) {
      case Ok(value: final v):
        out[entry.key] = v;
      case Err(error: final e):
        return Err(e);
    }
  }
  return Ok(Map<String, V>.unmodifiable(out));
}

/// Lifts a decoder over a nullable field: a missing or null value is [Ok] with
/// `null`, anything else defers to [decode] [model.json-codec].
Result<T?, DecodeError> dmxNullable<T>(
  Object? value,
  String path,
  DmxDecode<T> decode,
) =>
    value == null
        ? Ok<T?, DecodeError>(null)
        : switch (decode(value, path)) {
            Ok(value: final v) => Ok<T?, DecodeError>(v),
            Err(error: final e) => Err<T?, DecodeError>(e),
          };

// ---------------------------------------------------------------------------
// copyWith option type [model.copywith]
// ---------------------------------------------------------------------------

/// Distinguishes "leave this field alone" from "set it to null" in `copyWith`,
/// without a sentinel `Object?` parameter that would erase the field's type.
///
/// `copyWith()` keeps; `copyWith(email: DmxTo(null))` clears; and
/// `copyWith(email: DmxTo(42))` does not compile.
sealed class DmxPatch<T> {
  const DmxPatch();
}

/// Leave the field unchanged. The default for every nullable `copyWith` param.
final class DmxKeep<T> extends DmxPatch<T> {
  const DmxKeep();
}

/// Replace the field with [value], which may be null.
final class DmxTo<T> extends DmxPatch<T> {
  const DmxTo(this.value);

  final T value;
}

// ---------------------------------------------------------------------------
// Structural equality [model.equality]
// ---------------------------------------------------------------------------

/// Structural equality for collections [model.equality]. Referenced by generated `==`
/// implementations for `List`/`Set`/`Map`/`Iterable` fields.
bool dmxDeepEquals(Object? a, Object? b) => switch ((a, b)) {
      _ when identical(a, b) => true,
      (final List<Object?> x, final List<Object?> y) => x.length == y.length &&
          Iterable<int>.generate(
            x.length,
          ).every((i) => dmxDeepEquals(x[i], y[i])),
      (final Set<Object?> x, final Set<Object?> y) => x.length == y.length &&
          x.every((e) => y.any((o) => dmxDeepEquals(e, o))),
      (final Map<Object?, Object?> x, final Map<Object?, Object?> y) =>
        x.length == y.length &&
            x.keys.every((k) => y.containsKey(k) && dmxDeepEquals(x[k], y[k])),
      _ => a == b,
    };

/// Hash consistent with [dmxDeepEquals]: equal collections hash equally [model.equality].
int dmxDeepHash(Object? o) => switch (o) {
      final List<Object?> v => Object.hashAll(v.map(dmxDeepHash)),
      final Set<Object?> v => Object.hashAllUnordered(v.map(dmxDeepHash)),
      final Map<Object?, Object?> v => Object.hashAllUnordered(
          v.entries.map(
              (e) => Object.hash(dmxDeepHash(e.key), dmxDeepHash(e.value))),
        ),
      _ => o.hashCode,
    };
