// dmx: generated from models/records.td — do not edit.
// dmx: rendered through the canonical model template, definition 564eca654d0cbefc, template 5fba7c04728545cb, context v1, dmx 0.0.0.

// Generated from models/records.td. Edit the definition, not this file.

import 'package:dmx/dmx.dart' as dmx;

/// User — an immutable value from the diagram.
final class User {
  /// Every field, in the order the diagram declares them.
  const User({required this.id, required this.name, this.email, required this.roles, required this.address});

  /// The `id` field, declared as `Uuid`.
  final String id;

  /// The `name` field, declared as `String`.
  final String name;

  /// The `email` field, declared as `Option<Email>`.
  final Email? email;

  /// The `roles` field, declared as `List<Role>`.
  final List<Role> roles;

  /// The `address` field, declared as `Address`.
  final Address address;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is User &&
          other.id == id &&
          other.name == name &&
          other.email == email &&
          dmx.dmxDeepEquals(other.roles, roles) &&
          other.address == address);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        name,
        email,
        dmx.dmxDeepHash(roles),
        address,
      );

  @override
  String toString() => 'User(id: $id, name: $name, email: $email, roles: $roles, address: $address)';

  /// A copy of this value with the named fields replaced.
  User copyWith({
    String? id,
    String? name,
    dmx.DmxPatch<Email?> email = const dmx.DmxKeep(),
    List<Role>? roles,
    Address? address,
  }) =>
      User(
        id: id ?? this.id,
        name: name ?? this.name,
        email: switch (email) { dmx.DmxKeep() => this.email, dmx.DmxTo(value: final value) => value },
        roles: roles ?? this.roles,
        address: address ?? this.address,
      );
}

/// JSON for [User].
extension UserJson on User {
  /// Decodes a `User` from a JSON value, or says why it could not.
  static dmx.Result<User, dmx.DecodeError> fromJson(Object? json, [String path = 'User']) =>
      switch (json) {
        {
          'id': final String id,
          'name': final String name,
          'roles': final List<dynamic> roles,
          'address': final Object? address,
        } =>
          switch ((
            dmx.dmxNullable<Email>(dmx.dmxKey(json, 'email'), '$path.email', EmailJson.fromJson),
            dmx.dmxList<Role>(roles, '$path.roles', RoleJson.fromJson),
            AddressJson.fromJson(address, '$path.address'),
          )) {
            (
              dmx.Ok(value: final email),
              dmx.Ok(value: final roles),
              dmx.Ok(value: final address),
            ) =>
              dmx.Ok(User(
                id: id,
                name: name,
                email: email,
                roles: roles,
                address: address,
              )),
            (dmx.Err(error: final e), _, _) => dmx.Err(e),
            (_, dmx.Err(error: final e), _) => dmx.Err(e),
            (_, _, dmx.Err(error: final e)) => dmx.Err(e),
          },
        _ => dmx.Err(dmx.DecodeError(path, 'User', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'id': id,
        'name': name,
        'email': email?.toJson(),
        'roles': roles.map((e0) => e0.toJson()).toList(),
        'address': address.toJson(),
      };
}

/// Pair — an immutable value from the diagram.
final class Pair<A, B> {
  /// Every field, in the order the diagram declares them.
  const Pair({required this.first, required this.second});

  /// The `first` field, declared as `A`.
  final A first;

  /// The `second` field, declared as `B`.
  final B second;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Pair<A, B> &&
          other.first == first &&
          other.second == second);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        first,
        second,
      );

  @override
  String toString() => 'Pair(first: $first, second: $second)';

  /// A copy of this value with the named fields replaced.
  Pair<A, B> copyWith({
    A? first,
    B? second,
  }) =>
      Pair(
        first: first ?? this.first,
        second: second ?? this.second,
      );
}

/// Box — an immutable value from the diagram.
final class Box<T> {
  /// Every field, in the order the diagram declares them.
  const Box({required this.value});

  /// The `value` field, declared as `T`.
  final T value;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Box<T> &&
          other.value == value);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        value,
      );

  @override
  String toString() => 'Box(value: $value)';

  /// A copy of this value with the named fields replaced.
  Box<T> copyWith({
    T? value,
  }) =>
      Box(
        value: value ?? this.value,
      );
}

/// Empty — an immutable value from the diagram.
final class Empty {
  /// Every field, in the order the diagram declares them.
  const Empty();

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Empty);

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() => 'Empty()';
}

/// JSON for [Empty].
extension EmptyJson on Empty {
  /// Decodes a `Empty` from a JSON value, or says why it could not.
  static dmx.Result<Empty, dmx.DecodeError> fromJson(Object? json, [String path = 'Empty']) =>
      switch (json) {
        Map<String, Object?>() =>
          dmx.Ok(Empty(
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Empty', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
      };
}

/// Separators — an immutable value from the diagram.
final class Separators {
  /// Every field, in the order the diagram declares them.
  const Separators({required this.a, required this.b, required this.c});

  /// The `a` field, declared as `Int`.
  final int a;

  /// The `b` field, declared as `Int`.
  final int b;

  /// The `c` field, declared as `Int`.
  final int c;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Separators &&
          other.a == a &&
          other.b == b &&
          other.c == c);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        a,
        b,
        c,
      );

  @override
  String toString() => 'Separators(a: $a, b: $b, c: $c)';

  /// A copy of this value with the named fields replaced.
  Separators copyWith({
    int? a,
    int? b,
    int? c,
  }) =>
      Separators(
        a: a ?? this.a,
        b: b ?? this.b,
        c: c ?? this.c,
      );
}

/// JSON for [Separators].
extension SeparatorsJson on Separators {
  /// Decodes a `Separators` from a JSON value, or says why it could not.
  static dmx.Result<Separators, dmx.DecodeError> fromJson(Object? json, [String path = 'Separators']) =>
      switch (json) {
        {
          'a': final int a,
          'b': final int b,
          'c': final int c,
        } =>
          dmx.Ok(Separators(
            a: a,
            b: b,
            c: c,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Separators', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'a': a,
        'b': b,
        'c': c,
      };
}

/// Email — an immutable value from the diagram.
final class Email {
  /// Every field, in the order the diagram declares them.
  const Email({required this.text});

  /// The `text` field, declared as `String`.
  final String text;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Email &&
          other.text == text);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        text,
      );

  @override
  String toString() => 'Email(text: $text)';

  /// A copy of this value with the named fields replaced.
  Email copyWith({
    String? text,
  }) =>
      Email(
        text: text ?? this.text,
      );
}

/// JSON for [Email].
extension EmailJson on Email {
  /// Decodes a `Email` from a JSON value, or says why it could not.
  static dmx.Result<Email, dmx.DecodeError> fromJson(Object? json, [String path = 'Email']) =>
      switch (json) {
        {
          'text': final String text,
        } =>
          dmx.Ok(Email(
            text: text,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Email', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'text': text,
      };
}

/// Role — an immutable value from the diagram.
final class Role {
  /// Every field, in the order the diagram declares them.
  const Role({required this.name});

  /// The `name` field, declared as `String`.
  final String name;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Role &&
          other.name == name);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        name,
      );

  @override
  String toString() => 'Role(name: $name)';

  /// A copy of this value with the named fields replaced.
  Role copyWith({
    String? name,
  }) =>
      Role(
        name: name ?? this.name,
      );
}

/// JSON for [Role].
extension RoleJson on Role {
  /// Decodes a `Role` from a JSON value, or says why it could not.
  static dmx.Result<Role, dmx.DecodeError> fromJson(Object? json, [String path = 'Role']) =>
      switch (json) {
        {
          'name': final String name,
        } =>
          dmx.Ok(Role(
            name: name,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Role', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'name': name,
      };
}

/// Address — an immutable value from the diagram.
final class Address {
  /// Every field, in the order the diagram declares them.
  const Address({required this.line});

  /// The `line` field, declared as `String`.
  final String line;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Address &&
          other.line == line);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        line,
      );

  @override
  String toString() => 'Address(line: $line)';

  /// A copy of this value with the named fields replaced.
  Address copyWith({
    String? line,
  }) =>
      Address(
        line: line ?? this.line,
      );
}

/// JSON for [Address].
extension AddressJson on Address {
  /// Decodes a `Address` from a JSON value, or says why it could not.
  static dmx.Result<Address, dmx.DecodeError> fromJson(Object? json, [String path = 'Address']) =>
      switch (json) {
        {
          'line': final String line,
        } =>
          dmx.Ok(Address(
            line: line,
          )),
        _ => dmx.Err(dmx.DecodeError(path, 'Address', json)),
      };

  /// This value as a JSON map.
  Map<String, Object?> toJson() => <String, Object?>{
        'line': line,
      };
}
