// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('diff')` [catalogue.diff] — what changed, as data.
//
// Audit trails, optimistic-update reconciliation, "unsaved changes" banners,
// and half of every sync engine all want the same thing: the list of fields
// that differ between two versions of an object. Written by hand it is one
// `if` per field, and the field somebody forgot to add is invisible — the code
// still compiles, still runs, and silently under-reports.
//
// Generated from the field list, that failure mode does not exist. Collections
// compare by content, matching `==`, so a reordered list of locations is a
// change and a rebuilt-but-identical map is not.

import 'package:dmx/dmx.dart';

/// How much of one product exists, and where.
@dmx('model', {'fieldRename': 'snake'})
@dmx('diff')
class StockLevel {
  const StockLevel({
    required this.sku,
    required this.onHand,
    required this.reserved,
    required this.reorderPoint,
    required this.locations,
    this.discontinuedAt,
  });

  final String sku;
  final int onHand;
  final int reserved;
  final int reorderPoint;

  /// Warehouse code to quantity.
  final Map<String, int> locations;

  final DateTime? discontinuedAt;

  /// Hand-written, above the divider, and untouched by either macro.
  int get available => onHand - reserved;

  bool get needsReorder => available <= reorderPoint;

  //#region
  static Result<StockLevel, DecodeError> fromJson(Object? json, [String path = 'StockLevel']) =>
      switch (json) {
        {
          'sku': final String sku,
          'on_hand': final int onHand,
          'reserved': final int reserved,
          'reorder_point': final int reorderPoint,
          'locations': final Map<String, dynamic> locations,
        } =>
          switch ((
            dmxMap<int>(locations, '$path.locations', (value, path) => switch (value) {
              final int value => Ok(value),
              _ => Err(DecodeError(path, 'int', value)),
            }),
            dmxNullable<DateTime>(dmxKey(json, 'discontinued_at'), '$path.discontinued_at', (value, path) => switch (value) {
              final String value => switch (DateTime.tryParse(value)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError(path, 'DateTime', value)) },
              _ => Err(DecodeError(path, 'DateTime', value)),
            }),
          )) {
            (
              Ok(value: final locations),
              Ok(value: final discontinuedAt),
            ) =>
              Ok(StockLevel(
                sku: sku,
                onHand: onHand,
                reserved: reserved,
                reorderPoint: reorderPoint,
                locations: locations,
                discontinuedAt: discontinuedAt,
              )),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'StockLevel', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'sku': sku,
        'on_hand': onHand,
        'reserved': reserved,
        'reorder_point': reorderPoint,
        'locations': locations,
        'discontinued_at': discontinuedAt?.toIso8601String(),
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is StockLevel &&
          other.sku == sku &&
          other.onHand == onHand &&
          other.reserved == reserved &&
          other.reorderPoint == reorderPoint &&
          dmxDeepEquals(other.locations, locations) &&
          other.discontinuedAt == discontinuedAt);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        sku,
        onHand,
        reserved,
        reorderPoint,
        dmxDeepHash(locations),
        discontinuedAt,
      );

  @override
  String toString() => 'StockLevel(sku: $sku, onHand: $onHand, reserved: $reserved, reorderPoint: $reorderPoint, locations: $locations, discontinuedAt: $discontinuedAt)';

  StockLevel copyWith({
    String? sku,
    int? onHand,
    int? reserved,
    int? reorderPoint,
    Map<String, int>? locations,
    DmxPatch<DateTime?> discontinuedAt = const DmxKeep(),
  }) =>
      StockLevel(
        sku: sku ?? this.sku,
        onHand: onHand ?? this.onHand,
        reserved: reserved ?? this.reserved,
        reorderPoint: reorderPoint ?? this.reorderPoint,
        locations: locations ?? this.locations,
        discontinuedAt: switch (discontinuedAt) { DmxKeep() => this.discontinuedAt, DmxTo(value: final value) => value },
      );

  /// Every field that differs, in field order. Collection fields compare by
  /// content, so this agrees with `==` rather than with identity.
  ///
  /// Nothing here is reflective: the field list is fixed at generation time,
  /// so adding a field adds a line on the next build and cannot be forgotten.
  List<DmxChange> diff(StockLevel other) => <DmxChange>[
        if (other.sku != sku)
          DmxChange('sku', sku, other.sku),
        if (other.onHand != onHand)
          DmxChange('on_hand', onHand, other.onHand),
        if (other.reserved != reserved)
          DmxChange('reserved', reserved, other.reserved),
        if (other.reorderPoint != reorderPoint)
          DmxChange('reorder_point', reorderPoint, other.reorderPoint),
        if (!dmxDeepEquals(other.locations, locations))
          DmxChange('locations', locations, other.locations),
        if (other.discontinuedAt != discontinuedAt)
          DmxChange('discontinued_at', discontinuedAt, other.discontinuedAt),
      ];

  /// The names alone, for a "3 unsaved changes" badge that does not need the
  /// values behind them.
  List<String> changedFields(StockLevel other) =>
      <String>[for (final change in diff(other)) change.field];

  bool differsFrom(StockLevel other) => diff(other).isNotEmpty;
  //#endregion
}

/// A stock movement, kept for the audit trail. Diffing two [StockLevel]s
/// produces exactly the rows this class stores.
@dmx('model', {'fieldRename': 'snake'})
@dmx('diff')
class StockAdjustment {
  const StockAdjustment({
    required this.sku,
    required this.delta,
    required this.reason,
    required this.at,
    required this.operator,
  });

  final String sku;
  final int delta;
  final String reason;
  final DateTime at;

  /// Named `operator`, which is a reserved word in a great many generators.
  /// It is not one here, because nothing is being pasted into a position where
  /// it could be mistaken for one.
  final String operator;

  //#region
  static Result<StockAdjustment, DecodeError> fromJson(Object? json, [String path = 'StockAdjustment']) =>
      switch (json) {
        {
          'sku': final String sku,
          'delta': final int delta,
          'reason': final String reason,
          'at': final String at,
          'operator': final String operator,
        } =>
          switch ((
            switch (DateTime.tryParse(at)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError('$path.at', 'DateTime', at)) },
          )) {
            (
              Ok(value: final at),
            ) =>
              Ok(StockAdjustment(
                sku: sku,
                delta: delta,
                reason: reason,
                at: at,
                operator: operator,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'StockAdjustment', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'sku': sku,
        'delta': delta,
        'reason': reason,
        'at': at.toIso8601String(),
        'operator': operator,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is StockAdjustment &&
          other.sku == sku &&
          other.delta == delta &&
          other.reason == reason &&
          other.at == at &&
          other.operator == operator);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        sku,
        delta,
        reason,
        at,
        operator,
      );

  @override
  String toString() => 'StockAdjustment(sku: $sku, delta: $delta, reason: $reason, at: $at, operator: $operator)';

  StockAdjustment copyWith({
    String? sku,
    int? delta,
    String? reason,
    DateTime? at,
    String? operator,
  }) =>
      StockAdjustment(
        sku: sku ?? this.sku,
        delta: delta ?? this.delta,
        reason: reason ?? this.reason,
        at: at ?? this.at,
        operator: operator ?? this.operator,
      );

  /// Every field that differs, in field order. Collection fields compare by
  /// content, so this agrees with `==` rather than with identity.
  ///
  /// Nothing here is reflective: the field list is fixed at generation time,
  /// so adding a field adds a line on the next build and cannot be forgotten.
  List<DmxChange> diff(StockAdjustment other) => <DmxChange>[
        if (other.sku != sku)
          DmxChange('sku', sku, other.sku),
        if (other.delta != delta)
          DmxChange('delta', delta, other.delta),
        if (other.reason != reason)
          DmxChange('reason', reason, other.reason),
        if (other.at != at)
          DmxChange('at', at, other.at),
        if (other.operator != operator)
          DmxChange('operator', operator, other.operator),
      ];

  /// The names alone, for a "3 unsaved changes" badge that does not need the
  /// values behind them.
  List<String> changedFields(StockAdjustment other) =>
      <String>[for (final change in diff(other)) change.field];

  bool differsFrom(StockAdjustment other) => diff(other).isNotEmpty;
  //#endregion
}
