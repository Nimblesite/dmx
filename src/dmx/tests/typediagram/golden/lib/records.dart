// dmx: generated from docs/records.dmx.md — do not edit.
// dmx: group 1, fences 1/2, definition 564eca654d0cbefc, template ebcc1789a3d0fa99, context v1, dmx 0.0.0.

// Generated from docs/records.dmx.md. Edit the diagram, not this file.

/// User — a record from the diagram.
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
}

/// Pair — a record from the diagram.
final class Pair<A, B> {
  /// Every field, in the order the diagram declares them.
  const Pair({required this.first, required this.second});

  /// The `first` field, declared as `A`.
  final A first;

  /// The `second` field, declared as `B`.
  final B second;
}

/// Box — a record from the diagram.
final class Box<T> {
  /// Every field, in the order the diagram declares them.
  const Box({required this.value});

  /// The `value` field, declared as `T`.
  final T value;
}

/// Empty — a record from the diagram.
final class Empty {
  /// Every field, in the order the diagram declares them.
  const Empty();
}

/// Separators — a record from the diagram.
final class Separators {
  /// Every field, in the order the diagram declares them.
  const Separators({required this.a, required this.b, required this.c});

  /// The `a` field, declared as `Int`.
  final int a;

  /// The `b` field, declared as `Int`.
  final int b;

  /// The `c` field, declared as `Int`.
  final int c;
}

/// Email — a record from the diagram.
final class Email {
  /// Every field, in the order the diagram declares them.
  const Email({required this.text});

  /// The `text` field, declared as `String`.
  final String text;
}

/// Role — a record from the diagram.
final class Role {
  /// Every field, in the order the diagram declares them.
  const Role({required this.name});

  /// The `name` field, declared as `String`.
  final String name;
}

/// Address — a record from the diagram.
final class Address {
  /// Every field, in the order the diagram declares them.
  const Address({required this.line});

  /// The `line` field, declared as `String`.
  final String line;
}
