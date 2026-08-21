// dmx: generated from models/scalars.td — do not edit.
// dmx: rendered through the canonical model template, definition 7db716b44e16128d, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/scalars.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// Scalars — an immutable value from the diagram.
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

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Scalars &&
          other.flag == flag &&
          other.count == count &&
          other.ratio == ratio &&
          other.text == text &&
          dmx.dmxDeepEquals(other.blob, blob) &&
          other.at == at &&
          other.id == id &&
          other.amount == amount &&
          dmx.dmxDeepEquals(other.tags, tags) &&
          dmx.dmxDeepEquals(other.index, index) &&
          other.maybe == maybe &&
          other.anything == anything &&
          dmx.dmxDeepEquals(other.deep, deep));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        flag,
        count,
        ratio,
        text,
        dmx.dmxDeepHash(blob),
        at,
        id,
        amount,
        dmx.dmxDeepHash(tags),
        dmx.dmxDeepHash(index),
        maybe,
        anything,
        dmx.dmxDeepHash(deep),
      );

  @override
  String toString() => 'Scalars(flag: $flag, count: $count, ratio: $ratio, text: $text, blob: $blob, at: $at, id: $id, amount: $amount, tags: $tags, index: $index, maybe: $maybe, anything: $anything, deep: $deep)';
}

/// `Uuid` as the diagram declares it.
typedef Uuid = String;
