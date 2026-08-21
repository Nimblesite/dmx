// dmx: generated from models/shipping.td — do not edit.
// dmx: rendered through the canonical model template, definition bd16c86d530f3daa, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/shipping.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// Parcel — an immutable value from the diagram.
final class Parcel {
  /// Every field, in the order the diagram declares them.
  const Parcel({required this.id, required this.weightG, this.insured, required this.labels});

  /// The `id` field, declared as `Uuid`.
  final String id;

  /// The `weightG` field, declared as `Int`.
  final int weightG;

  /// The `insured` field, declared as `Option<Decimal>`.
  final String? insured;

  /// The `labels` field, declared as `List<String>`.
  final List<String> labels;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Parcel &&
          other.id == id &&
          other.weightG == weightG &&
          other.insured == insured &&
          dmx.dmxDeepEquals(other.labels, labels));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        weightG,
        insured,
        dmx.dmxDeepHash(labels),
      );

  @override
  String toString() => 'Parcel(id: $id, weightG: $weightG, insured: $insured, labels: $labels)';

  /// A copy of this value with the named fields replaced.
  Parcel copyWith({
    String? id,
    int? weightG,
    dmx.DmxPatch<String?> insured = const dmx.DmxKeep(),
    List<String>? labels,
  }) =>
      Parcel(
        id: id ?? this.id,
        weightG: weightG ?? this.weightG,
        insured: switch (insured) { dmx.DmxKeep() => this.insured, dmx.DmxTo(value: final value) => value },
        labels: labels ?? this.labels,
      );
}

/// JSON for [Parcel].
extension ParcelJson on Parcel {
  /// Decodes a `Parcel` from a JSON value, or says why it could not.
  static dmx.Result<Parcel, dmx.DecodeError> fromJson(Object? json, [String path = 'Parcel']) =>
      switch (json) {
        {
          'id': final String id,
          'weightG': final int weightG,
          'labels': final List<dynamic> labels,
        } =>
          switch ((
            dmx.dmxNullable<String>(dmx.dmxKey(json, 'insured'), '$path.insured', (value, path) => switch (value) {
              final String value => dmx.Ok(value),
              _ => dmx.Err(dmx.DecodeError(path, 'String', value)),
            }),
            dmx.dmxList<String>(labels, '$path.labels', (value, path) => switch (value) {
              final String value => dmx.Ok(value),
              _ => dmx.Err(dmx.DecodeError(path, 'String', value)),
            }),
          )) {
            (
              dmx.Ok(value: final insured),
              dmx.Ok(value: final labels),
            ) =>
              dmx.Ok(Parcel(
                id: id,
                weightG: weightG,
                insured: insured,
                labels: labels,
              )),
            (dmx.Err(error: final e), _) => dmx.Err(e),
            (_, dmx.Err(error: final e)) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Parcel', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'id': id,
        'weightG': weightG,
        'insured': insured,
        'labels': labels,
      };
}

/// Leg — exactly one of the cases below.
sealed class Leg {
  /// The shared constructor every case delegates to.
  const Leg();
}

/// JSON for [Leg].
extension LegJson on Leg {
  /// Decodes whichever case the payload's 'type' names.
  static dmx.Result<Leg, dmx.DecodeError> fromJson(Object? json, [String path = 'Leg']) =>
      switch (json) {
        {
          'type': final String type,
        } =>
          switch (type) {
            'pickup' => PickupJson.fromJson(json, path),
            'transit' => TransitJson.fromJson(json, path),
            'delivered' => DeliveredJson.fromJson(json, path),
            _ => dmx.Err(dmx.DecodeError(path, 'Leg', json)),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Leg', json)),
      };

  /// This value as a JSON map, tagged with the case it is.
  Map<String, Object?> toJson() => switch (this) {
        final Pickup value => <String, Object?>{
            'type': 'pickup',
            ...value.toJson(),
          },
        final Transit value => <String, Object?>{
            'type': 'transit',
            ...value.toJson(),
          },
        final Delivered value => <String, Object?>{
            'type': 'delivered',
            ...value.toJson(),
          },
      };
}

/// The `Pickup` case of Leg, as an immutable value.
final class Pickup extends Leg {
  /// Every field, in the order the diagram declares them.
  const Pickup({required this.at}) : super();

  /// The `at` field, declared as `DateTime`.
  final DateTime at;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Pickup &&
          other.at == at);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        at,
      );

  @override
  String toString() => 'Pickup(at: $at)';

  /// A copy of this value with the named fields replaced.
  Pickup copyWith({
    DateTime? at,
  }) =>
      Pickup(
        at: at ?? this.at,
      );
}

/// JSON for [Pickup].
extension PickupJson on Pickup {
  /// Decodes a `Pickup` from a JSON value, or says why it could not.
  static dmx.Result<Pickup, dmx.DecodeError> fromJson(Object? json, [String path = 'Pickup']) =>
      switch (json) {
        {
          'at': final String at,
        } =>
          switch ((
            switch (DateTime.tryParse(at)) { final DateTime parsed => dmx.Ok<DateTime, dmx.DecodeError>(parsed), null => dmx.Err<DateTime, dmx.DecodeError>(dmx.DecodeError('$path.at', 'DateTime', at)) },
          )) {
            (
              dmx.Ok(value: final at),
            ) =>
              dmx.Ok(Pickup(
                at: at,
              )),
            (dmx.Err(error: final e),) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Pickup', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'at': at.toIso8601String(),
      };
}

/// The `Transit` case of Leg, as an immutable value.
final class Transit extends Leg {
  /// Every field, in the order the diagram declares them.
  const Transit({required this.carrier, required this.etaHours}) : super();

  /// The `carrier` field, declared as `String`.
  final String carrier;

  /// The `etaHours` field, declared as `Int`.
  final int etaHours;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Transit &&
          other.carrier == carrier &&
          other.etaHours == etaHours);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        carrier,
        etaHours,
      );

  @override
  String toString() => 'Transit(carrier: $carrier, etaHours: $etaHours)';

  /// A copy of this value with the named fields replaced.
  Transit copyWith({
    String? carrier,
    int? etaHours,
  }) =>
      Transit(
        carrier: carrier ?? this.carrier,
        etaHours: etaHours ?? this.etaHours,
      );
}

/// JSON for [Transit].
extension TransitJson on Transit {
  /// Decodes a `Transit` from a JSON value, or says why it could not.
  static dmx.Result<Transit, dmx.DecodeError> fromJson(Object? json, [String path = 'Transit']) =>
      switch (json) {
        {
          'carrier': final String carrier,
          'etaHours': final int etaHours,
        } =>
          dmx.Ok(Transit(
            carrier: carrier,
            etaHours: etaHours,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Transit', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'carrier': carrier,
        'etaHours': etaHours,
      };
}

/// The `Delivered` case of Leg, as an immutable value.
final class Delivered extends Leg {
  /// Every field, in the order the diagram declares them.
  const Delivered({required this.at, this.signedBy}) : super();

  /// The `at` field, declared as `DateTime`.
  final DateTime at;

  /// The `signedBy` field, declared as `Option<String>`.
  final String? signedBy;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Delivered &&
          other.at == at &&
          other.signedBy == signedBy);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        at,
        signedBy,
      );

  @override
  String toString() => 'Delivered(at: $at, signedBy: $signedBy)';

  /// A copy of this value with the named fields replaced.
  Delivered copyWith({
    DateTime? at,
    dmx.DmxPatch<String?> signedBy = const dmx.DmxKeep(),
  }) =>
      Delivered(
        at: at ?? this.at,
        signedBy: switch (signedBy) { dmx.DmxKeep() => this.signedBy, dmx.DmxTo(value: final value) => value },
      );
}

/// JSON for [Delivered].
extension DeliveredJson on Delivered {
  /// Decodes a `Delivered` from a JSON value, or says why it could not.
  static dmx.Result<Delivered, dmx.DecodeError> fromJson(Object? json, [String path = 'Delivered']) =>
      switch (json) {
        {
          'at': final String at,
        } =>
          switch ((
            switch (DateTime.tryParse(at)) { final DateTime parsed => dmx.Ok<DateTime, dmx.DecodeError>(parsed), null => dmx.Err<DateTime, dmx.DecodeError>(dmx.DecodeError('$path.at', 'DateTime', at)) },
            dmx.dmxNullable<String>(dmx.dmxKey(json, 'signedBy'), '$path.signedBy', (value, path) => switch (value) {
              final String value => dmx.Ok(value),
              _ => dmx.Err(dmx.DecodeError(path, 'String', value)),
            }),
          )) {
            (
              dmx.Ok(value: final at),
              dmx.Ok(value: final signedBy),
            ) =>
              dmx.Ok(Delivered(
                at: at,
                signedBy: signedBy,
              )),
            (dmx.Err(error: final e), _) => dmx.Err(e),
            (_, dmx.Err(error: final e)) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Delivered', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'at': at.toIso8601String(),
        'signedBy': signedBy,
      };
}

/// `TrackingNumber` as the diagram declares it.
typedef TrackingNumber = String;

/// Shipment — an immutable value from the diagram.
final class Shipment {
  /// Every field, in the order the diagram declares them.
  const Shipment({required this.parcel, required this.legs, required this.tracking});

  /// The `parcel` field, declared as `Parcel`.
  final Parcel parcel;

  /// The `legs` field, declared as `List<Leg>`.
  final List<Leg> legs;

  /// The `tracking` field, declared as `TrackingNumber`.
  final TrackingNumber tracking;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Shipment &&
          other.parcel == parcel &&
          dmx.dmxDeepEquals(other.legs, legs) &&
          other.tracking == tracking);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        parcel,
        dmx.dmxDeepHash(legs),
        tracking,
      );

  @override
  String toString() => 'Shipment(parcel: $parcel, legs: $legs, tracking: $tracking)';

  /// A copy of this value with the named fields replaced.
  Shipment copyWith({
    Parcel? parcel,
    List<Leg>? legs,
    TrackingNumber? tracking,
  }) =>
      Shipment(
        parcel: parcel ?? this.parcel,
        legs: legs ?? this.legs,
        tracking: tracking ?? this.tracking,
      );
}

/// JSON for [Shipment].
extension ShipmentJson on Shipment {
  /// Decodes a `Shipment` from a JSON value, or says why it could not.
  static dmx.Result<Shipment, dmx.DecodeError> fromJson(Object? json, [String path = 'Shipment']) =>
      switch (json) {
        {
          'parcel': final Object? parcel,
          'legs': final List<dynamic> legs,
          'tracking': final String tracking,
        } =>
          switch ((
            ParcelJson.fromJson(parcel, '$path.parcel'),
            dmx.dmxList<Leg>(legs, '$path.legs', LegJson.fromJson),
          )) {
            (
              dmx.Ok(value: final parcel),
              dmx.Ok(value: final legs),
            ) =>
              dmx.Ok(Shipment(
                parcel: parcel,
                legs: legs,
                tracking: tracking,
              )),
            (dmx.Err(error: final e), _) => dmx.Err(e),
            (_, dmx.Err(error: final e)) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'Shipment', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'parcel': parcel.toJson(),
        'legs': legs.map((e0) => e0.toJson()).toList(),
        'tracking': tracking,
      };
}
