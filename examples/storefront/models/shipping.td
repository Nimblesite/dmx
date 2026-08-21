# A parcel on its way to a customer.
type Parcel {
  id:      Uuid
  weightG: Int
  insured: Option<Decimal>
  labels:  List<String>
}

# Where the parcel has got to. One of these, never two.
union Leg {
  Pickup    { at: DateTime }
  Transit   { carrier: String, etaHours: Int }
  Delivered { at: DateTime, signedBy: Option<String> }
}

alias TrackingNumber = String

type Shipment {
  parcel:   Parcel
  legs:     List<Leg>
  tracking: TrackingNumber
}
