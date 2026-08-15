/// The OpenAPI document, read into the shapes this macro generates from.
///
/// A real parser over real JSON — `dart:convert`, never a regex — because the
/// document is structured data and the generated Dart is only ever as correct
/// as the reading of it.
///
/// Deliberately narrow: this understands the slice of OpenAPI 3.1 the
/// Frankfurter document actually uses, and says so when it meets anything
/// else. A macro that guessed at a construct it did not understand would emit
/// Dart that compiles and lies.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';

/// A property of an object schema, or an item of an array.
final class SchemaNode {
  /// The JSON type names this node allows, `null` included when nullable.
  final List<String> types;

  /// `date`, `uri`, and friends — OpenAPI's refinement of a string.
  final String? format;

  /// The component schema this node points at, when it is a `$ref`.
  final String? ref;

  /// What an array's elements are.
  final SchemaNode? items;

  /// An inline object's own properties, in document order.
  final Map<String, SchemaNode> properties;

  /// Which of [properties] the document marks required.
  final List<String> required;

  /// The prose the document carries, for the generated doc comment.
  final String? description;

  /// The values an enumerated node allows, `null` entries dropped.
  final List<String> enumeration;

  /// Builds a node.
  const SchemaNode({
    this.types = const [],
    this.format,
    this.ref,
    this.items,
    this.properties = const {},
    this.required = const [],
    this.description,
    this.enumeration = const [],
  });

  /// Whether the document says this may be null.
  bool get isNullable => types.contains('null');

  /// The one non-null type name, when there is exactly one.
  String? get type {
    final named = [
      for (final name in types)
        if (name != 'null') name,
    ];
    return named.length == 1 ? named.first : null;
  }

  /// Whether this is an object that declares its own properties — the case
  /// that earns a synthesized Dart class.
  bool get isInlineObject => ref == null && properties.isNotEmpty;
}

/// One named schema under `components/schemas`, or one this macro synthesized
/// for an inline object.
final class NamedSchema {
  /// The name the generated Dart class takes, before casing.
  final String name;

  /// The schema itself.
  final SchemaNode node;

  /// Whether the document named this, or the macro did.
  final bool isSynthesized;

  /// Builds a named schema.
  const NamedSchema(this.name, this.node, {this.isSynthesized = false});
}

/// One parameter an operation accepts.
final class Parameter {
  /// The name on the wire.
  final String name;

  /// `path` or `query`.
  final String location;

  /// Whether the document marks it required.
  final bool isRequired;

  /// Its schema.
  final SchemaNode schema;

  /// The prose the document carries.
  final String? description;

  /// Builds a parameter.
  const Parameter(
    this.name, {
    required this.location,
    required this.isRequired,
    required this.schema,
    this.description,
  });
}

/// One operation: a method, a path, and what it takes and returns.
final class Operation {
  /// The `operationId` the document declares.
  final String id;

  /// `get`, `post`, and so on, upper-cased for the wire.
  final String method;

  /// The templated path, `/rate/{base}/{quote}`.
  final String path;

  /// One-line prose from the document.
  final String? summary;

  /// Every parameter, `$ref`s already resolved, in document order.
  final List<Parameter> parameters;

  /// What a 200 carries, or null when it carries nothing this macro maps.
  final SchemaNode? success;

  /// Builds an operation.
  const Operation(
    this.id, {
    required this.method,
    required this.path,
    required this.parameters,
    this.summary,
    this.success,
  });

  /// The path parameters, in the order the path mentions them.
  List<Parameter> get pathParameters => [
        for (final parameter in parameters)
          if (parameter.location == 'path') parameter,
      ];

  /// The query parameters, in document order.
  List<Parameter> get queryParameters => [
        for (final parameter in parameters)
          if (parameter.location == 'query') parameter,
      ];
}

/// A whole OpenAPI document, as much of it as this macro reads.
final class Document {
  /// `info.title`.
  final String title;

  /// `info.version` — the API's version, not this macro's.
  final String version;

  /// `info.description`, when the document carries one.
  final String? description;

  /// The first `servers[].url`, which becomes the client's base URL.
  final String server;

  /// Every schema a Dart class is generated for: the document's own, then the
  /// ones synthesized for inline objects, in a stable order.
  final List<NamedSchema> schemas;

  /// Every operation, in document order.
  final List<Operation> operations;

  /// Builds a document.
  const Document({
    required this.title,
    required this.version,
    required this.server,
    required this.schemas,
    required this.operations,
    this.description,
  });
}

/// Reads `source` — the bytes of an OpenAPI document — into a [Document].
///
/// Every failure is a value: an author who points the macro at the wrong file,
/// or at a document using a construct this macro does not read, gets a
/// diagnostic naming the place rather than a stack trace.
Result<Document, DmxRefusal> readDocument(Object? source) {
  if (source is! Map<String, Object?>) {
    return const Err(
      DmxRefusal('DMX3920', 'the OpenAPI document is not a JSON object.'),
    );
  }
  final info = _object(source['info']);
  final servers = source['servers'];
  final server = switch (servers) {
    [final Map<String, Object?> first, ...] => _string(first['url']),
    _ => null,
  };
  if (server == null) {
    return const Err(
      DmxRefusal(
        'DMX3921',
        'the OpenAPI document declares no `servers[0].url`, so the generated '
            'client would have no base URL to call.',
      ),
    );
  }
  final components = _object(source['components']);
  final schemas = _object(components['schemas']);
  final resolver = _Resolver(
    parameters: _object(components['parameters']),
    responses: _object(components['responses']),
  );
  final named = <NamedSchema>[
    for (final MapEntry(:key, :value) in schemas.entries)
      NamedSchema(key, resolver.node(_object(value))),
  ];
  final operations = resolver.operations(_object(source['paths']));
  if (operations.isEmpty) {
    return const Err(
      DmxRefusal(
        'DMX3922',
        'the OpenAPI document declares no operations with an `operationId`. '
            'This macro names its methods after that id rather than inventing '
            'one from the path.',
      ),
    );
  }
  return Ok(
    Document(
      title: _string(info['title']) ?? 'API',
      version: _string(info['version']) ?? '0.0.0',
      description: _string(info['description']),
      server: server,
      schemas: [...named, ..._synthesized(named)],
      operations: operations,
    ),
  );
}

/// The classes this macro invents for object schemas written inline.
///
/// The document names `Rate`; it does not name the shape inside
/// `Rate.providers[]`. That shape still needs a Dart class, and the only name
/// available is the one the macro builds from where it was found —
/// `RateProvider` — so the naming rule is part of the generator, not the
/// document.
List<NamedSchema> _synthesized(List<NamedSchema> named) {
  final out = <NamedSchema>[];
  for (final schema in named) {
    _collect(schema.name, schema.node, out);
  }
  return out;
}

/// The file one schema's class lives in: `CurrencyDetail` is in
/// `currency_detail.dart` [dartmacros.files].
///
/// The one place this rule lives. The macro names the files it authors with
/// it, and a model that references another names its import with it — two
/// callers that must agree or the generated tree does not compile.
String fileNameFor(String schemaName) => '${dmxSnakeCase(schemaName)}.dart';

/// The class an inline object gets, named after where it was found:
/// `Rate.providers[]` becomes `RateProvider`.
///
/// The one place this rule lives. The schema walk names the classes and the
/// type mapper names the decoders, and a second copy of the rule is how those
/// two come to disagree about the same shape.
String synthesizedName(String owner, String property) =>
    '$owner${dmxPascalCase(_singular(property))}';

/// Walks `node` for inline objects, naming each after the path that reached it.
void _collect(String prefix, SchemaNode node, List<NamedSchema> out) {
  for (final MapEntry(:key, :value) in node.properties.entries) {
    final name = synthesizedName(prefix, key);
    if (value.isInlineObject) {
      out.add(NamedSchema(name, value, isSynthesized: true));
      _collect(name, value, out);
    }
    final items = value.items;
    if (items != null && items.isInlineObject) {
      out.add(NamedSchema(name, items, isSynthesized: true));
      _collect(name, items, out);
    }
  }
}

/// `providers` names one `Provider`. The rule stays deliberately small — it
/// runs on property names read out of a document, never on guesses.
String _singular(String name) => name.endsWith('ies')
    ? '${name.substring(0, name.length - 3)}y'
    : name.endsWith('s') && !name.endsWith('ss')
        ? name.substring(0, name.length - 1)
        : name;

/// Resolves the `$ref`s a document uses into the values behind them.
final class _Resolver {
  /// `components/parameters`.
  final Map<String, Object?> parameters;

  /// `components/responses`.
  final Map<String, Object?> responses;

  /// Builds a resolver over one document's components.
  const _Resolver({required this.parameters, required this.responses});

  /// Every operation in `paths` that declares an `operationId`.
  List<Operation> operations(Map<String, Object?> paths) {
    const methods = ['get', 'post', 'put', 'patch', 'delete'];
    return [
      for (final MapEntry(key: path, value: item) in paths.entries)
        for (final MapEntry(key: method, value: raw) in _object(item).entries)
          if (methods.contains(method))
            if (_operation(path, method, _object(raw)) case final Operation op)
              op,
    ];
  }

  /// One operation, or null when it has no `operationId` to be named after.
  Operation? _operation(String path, String method, Map<String, Object?> raw) {
    final id = _string(raw['operationId']);
    if (id == null) {
      return null;
    }
    final declared = raw['parameters'];
    return Operation(
      id,
      method: method.toUpperCase(),
      path: path,
      summary: _string(raw['summary']),
      parameters: [
        if (declared is List<Object?>)
          for (final entry in declared)
            if (_parameter(_object(entry)) case final Parameter parameter)
              parameter,
      ],
      success: _success(_object(raw['responses'])),
    );
  }

  /// One parameter, following a `$ref` into `components/parameters` first.
  Parameter? _parameter(Map<String, Object?> raw) {
    final resolved = _follow(raw, parameters, 'parameters');
    final name = _string(resolved['name']);
    final location = _string(resolved['in']);
    if (name == null || location == null) {
      return null;
    }
    return Parameter(
      name,
      location: location,
      isRequired: resolved['required'] == true,
      schema: node(_object(resolved['schema'])),
      description: _string(resolved['description']),
    );
  }

  /// What a `200` response carries as JSON, or null when it carries nothing.
  SchemaNode? _success(Map<String, Object?> raw) {
    final ok = _follow(_object(raw['200']), responses, 'responses');
    final schema = _object(
      _object(_object(ok['content'])['application/json'])['schema'],
    );
    return schema.isEmpty ? null : node(schema);
  }

  /// One schema node, recursively.
  SchemaNode node(Map<String, Object?> raw) {
    final type = raw['type'];
    final items = raw['items'];
    final properties = _object(raw['properties']);
    return SchemaNode(
      types: switch (type) {
        final String single => [single],
        final List<Object?> many => [
            for (final entry in many)
              if (entry is String) entry,
          ],
        _ => const [],
      },
      format: _string(raw['format']),
      ref: _schemaRef(raw),
      items: items is Map<String, Object?> ? node(items) : null,
      properties: {
        for (final MapEntry(:key, :value) in properties.entries)
          key: node(_object(value)),
      },
      required: switch (raw['required']) {
        final List<Object?> names => [
            for (final name in names)
              if (name is String) name,
          ],
        _ => const [],
      },
      description: _string(raw['description']),
      enumeration: switch (raw['enum']) {
        final List<Object?> values => [
            for (final value in values)
              if (value is String) value,
          ],
        _ => const [],
      },
    );
  }

  /// The component name a `$ref` into `components/schemas` points at.
  String? _schemaRef(Map<String, Object?> raw) {
    const prefix = '#/components/schemas/';
    final ref = _string(raw[r'$ref']);
    return ref != null && ref.startsWith(prefix)
        ? ref.substring(prefix.length)
        : null;
  }

  /// `raw` itself, or the component it refers to.
  Map<String, Object?> _follow(
    Map<String, Object?> raw,
    Map<String, Object?> components,
    String section,
  ) {
    final ref = _string(raw[r'$ref']);
    final prefix = '#/components/$section/';
    return ref != null && ref.startsWith(prefix)
        ? _object(components[ref.substring(prefix.length)])
        : raw;
  }
}

/// A JSON value as an object, empty when it is anything else.
Map<String, Object?> _object(Object? json) =>
    json is Map<String, Object?> ? json : const {};

/// A JSON value as a string, null when it is anything else.
String? _string(Object? json) => json is String ? json : null;
