// dmx: generated from docs/unions.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition 5214cc1d7a2b8b4d, template ebcc1789a3d0fa99, context v1, dmx 0.0.0.

// Generated from docs/unions.dmx.md. Edit the diagram, not this file.

/// Shape — exactly one of the cases below.
sealed class Shape {
  /// The shared constructor every case delegates to.
  const Shape();
}

/// The `Circle` case of Shape.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ShapeCircle extends Shape {
  /// This case's payload, in diagram order.
  const ShapeCircle({required this.radius}) : super();

  /// The `radius` member, declared as `Float`.
  final double radius;
}

/// The `Rectangle` case of Shape.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ShapeRectangle extends Shape {
  /// This case's payload, in diagram order.
  const ShapeRectangle({required this.width, required this.height}) : super();

  /// The `width` member, declared as `Float`.
  final double width;

  /// The `height` member, declared as `Float`.
  final double height;
}

/// The `Triangle` case of Shape.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ShapeTriangle extends Shape {
  /// This case's payload, in diagram order.
  const ShapeTriangle({required this.a, required this.b, required this.c}) : super();

  /// The `a` member, declared as `Float`.
  final double a;

  /// The `b` member, declared as `Float`.
  final double b;

  /// The `c` member, declared as `Float`.
  final double c;
}

/// The `Point` case of Shape.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ShapePoint extends Shape {
  /// This case's payload, in diagram order.
  const ShapePoint() : super();
}

/// Error code — exactly one of the cases below.
sealed class ErrorCode {
  /// The shared constructor every case delegates to.
  const ErrorCode();
}

/// The `ParseError` case of ErrorCode.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ErrorCodeParseError extends ErrorCode {
  /// This case's payload, in diagram order.
  const ErrorCodeParseError() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32700;
}

/// The `InvalidRequest` case of ErrorCode.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ErrorCodeInvalidRequest extends ErrorCode {
  /// This case's payload, in diagram order.
  const ErrorCodeInvalidRequest() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32600;
}

/// The `MethodNotFound` case of ErrorCode.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ErrorCodeMethodNotFound extends ErrorCode {
  /// This case's payload, in diagram order.
  const ErrorCodeMethodNotFound() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = -32601;
}

/// The `Ok` case of ErrorCode.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ErrorCodeOk extends ErrorCode {
  /// This case's payload, in diagram order.
  const ErrorCodeOk() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = 0;
}

/// The `Grouped` case of ErrorCode.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ErrorCodeGrouped extends ErrorCode {
  /// This case's payload, in diagram order.
  const ErrorCodeGrouped() : super();

  /// The discriminant the diagram gives this case.
  static const int discriminant = 1_000;
}

/// Option — exactly one of the cases below.
sealed class Option<T> {
  /// The shared constructor every case delegates to.
  const Option();
}

/// The `Some` case of Option.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class OptionSome<T> extends Option<T> {
  /// This case's payload, in diagram order.
  const OptionSome({required this.value}) : super();

  /// The `value` member, declared as `T`.
  final T value;
}

/// The `None` case of Option.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class OptionNone<T> extends Option<T> {
  /// This case's payload, in diagram order.
  const OptionNone() : super();
}

/// Result — exactly one of the cases below.
sealed class Result<T, E> {
  /// The shared constructor every case delegates to.
  const Result();
}

/// The `Ok` case of Result.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ResultOk<T, E> extends Result<T, E> {
  /// This case's payload, in diagram order.
  const ResultOk({required this.value}) : super();

  /// The `value` member, declared as `T`.
  final T value;
}

/// The `Err` case of Result.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class ResultErr<T, E> extends Result<T, E> {
  /// This case's payload, in diagram order.
  const ResultErr({required this.error}) : super();

  /// The `error` member, declared as `E`.
  final E error;
}

/// Request id — exactly one of the cases below.
sealed class RequestId {
  /// The shared constructor every case delegates to.
  const RequestId();
}

/// The `Number` case of RequestId.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class RequestIdNumber extends RequestId {
  /// This case's payload, in diagram order.
  const RequestIdNumber({required this.value1}) : super();

  /// The `value1` member, declared as `Int`.
  final int value1;
}

/// The `String` case of RequestId.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class RequestIdString extends RequestId {
  /// This case's payload, in diagram order.
  const RequestIdString({required this.value1}) : super();

  /// The `value1` member, declared as `String`.
  final String value1;
}

/// The `Triple` case of RequestId.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class RequestIdTriple extends RequestId {
  /// This case's payload, in diagram order.
  const RequestIdTriple({required this.value1, required this.value2, required this.value3}) : super();

  /// The `value1` member, declared as `Int`.
  final int value1;

  /// The `value2` member, declared as `String`.
  final String value2;

  /// The `value3` member, declared as `List<Bool>`.
  final List<bool> value3;
}

/// Loose — exactly one of the cases below, told apart by shape
/// rather than by a tag.
sealed class Loose {
  /// The shared constructor every case delegates to.
  const Loose();
}

/// The `Left` case of Loose.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class LooseLeft extends Loose {
  /// This case's payload, in diagram order.
  const LooseLeft({required this.value}) : super();

  /// The `value` member, declared as `Int`.
  final int value;
}

/// The `Right` case of Loose.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class LooseRight extends Loose {
  /// This case's payload, in diagram order.
  const LooseRight({required this.value}) : super();

  /// The `value` member, declared as `String`.
  final String value;
}
