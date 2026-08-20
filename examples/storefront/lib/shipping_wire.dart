// dmx: generated from docs/shipping.dmx.md — do not edit.
// dmx: group 1, fences 1/3, definition bd16c86d530f3daa, template 6737510090829881, context v1, dmx 0.0.0.

// Generated from docs/shipping.dmx.md. Edit the diagram, not this file.

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
