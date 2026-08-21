// dmx: generated from models/targeting.td — do not edit.
// dmx: rendered through the canonical model template, definition 6070c6f7e26a9e98, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/targeting.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// Only dart and rust — an immutable value from the diagram.
final class OnlyDartAndRust {
  /// Every field, in the order the diagram declares them.
  const OnlyDartAndRust({required this.a});

  /// The `a` field, declared as `Int`.
  final int a;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is OnlyDartAndRust &&
          other.a == a);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        a,
      );

  @override
  String toString() => 'OnlyDartAndRust(a: $a)';

  /// A copy of this value with the named fields replaced.
  OnlyDartAndRust copyWith({
    int? a,
  }) =>
      OnlyDartAndRust(
        a: a ?? this.a,
      );
}

/// JSON for [OnlyDartAndRust].
extension OnlyDartAndRustJson on OnlyDartAndRust {
  /// Decodes a `OnlyDartAndRust` from a JSON value, or says why it could not.
  static dmx.Result<OnlyDartAndRust, dmx.DecodeError> fromJson(Object? json, [String path = 'OnlyDartAndRust']) =>
      switch (json) {
        {
          'a': final int a,
        } =>
          dmx.Ok(OnlyDartAndRust(
            a: a,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'OnlyDartAndRust', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'a': a,
      };
}

/// Not go — an immutable value from the diagram.
final class NotGo {
  /// Every field, in the order the diagram declares them.
  const NotGo({required this.b});

  /// The `b` field, declared as `String`.
  final String b;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is NotGo &&
          other.b == b);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        b,
      );

  @override
  String toString() => 'NotGo(b: $b)';

  /// A copy of this value with the named fields replaced.
  NotGo copyWith({
    String? b,
  }) =>
      NotGo(
        b: b ?? this.b,
      );
}

/// JSON for [NotGo].
extension NotGoJson on NotGo {
  /// Decodes a `NotGo` from a JSON value, or says why it could not.
  static dmx.Result<NotGo, dmx.DecodeError> fromJson(Object? json, [String path = 'NotGo']) =>
      switch (json) {
        {
          'b': final String b,
        } =>
          dmx.Ok(NotGo(
            b: b,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'NotGo', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'b': b,
      };
}

/// Both — exactly one of the cases below.
sealed class Both {
  /// The shared constructor every case delegates to.
  const Both();
}

/// JSON for [Both].
extension BothJson on Both {
  /// Decodes whichever case the payload's 'type' names.
  static dmx.Result<Both, dmx.DecodeError> fromJson(Object? json, [String path = 'Both']) =>
      switch (json) {
        {
          'type': final String type,
        } =>
          switch (type) {
            'one' => OneJson.fromJson(json, path),
            'two' => TwoJson.fromJson(json, path),
            _ => dmx.Err(dmx.DecodeError(path, 'Both', json)),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Both', json)),
      };

  /// This value as a JSON map, tagged with the case it is.
  Map<String, Object?> toJson() => switch (this) {
        final One value => <String, Object?>{
            'type': 'one',
            ...value.toJson(),
          },
        final Two value => <String, Object?>{
            'type': 'two',
            ...value.toJson(),
          },
      };
}

/// The `One` case of Both, as an immutable value.
final class One extends Both {
  /// Every field, in the order the diagram declares them.
  const One() : super();

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is One);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'One()';
}

/// JSON for [One].
extension OneJson on One {
  /// Decodes a `One` from a JSON value, or says why it could not.
  static dmx.Result<One, dmx.DecodeError> fromJson(Object? json, [String path = 'One']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(One(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'One', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// The `Two` case of Both, as an immutable value.
final class Two extends Both {
  /// Every field, in the order the diagram declares them.
  const Two({required this.x}) : super();

  /// The `x` field, declared as `Int`.
  final int x;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Two &&
          other.x == x);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        x,
      );

  @override
  String toString() => 'Two(x: $x)';

  /// A copy of this value with the named fields replaced.
  Two copyWith({
    int? x,
  }) =>
      Two(
        x: x ?? this.x,
      );
}

/// JSON for [Two].
extension TwoJson on Two {
  /// Decodes a `Two` from a JSON value, or says why it could not.
  static dmx.Result<Two, dmx.DecodeError> fromJson(Object? json, [String path = 'Two']) =>
      switch (json) {
        {
          'x': final int x,
        } =>
          dmx.Ok(Two(
            x: x,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Two', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'x': x,
      };
}

/// `Plain` as the diagram declares it.
typedef Plain = int;
