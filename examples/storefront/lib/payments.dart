// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('enum')` [catalogue.enum] — an enum that survives the wire.
//
// Dart enums come with `.name` and `.values`, and that is where the language
// stops. The moment an enum crosses a network boundary you need a wire name
// that is *not* the Dart identifier, a human label that is *not* either of
// them, a decode that fails as data instead of throwing, and — for the enums
// a third party owns — a forward-compatible fallback so that shipping a new
// constant on their side does not take your app down.

import 'package:dmx/dmx.dart';

/// How a customer paid.
///
/// The wire names belong to the payment provider, not to us. `fieldRename`
/// sets the house policy, and `@dmx('value', {'wire': })` pins the two the provider named
/// differently — so the Dart identifiers stay idiomatic and nothing hand-maps
/// strings in a `switch` somebody will forget to update.
@dmx('enum', {'fieldRename': 'snake'})
@dmx('fake')
enum PaymentMethod {
  card,
  applePay,
  googlePay,
  @dmx('value', {'wire': 'ach_transfer', 'label': 'Bank transfer'})
  bankTransfer,
  @dmx('value', {'label': 'Gift card'})
  giftCard;

  //#region
  /// The wire name this constant encodes to.
  String get wire => switch (this) {
        PaymentMethod.card => 'card',
        PaymentMethod.applePay => 'apple_pay',
        PaymentMethod.googlePay => 'google_pay',
        PaymentMethod.bankTransfer => 'ach_transfer',
        PaymentMethod.giftCard => 'gift_card',
      };

  /// A label fit to show a person.
  String get label => switch (this) {
        PaymentMethod.card => 'Card',
        PaymentMethod.applePay => 'Apple pay',
        PaymentMethod.googlePay => 'Google pay',
        PaymentMethod.bankTransfer => 'Bank transfer',
        PaymentMethod.giftCard => 'Gift card',
      };

  String toJson() => wire;

  /// The constant with this wire name, or null when nothing matches.
  static PaymentMethod? tryParse(String wire) => switch (wire) {
        'card' => PaymentMethod.card,
        'apple_pay' => PaymentMethod.applePay,
        'google_pay' => PaymentMethod.googlePay,
        'ach_transfer' => PaymentMethod.bankTransfer,
        'gift_card' => PaymentMethod.giftCard,
        _ => null,
      };

  static Result<PaymentMethod, DecodeError> fromJson(
    Object? json, [
    String path = 'PaymentMethod',
  ]) =>
      switch (json) {
        final String value => switch (tryParse(value)) {
            final PaymentMethod parsed => Ok(parsed),
            null => Err(DecodeError(path, 'PaymentMethod', value)),
          },
        _ => Err(DecodeError(path, 'PaymentMethod', json)),
      };

  bool get isCard => this == PaymentMethod.card;
  bool get isApplePay => this == PaymentMethod.applePay;
  bool get isGooglePay => this == PaymentMethod.googlePay;
  bool get isBankTransfer => this == PaymentMethod.bankTransfer;
  bool get isGiftCard => this == PaymentMethod.giftCard;

  /// A deterministic constant: the same seed always names the same one, so a
  /// fixture that reaches this enum is as stable as the rest of it.
  static PaymentMethod fake({int seed = 0}) =>
      PaymentMethod.values[seed % PaymentMethod.values.length];

  /// `count` constants, walking the declaration order and wrapping.
  static List<PaymentMethod> fakes(int count, {int seed = 0}) =>
      List<PaymentMethod>.generate(count, (index) => fake(seed: seed + index));
  //#endregion
}

/// Why money went back.
///
/// `unknown:` makes the decode total: a reason this build has never heard of
/// arrives as [RefundReason.other] instead of failing the whole payload. That
/// is the correct posture for any enum whose values someone else can add to
/// while your app is already in the store.
@dmx('enum', {'fieldRename': 'screaming_snake', 'unknown': RefundReason.other})
enum RefundReason {
  duplicateCharge,
  itemNotReceived,
  @dmx('value', {'label': 'Not as described'})
  notAsDescribed,
  fraudulent,
  requestedByCustomer,
  other;

  //#region
  /// The wire name this constant encodes to.
  String get wire => switch (this) {
        RefundReason.duplicateCharge => 'DUPLICATE_CHARGE',
        RefundReason.itemNotReceived => 'ITEM_NOT_RECEIVED',
        RefundReason.notAsDescribed => 'NOT_AS_DESCRIBED',
        RefundReason.fraudulent => 'FRAUDULENT',
        RefundReason.requestedByCustomer => 'REQUESTED_BY_CUSTOMER',
        RefundReason.other => 'OTHER',
      };

  /// A label fit to show a person.
  String get label => switch (this) {
        RefundReason.duplicateCharge => 'Duplicate charge',
        RefundReason.itemNotReceived => 'Item not received',
        RefundReason.notAsDescribed => 'Not as described',
        RefundReason.fraudulent => 'Fraudulent',
        RefundReason.requestedByCustomer => 'Requested by customer',
        RefundReason.other => 'Other',
      };

  String toJson() => wire;

  /// The constant with this wire name, or null when nothing matches.
  static RefundReason? tryParse(String wire) => switch (wire) {
        'DUPLICATE_CHARGE' => RefundReason.duplicateCharge,
        'ITEM_NOT_RECEIVED' => RefundReason.itemNotReceived,
        'NOT_AS_DESCRIBED' => RefundReason.notAsDescribed,
        'FRAUDULENT' => RefundReason.fraudulent,
        'REQUESTED_BY_CUSTOMER' => RefundReason.requestedByCustomer,
        'OTHER' => RefundReason.other,
        _ => null,
      };

  /// An unrecognised wire name decodes to [RefundReason.other] rather than
  /// failing: this enum is declared `unknown:`, so a value added upstream
  /// after this build shipped is data, not an outage.
  static Result<RefundReason, DecodeError> fromJson(
    Object? json, [
    String path = 'RefundReason',
  ]) =>
      switch (json) {
        final String value => Ok(tryParse(value) ?? RefundReason.other),
        _ => Err(DecodeError(path, 'RefundReason', json)),
      };

  bool get isDuplicateCharge => this == RefundReason.duplicateCharge;
  bool get isItemNotReceived => this == RefundReason.itemNotReceived;
  bool get isNotAsDescribed => this == RefundReason.notAsDescribed;
  bool get isFraudulent => this == RefundReason.fraudulent;
  bool get isRequestedByCustomer => this == RefundReason.requestedByCustomer;
  bool get isOther => this == RefundReason.other;
  //#endregion
}

/// Enums carry behaviour too. `@dmx('enum')` never touches what you wrote above the
/// divider, so a hand-written member sits next to five generated ones without
/// a mixin, a part file, or an extension in a third place.
@dmx('enum')
enum Fulfilment {
  pending,
  packed,
  shipped,
  delivered;

  /// Hand-written, author-owned, untouched by generation.
  bool get isTerminal => this == Fulfilment.delivered;

  //#region
  /// The wire name this constant encodes to.
  String get wire => switch (this) {
        Fulfilment.pending => 'pending',
        Fulfilment.packed => 'packed',
        Fulfilment.shipped => 'shipped',
        Fulfilment.delivered => 'delivered',
      };

  /// A label fit to show a person.
  String get label => switch (this) {
        Fulfilment.pending => 'Pending',
        Fulfilment.packed => 'Packed',
        Fulfilment.shipped => 'Shipped',
        Fulfilment.delivered => 'Delivered',
      };

  String toJson() => wire;

  /// The constant with this wire name, or null when nothing matches.
  static Fulfilment? tryParse(String wire) => switch (wire) {
        'pending' => Fulfilment.pending,
        'packed' => Fulfilment.packed,
        'shipped' => Fulfilment.shipped,
        'delivered' => Fulfilment.delivered,
        _ => null,
      };

  static Result<Fulfilment, DecodeError> fromJson(
    Object? json, [
    String path = 'Fulfilment',
  ]) =>
      switch (json) {
        final String value => switch (tryParse(value)) {
            final Fulfilment parsed => Ok(parsed),
            null => Err(DecodeError(path, 'Fulfilment', value)),
          },
        _ => Err(DecodeError(path, 'Fulfilment', json)),
      };

  bool get isPending => this == Fulfilment.pending;
  bool get isPacked => this == Fulfilment.packed;
  bool get isShipped => this == Fulfilment.shipped;
  bool get isDelivered => this == Fulfilment.delivered;
  //#endregion
}
