// dmx: generated from docs/targeting.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition 6070c6f7e26a9e98, template ebcc1789a3d0fa99, context v1, dmx 0.0.0.

// Generated from docs/targeting.dmx.md. Edit the diagram, not this file.

/// Only dart and rust — a record from the diagram.
final class OnlyDartAndRust {
  /// Every field, in the order the diagram declares them.
  const OnlyDartAndRust({required this.a});

  /// The `a` field, declared as `Int`.
  final int a;
}

/// Not go — a record from the diagram.
final class NotGo {
  /// Every field, in the order the diagram declares them.
  const NotGo({required this.b});

  /// The `b` field, declared as `String`.
  final String b;
}

/// Both — exactly one of the cases below.
sealed class Both {
  /// The shared constructor every case delegates to.
  const Both();
}

/// The `One` case of Both.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class BothOne extends Both {
  /// This case's payload, in diagram order.
  const BothOne() : super();
}

/// The `Two` case of Both.
///
/// The class carries its union's name because variant names collide across
/// unions in one library — `Ok` belongs to two of them here — and a template,
/// not the generator, decides what a case is called.
final class BothTwo extends Both {
  /// This case's payload, in diagram order.
  const BothTwo({required this.x}) : super();

  /// The `x` member, declared as `Int`.
  final int x;
}

/// `Plain` as the diagram declares it.
typedef Plain = int;
