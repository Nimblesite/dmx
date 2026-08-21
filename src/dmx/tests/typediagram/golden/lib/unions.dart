// dmx: generated from models/unions.td — do not edit.
// dmx: rendered through the canonical model template, definition 5214cc1d7a2b8b4d, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/unions.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// Shape — exactly one of the cases below.
sealed class Shape {
  /// The shared constructor every case delegates to.
  const Shape();
}

/// JSON for [Shape].
extension ShapeJson on Shape {
  /// Decodes whichever case the payload's 'type' names.
  static dmx.Result<Shape, dmx.DecodeError> fromJson(Object? json, [String path = 'Shape']) =>
      switch (json) {
        {
          'type': final String type,
        } =>
          switch (type) {
            'circle' => CircleJson.fromJson(json, path),
            'rectangle' => RectangleJson.fromJson(json, path),
            'triangle' => TriangleJson.fromJson(json, path),
            'point' => PointJson.fromJson(json, path),
            _ => dmx.Err(dmx.DecodeError(path, 'Shape', json)),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Shape', json)),
      };

  /// This value as a JSON map, tagged with the case it is.
  Map<String, Object?> toJson() => switch (this) {
        final Circle value => <String, Object?>{
            'type': 'circle',
            ...value.toJson(),
          },
        final Rectangle value => <String, Object?>{
            'type': 'rectangle',
            ...value.toJson(),
          },
        final Triangle value => <String, Object?>{
            'type': 'triangle',
            ...value.toJson(),
          },
        final Point value => <String, Object?>{
            'type': 'point',
            ...value.toJson(),
          },
      };
}

/// The `Circle` case of Shape, as an immutable value.
final class Circle extends Shape {
  /// Every field, in the order the diagram declares them.
  const Circle({required this.radius}) : super();

  /// The `radius` field, declared as `Float`.
  final double radius;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Circle &&
          other.radius == radius);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        radius,
      );

  @override
  String toString() => 'Circle(radius: $radius)';

  /// A copy of this value with the named fields replaced.
  Circle copyWith({
    double? radius,
  }) =>
      Circle(
        radius: radius ?? this.radius,
      );
}

/// JSON for [Circle].
extension CircleJson on Circle {
  /// Decodes a `Circle` from a JSON value, or says why it could not.
  static dmx.Result<Circle, dmx.DecodeError> fromJson(Object? json, [String path = 'Circle']) =>
      switch (json) {
        {
          'radius': final num radius,
        } =>
          dmx.Ok(Circle(
            radius: radius.toDouble(),
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Circle', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'radius': radius,
      };
}

/// The `Rectangle` case of Shape, as an immutable value.
final class Rectangle extends Shape {
  /// Every field, in the order the diagram declares them.
  const Rectangle({required this.width, required this.height}) : super();

  /// The `width` field, declared as `Float`.
  final double width;

  /// The `height` field, declared as `Float`.
  final double height;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Rectangle &&
          other.width == width &&
          other.height == height);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        width,
        height,
      );

  @override
  String toString() => 'Rectangle(width: $width, height: $height)';

  /// A copy of this value with the named fields replaced.
  Rectangle copyWith({
    double? width,
    double? height,
  }) =>
      Rectangle(
        width: width ?? this.width,
        height: height ?? this.height,
      );
}

/// JSON for [Rectangle].
extension RectangleJson on Rectangle {
  /// Decodes a `Rectangle` from a JSON value, or says why it could not.
  static dmx.Result<Rectangle, dmx.DecodeError> fromJson(Object? json, [String path = 'Rectangle']) =>
      switch (json) {
        {
          'width': final num width,
          'height': final num height,
        } =>
          dmx.Ok(Rectangle(
            width: width.toDouble(),
            height: height.toDouble(),
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Rectangle', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'width': width,
        'height': height,
      };
}

/// The `Triangle` case of Shape, as an immutable value.
final class Triangle extends Shape {
  /// Every field, in the order the diagram declares them.
  const Triangle({required this.a, required this.b, required this.c}) : super();

  /// The `a` field, declared as `Float`.
  final double a;

  /// The `b` field, declared as `Float`.
  final double b;

  /// The `c` field, declared as `Float`.
  final double c;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Triangle &&
          other.a == a &&
          other.b == b &&
          other.c == c);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        a,
        b,
        c,
      );

  @override
  String toString() => 'Triangle(a: $a, b: $b, c: $c)';

  /// A copy of this value with the named fields replaced.
  Triangle copyWith({
    double? a,
    double? b,
    double? c,
  }) =>
      Triangle(
        a: a ?? this.a,
        b: b ?? this.b,
        c: c ?? this.c,
      );
}

/// JSON for [Triangle].
extension TriangleJson on Triangle {
  /// Decodes a `Triangle` from a JSON value, or says why it could not.
  static dmx.Result<Triangle, dmx.DecodeError> fromJson(Object? json, [String path = 'Triangle']) =>
      switch (json) {
        {
          'a': final num a,
          'b': final num b,
          'c': final num c,
        } =>
          dmx.Ok(Triangle(
            a: a.toDouble(),
            b: b.toDouble(),
            c: c.toDouble(),
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Triangle', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'a': a,
        'b': b,
        'c': c,
      };
}

/// The `Point` case of Shape, as an immutable value.
final class Point extends Shape {
  /// Every field, in the order the diagram declares them.
  const Point() : super();

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Point);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'Point()';
}

/// JSON for [Point].
extension PointJson on Point {
  /// Decodes a `Point` from a JSON value, or says why it could not.
  static dmx.Result<Point, dmx.DecodeError> fromJson(Object? json, [String path = 'Point']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(Point(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Point', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// Error code — exactly one of the cases below.
sealed class ErrorCode {
  /// The shared constructor every case delegates to.
  const ErrorCode();
}

/// JSON for [ErrorCode].
extension ErrorCodeJson on ErrorCode {
  /// Decodes whichever case the payload's 'type' names.
  static dmx.Result<ErrorCode, dmx.DecodeError> fromJson(Object? json, [String path = 'ErrorCode']) =>
      switch (json) {
        {
          'type': final String type,
        } =>
          switch (type) {
            'parseError' => ParseErrorJson.fromJson(json, path),
            'invalidRequest' => InvalidRequestJson.fromJson(json, path),
            'methodNotFound' => MethodNotFoundJson.fromJson(json, path),
            'ok' => ErrorCodeOkJson.fromJson(json, path),
            'grouped' => GroupedJson.fromJson(json, path),
            _ => dmx.Err(dmx.DecodeError(path, 'ErrorCode', json)),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'ErrorCode', json)),
      };

  /// This value as a JSON map, tagged with the case it is.
  Map<String, Object?> toJson() => switch (this) {
        final ParseError value => <String, Object?>{
            'type': 'parseError',
            ...value.toJson(),
          },
        final InvalidRequest value => <String, Object?>{
            'type': 'invalidRequest',
            ...value.toJson(),
          },
        final MethodNotFound value => <String, Object?>{
            'type': 'methodNotFound',
            ...value.toJson(),
          },
        final ErrorCodeOk value => <String, Object?>{
            'type': 'ok',
            ...value.toJson(),
          },
        final Grouped value => <String, Object?>{
            'type': 'grouped',
            ...value.toJson(),
          },
      };
}

/// The `ParseError` case of ErrorCode, as an immutable value.
final class ParseError extends ErrorCode {
  /// Every field, in the order the diagram declares them.
  const ParseError() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32700;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ParseError);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'ParseError()';
}

/// JSON for [ParseError].
extension ParseErrorJson on ParseError {
  /// Decodes a `ParseError` from a JSON value, or says why it could not.
  static dmx.Result<ParseError, dmx.DecodeError> fromJson(Object? json, [String path = 'ParseError']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(ParseError(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'ParseError', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// The `InvalidRequest` case of ErrorCode, as an immutable value.
final class InvalidRequest extends ErrorCode {
  /// Every field, in the order the diagram declares them.
  const InvalidRequest() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32600;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is InvalidRequest);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'InvalidRequest()';
}

/// JSON for [InvalidRequest].
extension InvalidRequestJson on InvalidRequest {
  /// Decodes a `InvalidRequest` from a JSON value, or says why it could not.
  static dmx.Result<InvalidRequest, dmx.DecodeError> fromJson(Object? json, [String path = 'InvalidRequest']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(InvalidRequest(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'InvalidRequest', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// The `MethodNotFound` case of ErrorCode, as an immutable value.
final class MethodNotFound extends ErrorCode {
  /// Every field, in the order the diagram declares them.
  const MethodNotFound() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32601;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is MethodNotFound);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'MethodNotFound()';
}

/// JSON for [MethodNotFound].
extension MethodNotFoundJson on MethodNotFound {
  /// Decodes a `MethodNotFound` from a JSON value, or says why it could not.
  static dmx.Result<MethodNotFound, dmx.DecodeError> fromJson(Object? json, [String path = 'MethodNotFound']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(MethodNotFound(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'MethodNotFound', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// The `Ok` case of ErrorCode, as an immutable value.
final class ErrorCodeOk extends ErrorCode {
  /// Every field, in the order the diagram declares them.
  const ErrorCodeOk() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = 0;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ErrorCodeOk);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'ErrorCodeOk()';
}

/// JSON for [ErrorCodeOk].
extension ErrorCodeOkJson on ErrorCodeOk {
  /// Decodes a `ErrorCodeOk` from a JSON value, or says why it could not.
  static dmx.Result<ErrorCodeOk, dmx.DecodeError> fromJson(Object? json, [String path = 'ErrorCodeOk']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(ErrorCodeOk(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'ErrorCodeOk', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// The `Grouped` case of ErrorCode, as an immutable value.
final class Grouped extends ErrorCode {
  /// Every field, in the order the diagram declares them.
  const Grouped() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = 1_000;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Grouped);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'Grouped()';
}

/// JSON for [Grouped].
extension GroupedJson on Grouped {
  /// Decodes a `Grouped` from a JSON value, or says why it could not.
  static dmx.Result<Grouped, dmx.DecodeError> fromJson(Object? json, [String path = 'Grouped']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(Grouped(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Grouped', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// Option — exactly one of the cases below.
sealed class Option<T> {
  /// The shared constructor every case delegates to.
  const Option();
}

/// The `Some` case of Option, as an immutable value.
final class Some<T> extends Option<T> {
  /// Every field, in the order the diagram declares them.
  const Some({required this.value}) : super();

  /// The `value` field, declared as `T`.
  final T value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Some<T> &&
          other.value == value);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value,
      );

  @override
  String toString() => 'Some(value: $value)';

  /// A copy of this value with the named fields replaced.
  Some<T> copyWith({
    T? value,
  }) =>
      Some(
        value: value ?? this.value,
      );
}

/// The `None` case of Option, as an immutable value.
final class None<T> extends Option<T> {
  /// Every field, in the order the diagram declares them.
  const None() : super();

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is None<T>);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'None()';
}

/// Result — exactly one of the cases below.
sealed class Result<T, E> {
  /// The shared constructor every case delegates to.
  const Result();
}

/// The `Ok` case of Result, as an immutable value.
final class ResultOk<T, E> extends Result<T, E> {
  /// Every field, in the order the diagram declares them.
  const ResultOk({required this.value}) : super();

  /// The `value` field, declared as `T`.
  final T value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ResultOk<T, E> &&
          other.value == value);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value,
      );

  @override
  String toString() => 'ResultOk(value: $value)';

  /// A copy of this value with the named fields replaced.
  ResultOk<T, E> copyWith({
    T? value,
  }) =>
      ResultOk(
        value: value ?? this.value,
      );
}

/// The `Err` case of Result, as an immutable value.
final class Err<T, E> extends Result<T, E> {
  /// Every field, in the order the diagram declares them.
  const Err({required this.error}) : super();

  /// The `error` field, declared as `E`.
  final E error;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Err<T, E> &&
          other.error == error);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        error,
      );

  @override
  String toString() => 'Err(error: $error)';

  /// A copy of this value with the named fields replaced.
  Err<T, E> copyWith({
    E? error,
  }) =>
      Err(
        error: error ?? this.error,
      );
}

/// Request id — exactly one of the cases below.
sealed class RequestId {
  /// The shared constructor every case delegates to.
  const RequestId();
}

/// JSON for [RequestId].
extension RequestIdJson on RequestId {
  /// Decodes whichever case the payload's 'type' names.
  static dmx.Result<RequestId, dmx.DecodeError> fromJson(Object? json, [String path = 'RequestId']) =>
      switch (json) {
        {
          'type': final String type,
        } =>
          switch (type) {
            'number' => NumberJson.fromJson(json, path),
            'string' => RequestIdStringJson.fromJson(json, path),
            'triple' => TripleJson.fromJson(json, path),
            _ => dmx.Err(dmx.DecodeError(path, 'RequestId', json)),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'RequestId', json)),
      };

  /// This value as a JSON map, tagged with the case it is.
  Map<String, Object?> toJson() => switch (this) {
        final Number value => <String, Object?>{
            'type': 'number',
            ...value.toJson(),
          },
        final RequestIdString value => <String, Object?>{
            'type': 'string',
            ...value.toJson(),
          },
        final Triple value => <String, Object?>{
            'type': 'triple',
            ...value.toJson(),
          },
      };
}

/// The `Number` case of RequestId, as an immutable value.
final class Number extends RequestId {
  /// Every field, in the order the diagram declares them.
  const Number({required this.value1}) : super();

  /// The `value1` field, declared as `Int`.
  final int value1;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Number &&
          other.value1 == value1);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value1,
      );

  @override
  String toString() => 'Number(value1: $value1)';

  /// A copy of this value with the named fields replaced.
  Number copyWith({
    int? value1,
  }) =>
      Number(
        value1: value1 ?? this.value1,
      );
}

/// JSON for [Number].
extension NumberJson on Number {
  /// Decodes a `Number` from a JSON value, or says why it could not.
  static dmx.Result<Number, dmx.DecodeError> fromJson(Object? json, [String path = 'Number']) =>
      switch (json) {
        {
          'value1': final int value1,
        } =>
          dmx.Ok(Number(
            value1: value1,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Number', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'value1': value1,
      };
}

/// The `String` case of RequestId, as an immutable value.
final class RequestIdString extends RequestId {
  /// Every field, in the order the diagram declares them.
  const RequestIdString({required this.value1}) : super();

  /// The `value1` field, declared as `String`.
  final String value1;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is RequestIdString &&
          other.value1 == value1);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value1,
      );

  @override
  String toString() => 'RequestIdString(value1: $value1)';

  /// A copy of this value with the named fields replaced.
  RequestIdString copyWith({
    String? value1,
  }) =>
      RequestIdString(
        value1: value1 ?? this.value1,
      );
}

/// JSON for [RequestIdString].
extension RequestIdStringJson on RequestIdString {
  /// Decodes a `RequestIdString` from a JSON value, or says why it could not.
  static dmx.Result<RequestIdString, dmx.DecodeError> fromJson(Object? json, [String path = 'RequestIdString']) =>
      switch (json) {
        {
          'value1': final String value1,
        } =>
          dmx.Ok(RequestIdString(
            value1: value1,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'RequestIdString', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'value1': value1,
      };
}

/// The `Triple` case of RequestId, as an immutable value.
final class Triple extends RequestId {
  /// Every field, in the order the diagram declares them.
  const Triple({required this.value1, required this.value2, required this.value3}) : super();

  /// The `value1` field, declared as `Int`.
  final int value1;

  /// The `value2` field, declared as `String`.
  final String value2;

  /// The `value3` field, declared as `List<Bool>`.
  final List<bool> value3;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Triple &&
          other.value1 == value1 &&
          other.value2 == value2 &&
          dmx.dmxDeepEquals(other.value3, value3));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value1,
        value2,
        dmx.dmxDeepHash(value3),
      );

  @override
  String toString() => 'Triple(value1: $value1, value2: $value2, value3: $value3)';

  /// A copy of this value with the named fields replaced.
  Triple copyWith({
    int? value1,
    String? value2,
    List<bool>? value3,
  }) =>
      Triple(
        value1: value1 ?? this.value1,
        value2: value2 ?? this.value2,
        value3: value3 ?? this.value3,
      );
}

/// JSON for [Triple].
extension TripleJson on Triple {
  /// Decodes a `Triple` from a JSON value, or says why it could not.
  static dmx.Result<Triple, dmx.DecodeError> fromJson(Object? json, [String path = 'Triple']) =>
      switch (json) {
        {
          'value1': final int value1,
          'value2': final String value2,
          'value3': final List<dynamic> value3,
        } =>
          switch ((
            dmx.dmxList<bool>(value3, '$path.value3', (value, path) => switch (value) {
              final bool value => dmx.Ok(value),
              _ => dmx.Err(dmx.DecodeError(path, 'bool', value)),
            }),
          )) {
            (
              dmx.Ok(value: final value3),
            ) =>
              dmx.Ok(Triple(
                value1: value1,
                value2: value2,
                value3: value3,
              )),
            (dmx.Err(error: final e),) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Triple', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'value1': value1,
        'value2': value2,
        'value3': value3,
      };
}

/// Loose — exactly one of the cases below, told apart by shape
/// rather than by a tag.
sealed class Loose {
  /// The shared constructor every case delegates to.
  const Loose();
}

/// The `Left` case of Loose, as an immutable value.
final class Left extends Loose {
  /// Every field, in the order the diagram declares them.
  const Left({required this.value}) : super();

  /// The `value` field, declared as `Int`.
  final int value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Left &&
          other.value == value);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value,
      );

  @override
  String toString() => 'Left(value: $value)';

  /// A copy of this value with the named fields replaced.
  Left copyWith({
    int? value,
  }) =>
      Left(
        value: value ?? this.value,
      );
}

/// JSON for [Left].
extension LeftJson on Left {
  /// Decodes a `Left` from a JSON value, or says why it could not.
  static dmx.Result<Left, dmx.DecodeError> fromJson(Object? json, [String path = 'Left']) =>
      switch (json) {
        {
          'value': final int value,
        } =>
          dmx.Ok(Left(
            value: value,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Left', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'value': value,
      };
}

/// The `Right` case of Loose, as an immutable value.
final class Right extends Loose {
  /// Every field, in the order the diagram declares them.
  const Right({required this.value}) : super();

  /// The `value` field, declared as `String`.
  final String value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Right &&
          other.value == value);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value,
      );

  @override
  String toString() => 'Right(value: $value)';

  /// A copy of this value with the named fields replaced.
  Right copyWith({
    String? value,
  }) =>
      Right(
        value: value ?? this.value,
      );
}

/// JSON for [Right].
extension RightJson on Right {
  /// Decodes a `Right` from a JSON value, or says why it could not.
  static dmx.Result<Right, dmx.DecodeError> fromJson(Object? json, [String path = 'Right']) =>
      switch (json) {
        {
          'value': final String value,
        } =>
          dmx.Ok(Right(
            value: value,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Right', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'value': value,
      };
}
