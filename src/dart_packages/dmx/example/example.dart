// Run with: dart run example/example.dart
//
// The `//#region` block below is generated. It appeared in this file when the
// file was saved with the dmx watcher running — there is no `part` directive,
// no `.g.dart`, and no build step of your own.
import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({required this.id, required this.name, this.email});

  final String id;
  final String name;
  final String? email;

  //#region
  static Result<User, DecodeError> fromJson(Object? json,
          [String path = 'User']) =>
      switch (json) {
        {
          'id': final String id,
          'name': final String name,
        } =>
          switch ((
            dmxNullable<String>(
                dmxKey(json, 'email'),
                '$path.email',
                (value, path) => switch (value) {
                      final String value => Ok(value),
                      _ => Err(DecodeError(path, 'String', value)),
                    }),
          )) {
            (Ok(value: final email),) => Ok(User(
                id: id,
                name: name,
                email: email,
              )),
            (Err(error: final e),) => Err(e),
          },
        _ => Err(DecodeError(path, 'User', json)),
      };

  Map<String, dynamic> toJson() => <String, dynamic>{
        'id': id,
        'name': name,
        'email': email,
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is User &&
          other.id == id &&
          other.name == name &&
          other.email == email);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        id,
        name,
        email,
      );

  @override
  String toString() => 'User(id: $id, name: $name, email: $email)';

  User copyWith({
    String? id,
    String? name,
    DmxPatch<String?> email = const DmxKeep(),
  }) =>
      User(
        id: id ?? this.id,
        name: name ?? this.name,
        email: switch (email) {
          DmxKeep() => this.email,
          DmxTo(value: final value) => value
        },
      );
  //#endregion
}

void main() {
  // Decoding returns a value, so bad input is a branch, not a crash.
  switch (User.fromJson(<String, dynamic>{'id': 'u_1', 'name': 'Ada'})) {
    case Ok(value: final user):
      print(user);
      // `copyWith()` keeps a field; `DmxTo(null)` clears it.
      print(user.copyWith(
          name: 'Ada Lovelace', email: const DmxTo('ada@example.com')));
      print(user.toJson());
    case Err(error: final error):
      print('${error.path}: expected ${error.expected}');
  }

  // A malformed payload names the field that failed, and where.
  switch (User.fromJson(<String, dynamic>{'id': 'u_2', 'name': 42})) {
    case Ok(value: final user):
      print(user);
    case Err(error: final error):
      // User: expected User, got {id: u_2, name: 42} (_Map<String, dynamic>)
      print(error);
  }
}
