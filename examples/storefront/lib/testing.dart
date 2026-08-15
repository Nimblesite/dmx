// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('fake')` [catalogue.fake] — fixtures that are the same tomorrow.
//
// Every test suite grows a `_aCustomer()` helper with fourteen arguments, and
// every one of them drifts from the class it fakes. `@dmx('fake')` derives fixtures
// from the field list instead, so adding a field cannot leave a stale builder
// behind.
//
// There is no randomness here on purpose. The values come from the seed by
// arithmetic, so a failing test fails identically on the next run, on CI, and
// on the machine of whoever picks it up — the opposite of a fixture library
// that reaches for `Random()` and calls the flakiness "realistic data".

import 'package:dmx/dmx.dart';

import 'payments.dart';

/// Where a customer lives.
@dmx('model', {'fieldRename': 'snake'})
@dmx('fake', {'seed': 1})
class Address {
  const Address({
    required this.street,
    required this.city,
    required this.postcode,
  });

  final String street;
  final String city;
  final String postcode;

  //#region
  static Result<Address, DecodeError> fromJson(Object? json, [String path = 'Address']) =>
      switch (json) {
        {
          'street': final String street,
          'city': final String city,
          'postcode': final String postcode,
        } =>
          Ok(Address(
            street: street,
            city: city,
            postcode: postcode,
          )),
        _ => Err(DecodeError(path, 'Address', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'street': street,
        'city': city,
        'postcode': postcode,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Address &&
          other.street == street &&
          other.city == city &&
          other.postcode == postcode);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        street,
        city,
        postcode,
      );

  @override
  String toString() => 'Address(street: $street, city: $city, postcode: $postcode)';

  Address copyWith({
    String? street,
    String? city,
    String? postcode,
  }) =>
      Address(
        street: street ?? this.street,
        city: city ?? this.city,
        postcode: postcode ?? this.postcode,
      );

  /// A deterministic fixture. Every argument is optional and overrides its
  /// field, so a test states only what the test is about.
  ///
  /// There is no randomness here on purpose: the values come from the seed by
  /// arithmetic, so a failing test fails identically on the next run, on CI,
  /// and on the machine of whoever picks it up. Nullable fields default to
  /// null — the fixture is the *simplest* valid value, not the fullest.
  static Address fake({
    int seed = 1,
    String? street,
    String? city,
    String? postcode,
  }) =>
      Address(
        street: street ?? 'street-${seed + 0}',
        city: city ?? 'city-${seed + 1}',
        postcode: postcode ?? 'postcode-${seed + 2}',
      );

  /// `count` distinct fixtures — the seed walks, so no two are equal.
  static List<Address> fakes(int count, {int seed = 1}) =>
      List<Address>.generate(
        count,
        (index) => fake(seed: seed + index * 10),
      );

  /// The fixture already encoded, for exercising a decoder against something
  /// that is guaranteed to round-trip.
  static Map<String, dynamic> fakeJson({int seed = 1}) =>
      fake(seed: seed).toJson();
  //#endregion
}

/// Somebody who buys things.
@dmx('model', {'fieldRename': 'snake'})
@dmx('fake', {'seed': 42})
class Customer {
  const Customer({
    required this.id,
    required this.email,
    required this.loyaltyPoints,
    required this.joinedAt,
    required this.isVip,
    required this.preferredMethod,
    required this.address,
    required this.tags,
    this.referredBy,
  });

  final String id;
  final String email;
  final int loyaltyPoints;
  final DateTime joinedAt;
  final bool isVip;
  final PaymentMethod preferredMethod;
  final Address address;
  final List<String> tags;
  final String? referredBy;

  //#region
  static Result<Customer, DecodeError> fromJson(Object? json, [String path = 'Customer']) =>
      switch (json) {
        {
          'id': final String id,
          'email': final String email,
          'loyalty_points': final int loyaltyPoints,
          'joined_at': final String joinedAt,
          'is_vip': final bool isVip,
          'preferred_method': final Object? preferredMethod,
          'address': final Object? address,
          'tags': final List<dynamic> tags,
        } =>
          switch ((
            switch (DateTime.tryParse(joinedAt)) { final DateTime parsed => Ok<DateTime, DecodeError>(parsed), null => Err<DateTime, DecodeError>(DecodeError('$path.joined_at', 'DateTime', joinedAt)) },
            PaymentMethod.fromJson(preferredMethod, '$path.preferred_method'),
            Address.fromJson(address, '$path.address'),
            dmxList<String>(tags, '$path.tags', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
            dmxNullable<String>(dmxKey(json, 'referred_by'), '$path.referred_by', (value, path) => switch (value) {
              final String value => Ok(value),
              _ => Err(DecodeError(path, 'String', value)),
            }),
          )) {
            (
              Ok(value: final joinedAt),
              Ok(value: final preferredMethod),
              Ok(value: final address),
              Ok(value: final tags),
              Ok(value: final referredBy),
            ) =>
              Ok(Customer(
                id: id,
                email: email,
                loyaltyPoints: loyaltyPoints,
                joinedAt: joinedAt,
                isVip: isVip,
                preferredMethod: preferredMethod,
                address: address,
                tags: tags,
                referredBy: referredBy,
              )),
            (Err(error: final e), _, _, _, _) => Err(e),
            (_, Err(error: final e), _, _, _) => Err(e),
            (_, _, Err(error: final e), _, _) => Err(e),
            (_, _, _, Err(error: final e), _) => Err(e),
            (_, _, _, _, Err(error: final e)) => Err(e),
          },
        _ => Err(DecodeError(path, 'Customer', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'email': email,
        'loyalty_points': loyaltyPoints,
        'joined_at': joinedAt.toIso8601String(),
        'is_vip': isVip,
        'preferred_method': preferredMethod.toJson(),
        'address': address.toJson(),
        'tags': tags,
        'referred_by': referredBy,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Customer &&
          other.id == id &&
          other.email == email &&
          other.loyaltyPoints == loyaltyPoints &&
          other.joinedAt == joinedAt &&
          other.isVip == isVip &&
          other.preferredMethod == preferredMethod &&
          other.address == address &&
          dmxDeepEquals(other.tags, tags) &&
          other.referredBy == referredBy);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        email,
        loyaltyPoints,
        joinedAt,
        isVip,
        preferredMethod,
        address,
        dmxDeepHash(tags),
        referredBy,
      );

  @override
  String toString() => 'Customer(id: $id, email: $email, loyaltyPoints: $loyaltyPoints, joinedAt: $joinedAt, isVip: $isVip, preferredMethod: $preferredMethod, address: $address, tags: $tags, referredBy: $referredBy)';

  Customer copyWith({
    String? id,
    String? email,
    int? loyaltyPoints,
    DateTime? joinedAt,
    bool? isVip,
    PaymentMethod? preferredMethod,
    Address? address,
    List<String>? tags,
    DmxPatch<String?> referredBy = const DmxKeep(),
  }) =>
      Customer(
        id: id ?? this.id,
        email: email ?? this.email,
        loyaltyPoints: loyaltyPoints ?? this.loyaltyPoints,
        joinedAt: joinedAt ?? this.joinedAt,
        isVip: isVip ?? this.isVip,
        preferredMethod: preferredMethod ?? this.preferredMethod,
        address: address ?? this.address,
        tags: tags ?? this.tags,
        referredBy: switch (referredBy) { DmxKeep() => this.referredBy, DmxTo(value: final value) => value },
      );

  /// A deterministic fixture. Every argument is optional and overrides its
  /// field, so a test states only what the test is about.
  ///
  /// There is no randomness here on purpose: the values come from the seed by
  /// arithmetic, so a failing test fails identically on the next run, on CI,
  /// and on the machine of whoever picks it up. Nullable fields default to
  /// null — the fixture is the *simplest* valid value, not the fullest.
  static Customer fake({
    int seed = 42,
    String? id,
    String? email,
    int? loyaltyPoints,
    DateTime? joinedAt,
    bool? isVip,
    PaymentMethod? preferredMethod,
    Address? address,
    List<String>? tags,
    String? referredBy,
  }) =>
      Customer(
        id: id ?? 'id-${seed + 0}',
        email: email ?? 'email-${seed + 1}@example.test',
        loyaltyPoints: loyaltyPoints ?? (seed + 2),
        joinedAt: joinedAt ?? DateTime.utc(2024).add(Duration(days: (seed + 3) % 365)),
        isVip: isVip ?? (seed + 4).isEven,
        preferredMethod: preferredMethod ?? PaymentMethod.fake(seed: seed + 5),
        address: address ?? Address.fake(seed: seed + 6),
        tags: tags ?? <String>['tags-${seed + 7}'],
        referredBy: referredBy,
      );

  /// `count` distinct fixtures — the seed walks, so no two are equal.
  static List<Customer> fakes(int count, {int seed = 42}) =>
      List<Customer>.generate(
        count,
        (index) => fake(seed: seed + index * 10),
      );

  /// The fixture already encoded, for exercising a decoder against something
  /// that is guaranteed to round-trip.
  static Map<String, dynamic> fakeJson({int seed = 42}) =>
      fake(seed: seed).toJson();
  //#endregion
}
