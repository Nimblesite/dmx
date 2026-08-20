// dmx: generated from docs/shipping.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition bd16c86d530f3daa, template 861e8207f9496f03, context v1, dmx 0.0.0.

// Generated from docs/shipping.dmx.md. Edit the diagram, not this file.

/// Parcel, generated from the shipping diagram.
final class Parcel {
  /// Every field of Parcel, in the order the diagram declares them.
  const Parcel({required this.id, required this.weightG, this.insured, required this.labels});

  /// The `id` field, declared as `Uuid`.
  final String id;

  /// The `weightG` field, declared as `Int`.
  final int weightG;

  /// The `insured` field, declared as `Option<Decimal>`.
  final String? insured;

  /// The `labels` field, declared as `List<String>`.
  final List<String> labels;
}

/// Leg — exactly one of the variants below.
sealed class Leg {
  /// The shared constructor every variant delegates to.
  const Leg();
}

/// The `Pickup` case of Leg.
final class Pickup extends Leg {
  /// Every field of this case, in diagram order.
  const Pickup({required this.at}) : super();

  /// The `at` field, declared as `DateTime`.
  final DateTime at;
}

/// The `Transit` case of Leg.
final class Transit extends Leg {
  /// Every field of this case, in diagram order.
  const Transit({required this.carrier, required this.etaHours}) : super();

  /// The `carrier` field, declared as `String`.
  final String carrier;

  /// The `etaHours` field, declared as `Int`.
  final int etaHours;
}

/// The `Delivered` case of Leg.
final class Delivered extends Leg {
  /// Every field of this case, in diagram order.
  const Delivered({required this.at, this.signedBy}) : super();

  /// The `at` field, declared as `DateTime`.
  final DateTime at;

  /// The `signedBy` field, declared as `Option<String>`.
  final String? signedBy;
}

/// `TrackingNumber` as the diagram declares it.
typedef TrackingNumber = String;

/// Shipment, generated from the shipping diagram.
final class Shipment {
  /// Every field of Shipment, in the order the diagram declares them.
  const Shipment({required this.parcel, required this.legs, required this.tracking});

  /// The `parcel` field, declared as `Parcel`.
  final Parcel parcel;

  /// The `legs` field, declared as `List<Leg>`.
  final List<Leg> legs;

  /// The `tracking` field, declared as `TrackingNumber`.
  final TrackingNumber tracking;
}
