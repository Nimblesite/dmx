/// What a schema node becomes in Dart, and how it is decoded.
///
/// Two answers per node, always together: the type as it is written in the
/// generated source, and the `DmxDecode` that produces one. Keeping them in
/// one place is what stops a field being declared `DateTime` and decoded with
/// `dmxString` — the failure this file exists to make impossible.
library;

import 'package:dmx/macros.dart';

import 'document.dart';

/// A Dart type and the decoder that yields it.
final class DartType {
  /// The type as written, `List<Rate>`.
  final String name;

  /// A `DmxDecode<T>` expression: a function name, a tear-off, or a
  /// combinator applied to one — never a call.
  final String decoder;

  /// Builds a mapping.
  const DartType(this.name, this.decoder);
}

/// The Dart mapping of `node`, owned by the class named `owner`.
///
/// `owner` is what an inline object is named after: the shape inside
/// `Rate.providers[]` has no name in the document, so the generator gives it
/// one from where it was found [dartmacros.files].
DartType dartTypeOf(SchemaNode node, {required String owner}) {
  final ref = node.ref;
  if (ref != null) {
    final className = dmxPascalCase(ref);
    return DartType(className, '$className.fromJson');
  }
  if (node.isInlineObject) {
    return DartType(owner, '$owner.fromJson');
  }
  final items = node.items;
  if (node.type == 'array' && items != null) {
    final element = dartTypeOf(items, owner: owner);
    return DartType('List<${element.name}>', 'dmxListOf(${element.decoder})');
  }
  return switch ((node.type, node.format)) {
    ('string', 'date') => const DartType('DateTime', 'dmxDateTime'),
    ('string', 'date-time') => const DartType('DateTime', 'dmxDateTime'),
    ('string', 'uri') => const DartType('Uri', 'dmxUri'),
    ('string', _) => const DartType('String', 'dmxString'),
    ('number', _) => const DartType('double', 'dmxDouble'),
    ('integer', _) => const DartType('int', 'dmxInt'),
    ('boolean', _) => const DartType('bool', 'dmxBool'),
    ('object', _) => const DartType('Map<String, Object?>', 'dmxMapOf(dmxAny)'),
    _ => const DartType('Object?', 'dmxAny'),
  };
}

/// The Dart type of a path or query parameter, and how it reaches the wire.
///
/// A parameter is not a field. It goes into a URL, where everything is text,
/// so `format: date` — which types a *response* property as `DateTime` — types
/// a parameter as the `String` the document says it is. Inventing a date
/// format here would be the generator guessing at what the server parses; the
/// document constrains the shape and says nothing about a Dart type.
DartType dartParameterTypeOf(SchemaNode node) => switch (node.type) {
      'integer' => const DartType('int', 'dmxInt'),
      'number' => const DartType('double', 'dmxDouble'),
      'boolean' => const DartType('bool', 'dmxBool'),
      _ => const DartType('String', 'dmxString'),
    };

/// How a parameter of `type` bound to `dartName` is written into a URL.
///
/// Already text stays as it is; anything else is asked for its text, inside
/// the null check that guards an optional parameter.
String wireExpression(DartType type, String dartName) =>
    type.name == 'String' ? dartName : '$dartName.toString()';

/// The expression that decodes property `wireName` out of a JSON object bound
/// to `json`, at `path`.
///
/// A required property is decoded straight; an optional or nullable one goes
/// through `dmxNullable`, so an absent key and a present null read alike — the
/// distinction an API that omits empty fields would otherwise force on every
/// caller.
String decodeExpression(
  DartType type, {
  required String wireName,
  required bool nullable,
}) {
  final read = "json['$wireName']";
  final path = "'\$path.$wireName'";
  return nullable
      ? 'dmxNullable($read, $path, ${type.decoder})'
      : '${type.decoder}($read, $path)';
}
