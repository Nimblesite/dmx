// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('validate')` [catalogue.validate] — rules next to the field they constrain.
//
// The rule that matters here is that validation *accumulates*. A form that
// reports one problem, gets fixed, and then reports the next one is how you
// lose a customer at checkout. `validate()` returns every violation at once,
// as data, keyed by field — ready to hang off a text field's `errorText`
// without a single exception in sight.

import 'package:dmx/dmx.dart';

/// What the customer typed. Constraints live on the fields, so the rule and
/// the thing it constrains cannot drift apart in review.
@dmx('model', {'fieldRename': 'snake'})
@dmx('validate')
class CheckoutForm {
  const CheckoutForm({
    required this.email,
    required this.postcode,
    required this.quantity,
    required this.acceptsTerms,
    this.giftMessage,
    this.discountCode,
  });

  @dmx('check.notEmpty')
  @dmx('check.matches', {'expression': r'^[^@\s]+@[^@\s]+\.[^@\s]+$', 'message': 'must look like an email address'})
  final String email;

  @dmx('check.length', {'min': 4, 'max': 10})
  final String postcode;

  @dmx('check.range', {'min': 1, 'max': 99})
  final int quantity;

  @dmx('check.isTrue', {'message': 'the terms must be accepted'})
  final bool acceptsTerms;

  /// Nullable fields are only checked when present: absent is not invalid.
  @dmx('check.maxLength', {'limit': 200})
  final String? giftMessage;

  @dmx('check.pattern', {'expression': r'^[A-Z0-9]{4,12}$'})
  final String? discountCode;

  //#region
  static Result<CheckoutForm, DecodeError> fromJson(Object? json, [String path = 'CheckoutForm']) =>
      switch (json) {
        {
          'email': final String email,
          'postcode': final String postcode,
          'quantity': final int quantity,
          'accepts_terms': final bool acceptsTerms,
        } =>
          switch ((
            dmxNullable<String>(dmxKey(json, 'gift_message'), '$path.gift_message', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
            dmxNullable<String>(dmxKey(json, 'discount_code'), '$path.discount_code', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final giftMessage),
              Ok(value: final discountCode),
            ) =>
              Ok(CheckoutForm(
                email: email,
                postcode: postcode,
                quantity: quantity,
                acceptsTerms: acceptsTerms,
                giftMessage: giftMessage,
                discountCode: discountCode,
              )),
            (Err(error: final e), _) => Err(e),
            (_, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'CheckoutForm', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'email': email,
        'postcode': postcode,
        'quantity': quantity,
        'accepts_terms': acceptsTerms,
        'gift_message': giftMessage,
        'discount_code': discountCode,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is CheckoutForm &&
          other.email == email &&
          other.postcode == postcode &&
          other.quantity == quantity &&
          other.acceptsTerms == acceptsTerms &&
          other.giftMessage == giftMessage &&
          other.discountCode == discountCode);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        email,
        postcode,
        quantity,
        acceptsTerms,
        giftMessage,
        discountCode,
      );

  @override
  String toString() => 'CheckoutForm(email: $email, postcode: $postcode, quantity: $quantity, acceptsTerms: $acceptsTerms, giftMessage: $giftMessage, discountCode: $discountCode)';

  CheckoutForm copyWith({
    String? email,
    String? postcode,
    int? quantity,
    bool? acceptsTerms,
    DmxPatch<String?> giftMessage = const DmxKeep(),
    DmxPatch<String?> discountCode = const DmxKeep(),
  }) =>
      CheckoutForm(
        email: email ?? this.email,
        postcode: postcode ?? this.postcode,
        quantity: quantity ?? this.quantity,
        acceptsTerms: acceptsTerms ?? this.acceptsTerms,
        giftMessage: switch (giftMessage) { DmxKeep() => this.giftMessage, DmxTo(value: final value) => value },
        discountCode: switch (discountCode) { DmxKeep() => this.discountCode, DmxTo(value: final value) => value },
      );

  /// Every rule on every field, evaluated once. Order is field order, then
  /// annotation order within a field, so the list a person sees reads down the
  /// form the way the form is laid out.
  ///
  /// Validation accumulates on purpose. A form that reports one problem, gets
  /// fixed, and then reports the next one is how you lose someone at checkout.
  List<Violation> get violations => <Violation>[
        if (email.isEmpty)
          const Violation('email', 'must not be empty'),
        if (!RegExp(r'^[^@\s]+@[^@\s]+\.[^@\s]+$').hasMatch(email))
          const Violation('email', 'must look like an email address'),
        if (postcode.length < 4)
          const Violation('postcode', 'must be at least 4 characters'),
        if (postcode.length > 10)
          const Violation('postcode', 'must be at most 10 characters'),
        if (quantity < 1)
          const Violation('quantity', 'must be at least 1'),
        if (quantity > 99)
          const Violation('quantity', 'must be at most 99'),
        if (!acceptsTerms)
          const Violation('accepts_terms', 'the terms must be accepted'),
        if (giftMessage case final String value when value.length > 200)
          const Violation('gift_message', 'must be at most 200 characters'),
        if (discountCode case final String value when !RegExp(r'^[A-Z0-9]{4,12}$').hasMatch(value))
          const Violation('discount_code', 'has the wrong format'),
      ];

  bool get isValid => violations.isEmpty;

  /// The whole value as a `Result`, so a caller can pipe it straight into
  /// whatever comes next without an `if` and without a throw.
  Result<CheckoutForm, List<Violation>> validate() => switch (violations) {
        final List<Violation> found when found.isEmpty => Ok(this),
        final List<Violation> found => Err(found),
      };

  /// The violations for one field, ready for a form control's `errorText`.
  List<String> messagesFor(String field) => <String>[
        for (final violation in violations)
          if (violation.field == field) violation.message,
      ];
  //#endregion
}

/// Validation composes with everything else. This one is a `@dmx('model')` too, so it
/// decodes from JSON *and* checks itself — the same class is the wire format
/// and the form state, which is how it should have been all along.
@dmx('model', {'fieldRename': 'snake'})
@dmx('validate')
class ShippingAddress {
  const ShippingAddress({
    required this.line1,
    required this.city,
    required this.country,
    this.line2,
  });

  @dmx('check.notEmpty')
  final String line1;

  @dmx('check.notEmpty')
  final String city;

  @dmx('check.length', {'min': 2, 'max': 2, 'message': 'must be an ISO country code'})
  final String country;

  final String? line2;

  //#region
  static Result<ShippingAddress, DecodeError> fromJson(Object? json, [String path = 'ShippingAddress']) =>
      switch (json) {
        {
          'line1': final String line1,
          'city': final String city,
          'country': final String country,
        } =>
          switch ((
            dmxNullable<String>(dmxKey(json, 'line2'), '$path.line2', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final line2),
            ) =>
              Ok(ShippingAddress(
                line1: line1,
                city: city,
                country: country,
                line2: line2,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'ShippingAddress', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'line1': line1,
        'city': city,
        'country': country,
        'line2': line2,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is ShippingAddress &&
          other.line1 == line1 &&
          other.city == city &&
          other.country == country &&
          other.line2 == line2);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        line1,
        city,
        country,
        line2,
      );

  @override
  String toString() => 'ShippingAddress(line1: $line1, city: $city, country: $country, line2: $line2)';

  ShippingAddress copyWith({
    String? line1,
    String? city,
    String? country,
    DmxPatch<String?> line2 = const DmxKeep(),
  }) =>
      ShippingAddress(
        line1: line1 ?? this.line1,
        city: city ?? this.city,
        country: country ?? this.country,
        line2: switch (line2) { DmxKeep() => this.line2, DmxTo(value: final value) => value },
      );

  /// Every rule on every field, evaluated once. Order is field order, then
  /// annotation order within a field, so the list a person sees reads down the
  /// form the way the form is laid out.
  ///
  /// Validation accumulates on purpose. A form that reports one problem, gets
  /// fixed, and then reports the next one is how you lose someone at checkout.
  List<Violation> get violations => <Violation>[
        if (line1.isEmpty)
          const Violation('line1', 'must not be empty'),
        if (city.isEmpty)
          const Violation('city', 'must not be empty'),
        if (country.length < 2)
          const Violation('country', 'must be an ISO country code'),
        if (country.length > 2)
          const Violation('country', 'must be an ISO country code'),
      ];

  bool get isValid => violations.isEmpty;

  /// The whole value as a `Result`, so a caller can pipe it straight into
  /// whatever comes next without an `if` and without a throw.
  Result<ShippingAddress, List<Violation>> validate() => switch (violations) {
        final List<Violation> found when found.isEmpty => Ok(this),
        final List<Violation> found => Err(found),
      };

  /// The violations for one field, ready for a form control's `errorText`.
  List<String> messagesFor(String field) => <String>[
        for (final violation in violations)
          if (violation.field == field) violation.message,
      ];
  //#endregion
}
