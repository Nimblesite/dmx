// dmx: generated from models/shipping.td — do not edit.
// dmx: rendered through models/shipping.wire.mustache, definition bd16c86d530f3daa, template 40b2301c361563df, context v1, dmx 0.0.0.

// Generated from models/shipping.td. Edit the definition, not this file.

/// The wire name of every field, keyed by declaration and then by Dart name.
const shippingWireNames = <String, Map<String, String>>{
  'Parcel': <String, String>{
    'id': 'id',
    'weightG': 'weight_g',
    'insured': 'insured',
    'labels': 'labels',
  },
  'Leg.Pickup': <String, String>{
    'at': 'at',
  },
  'Leg.Transit': <String, String>{
    'carrier': 'carrier',
    'etaHours': 'eta_hours',
  },
  'Leg.Delivered': <String, String>{
    'at': 'at',
    'signedBy': 'signed_by',
  },
  'Shipment': <String, String>{
    'parcel': 'parcel',
    'legs': 'legs',
    'tracking': 'tracking',
  },
};

/// Every declaration the shipping diagram carries, in source order.
const shippingDeclarations = <String>[
  'Parcel',
  'Leg',
  'TrackingNumber',
  'Shipment',
];
