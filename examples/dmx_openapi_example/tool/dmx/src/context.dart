/// The models the Mustache templates are rendered against.
///
/// This is the whole point of the split. Everything hard — reading OpenAPI,
/// resolving `$ref`s, choosing Dart types, deciding nullability, naming
/// classes the document never named, working out how a path is interpolated —
/// happens here, in Dart, where it can be tested. What comes out is plain
/// JSON: strings, booleans, and lists of maps.
///
/// The templates then decide *shape*. They contain no logic beyond
/// `{{#section}}`, which is what makes them a thing a team can edit without
/// reading a line of this file [dartmacros.render].
library;

import 'package:dmx/macros.dart';

import 'dart_types.dart';
import 'document.dart';

/// The model for one generated data class.
Map<String, Object?> modelContext(NamedSchema schema) {
  final className = dmxPascalCase(schema.name);
  final properties = [
    for (final MapEntry(key: wireName, value: node)
        in schema.node.properties.entries)
      _property(className, wireName, node, schema.node.required),
  ];
  return {
    'className': className,
    'schemaName': schema.name,
    'isSynthesized': schema.isSynthesized,
    'imports': [
      for (final file in _referencedFiles(schema)) {'file': file},
    ],
    'docLines': _doc(
      schema.node.description ??
          (schema.isSynthesized
              ? 'An object the document declares inline, named after where it '
                  'was found.'
              : 'The `${schema.name}` schema.'),
    ),
    'properties': properties,
    'hasProperties': properties.isNotEmpty,
    'propertyNames': _literalList([
      for (final property in properties) property['wireName'],
    ]),
    // The decoded results, named once, for the arm that reports the first
    // failure. Joined here so the template never emits a trailing comma.
    'resultNames': [
      for (final property in properties) property['dartName'],
    ].join(', '),
  };
}

/// The generated files `schema` has to import, in name order.
///
/// A class that names another class in a field type or a decoder needs that
/// class's file. Working this out from the schema — rather than importing the
/// barrel, or every sibling — is what keeps the generated tree free of import
/// cycles and of imports nothing uses.
List<String> _referencedFiles(NamedSchema schema) {
  final files = <String>{};
  final className = dmxPascalCase(schema.name);
  for (final MapEntry(key: wireName, value: node)
      in schema.node.properties.entries) {
    for (final referenced in _referenced(className, wireName, node)) {
      if (referenced != schema.name) {
        files.add(fileNameFor(referenced));
      }
    }
  }
  return files.toList()..sort();
}

/// Every schema `node` names, following arrays into their elements.
List<String> _referenced(String owner, String wireName, SchemaNode node) {
  final ref = node.ref;
  if (ref != null) {
    return [ref];
  }
  if (node.isInlineObject) {
    return [synthesizedName(owner, wireName)];
  }
  final items = node.items;
  return items == null ? const [] : _referenced(owner, wireName, items);
}

/// One property, as both a field and a decode.
Map<String, Object?> _property(
  String owner,
  String wireName,
  SchemaNode node,
  List<String> required,
) {
  final nullable = node.isNullable || !required.contains(wireName);
  final type = dartTypeOf(node, owner: synthesizedName(owner, wireName));
  final dartName = dmxCamelCase(wireName);
  return {
    'wireName': wireName,
    'dartName': dartName,
    'type': nullable ? '${type.name}?' : type.name,
    'nullable': nullable,
    'docLines': _doc(node.description ?? '`$wireName` from the document.'),
    'decodeExpr': decodeExpression(
      type,
      wireName: wireName,
      nullable: nullable,
    ),
    'hasEnum': node.enumeration.isNotEmpty,
    'enumConstant': '${dartName}Values',
    'enumValues': _literalList(node.enumeration),
  };
}

/// The model for the generated client.
Map<String, Object?> clientContext(Document document, String className) => {
      'className': className,
      'docLines': _doc(
        document.description ?? 'A client for ${document.title}.',
      ),
      // Prose and source are different jobs: `title` reads inside a doc
      // comment, `baseUrl` has to survive as a Dart literal. Mixing the two up
      // is how a generator emits a file that will not parse.
      'title': document.title,
      'apiVersion': document.version,
      'apiVersionLiteral': _literal(document.version),
      'baseUrl': _literal(document.server),
      'operationIds': _literalList([
        for (final operation in document.operations) operation.id,
      ]),
      'methods': [
        for (final operation in document.operations) _method(operation),
      ],
    };

/// One operation, as a Dart method.
Map<String, Object?> _method(Operation operation) {
  final success = operation.success;
  final type = success == null
      ? const DartType('void', 'dmxAny')
      : dartTypeOf(success, owner: dmxPascalCase(operation.id));
  final parameters = [
    // Path parameters are required and come first; a caller cannot omit one
    // and still have a URL.
    for (final parameter in operation.pathParameters)
      _parameter(parameter, isRequired: true),
    for (final parameter in operation.queryParameters)
      _parameter(parameter, isRequired: parameter.isRequired),
  ];
  return {
    'name': dmxCamelCase(operation.id),
    'docLines': _doc(operation.summary ?? 'Calls `${operation.path}`.'),
    'httpMethod': operation.method,
    'path': operation.path,
    'returnType': type.name,
    'decoder': type.decoder,
    'returnsValue': success != null,
    'pathExpr': _pathExpression(operation),
    'parameters': parameters,
    'hasParameters': parameters.isNotEmpty,
    'hasQuery': operation.queryParameters.isNotEmpty,
    'queryParameters': [
      for (final parameter in operation.queryParameters)
        _parameter(parameter, isRequired: parameter.isRequired),
    ],
  };
}

/// One parameter, as a named argument of the generated method.
Map<String, Object?> _parameter(Parameter parameter,
    {required bool isRequired}) {
  final type = dartParameterTypeOf(parameter.schema);
  final dartName = dmxCamelCase(parameter.name);
  final described = parameter.description ??
      'The `${parameter.name}` ${parameter.location} parameter.';
  final allowed = parameter.schema.enumeration.isEmpty
      ? ''
      : ' Allowed: `${parameter.schema.enumeration.join('`, `')}`.';
  return {
    'dartName': dartName,
    'wireName': parameter.name,
    'wireExpr': wireExpression(type, dartName),
    'type': isRequired ? type.name : '${type.name}?',
    'isRequired': isRequired,
    // The whole doc line, prefix included, is assembled here so the template
    // never has to wrap prose — it emits the lines it is handed.
    'docLines': _doc('[$dartName] — $described$allowed'),
  };
}

/// The templated path as a Dart string expression.
///
/// `/rate/{base}/{quote}` becomes an interpolation over the method's own
/// parameters, each percent-encoded — a currency code is safe, but a path
/// parameter is caller input and the generator does not get to assume.
String _pathExpression(Operation operation) {
  final buffer = StringBuffer("'");
  final segments = operation.path.split('/');
  for (var index = 0; index < segments.length; index++) {
    if (index > 0) {
      buffer.write('/');
    }
    final segment = segments[index];
    final name = segment.startsWith('{') && segment.endsWith('}')
        ? segment.substring(1, segment.length - 1)
        : null;
    if (name == null) {
      buffer.write(segment);
      continue;
    }
    final declared = operation.pathParameters
        .where((parameter) => parameter.name == name)
        .firstOrNull;
    final wire = declared == null
        ? dmxCamelCase(name)
        : wireExpression(
            dartParameterTypeOf(declared.schema),
            dmxCamelCase(name),
          );
    buffer.write('\${Uri.encodeComponent($wire)}');
  }
  buffer.write("'");
  return buffer.toString();
}

/// The model for the barrel that exports every generated file.
Map<String, Object?> barrelContext(List<String> fileNames, String title) => {
      'title': title,
      'exports': [
        for (final name in fileNames) {'file': name},
      ],
    };

/// The manifest the annotated class itself receives.
Map<String, Object?> manifestContext(
  Document document,
  Map<String, String> files,
  String indent,
) =>
    {
      'indent': indent,
      'title': _literal(document.title),
      'apiVersion': _literal(document.version),
      'baseUrl': _literal(document.server),
      'operationIds': _literalList([
        for (final operation in document.operations) operation.id,
      ]),
      'schemaNames': _literalList([
        for (final schema in document.schemas) schema.name,
      ]),
      'files': [
        for (final MapEntry(:key, :value) in files.entries)
          {'schema': _literal(key), 'file': _literal(value)},
      ],
    };

/// Prose split into the lines a doc comment carries, wrapped so generated
/// files stay readable at the width the rest of the repo is written to.
List<Map<String, Object?>> _doc(String text, {int width = 68}) {
  final lines = <String>[];
  final current = StringBuffer();
  for (final word in text.replaceAll('\n', ' ').split(' ')) {
    if (word.isEmpty) {
      continue;
    }
    if (current.isNotEmpty && current.length + word.length + 1 > width) {
      lines.add(current.toString());
      current.clear();
    }
    if (current.isNotEmpty) {
      current.write(' ');
    }
    current.write(word);
  }
  if (current.isNotEmpty) {
    lines.add(current.toString());
  }
  return [
    for (final line in lines) {'line': line},
  ];
}

/// A Dart single-quoted string literal, escaped [hygiene].
///
/// Every string the generator emits goes through here. A currency name or an
/// API description is document text, and document text that reached generated
/// source unescaped is how a generator emits Dart that will not parse.
String _literal(String text) {
  final buffer = StringBuffer("'");
  for (final character in text.split('')) {
    buffer.write(switch (character) {
      "'" => "\\'",
      r'\' => r'\\',
      r'$' => r'\$',
      '\n' => r'\n',
      '\r' => r'\r',
      _ => character,
    });
  }
  buffer.write("'");
  return buffer.toString();
}

/// A Dart list literal of strings, `['a', 'b']`.
String _literalList(List<Object?> values) => [
      '[',
      [
        for (final value in values)
          if (value is String) _literal(value),
      ].join(', '),
      ']',
    ].join();
