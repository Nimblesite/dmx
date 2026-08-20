// dmx: generated from docs/scalars.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition 7db716b44e16128d, template ebcc1789a3d0fa99, context v1, dmx 0.0.0.

// Generated from docs/scalars.dmx.md. Edit the diagram, not this file.

/// Scalars — a record from the diagram.
final class Scalars {
  /// Every field, in the order the diagram declares them.
  const Scalars({required this.flag, required this.count, required this.ratio, required this.text, required this.blob, required this.nothing, required this.at, required this.id, required this.amount, required this.tags, required this.index, this.maybe, required this.anything, this.deep});

  /// The `flag` field, declared as `Bool`.
  final bool flag;

  /// The `count` field, declared as `Int`.
  final int count;

  /// The `ratio` field, declared as `Float`.
  final double ratio;

  /// The `text` field, declared as `String`.
  final String text;

  /// The `blob` field, declared as `Bytes`.
  final List<int> blob;

  /// The `nothing` field, declared as `Unit`.
  final void nothing;

  /// The `at` field, declared as `DateTime`.
  final DateTime at;

  /// The `id` field, declared as `Uuid`.
  final Uuid id;

  /// The `amount` field, declared as `Decimal`.
  final String amount;

  /// The `tags` field, declared as `List<String>`.
  final List<String> tags;

  /// The `index` field, declared as `Map<String, List<Option<Decimal>>>`.
  final Map<String, List<String?>> index;

  /// The `maybe` field, declared as `Option<Int>`.
  final int? maybe;

  /// The `anything` field, declared as `Any`.
  final Object anything;

  /// The `deep` field, declared as `Option<Option<Map<Uuid, List<Any>>>>`.
  final Map<Uuid, List<Object>>? deep;
}

/// `Uuid` as the diagram declares it.
typedef Uuid = String;
