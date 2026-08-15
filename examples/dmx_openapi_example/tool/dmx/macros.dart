/// The project's macro worker [dartmacros.discovery].
///
/// `dmx build` finds this file by convention, runs it once, and asks it to
/// expand every `@dmx` name it serves. This one serves `openApiClient`, which
/// reads the OpenAPI document beside it and authors **one whole Dart file per
/// schema, plus a client and a barrel** [dartmacros.files]:
///
///   - `components/schemas/Rate` becomes `Rate` in `rate.dart`
///   - the unnamed object inside `Rate.providers[]` becomes `RateProvider`
///   - every `operationId` becomes a method on `FrankfurterClient`
///   - `api.dart` exports the lot
///
/// The author writes ONE annotated class with no members and never types an
/// endpoint, a parameter, a response type, a class name, or a file name. Add a
/// path to the document and a method appears; add a schema and a file appears;
/// remove one and dmx collects the file.
///
/// **Every line of that output is laid out by a Mustache template** in
/// `templates/`, rendered by dmx's own engine over the worker protocol
/// [dartmacros.render]. This macro computes; the templates decide shape. A
/// team that wants a different client — a different error type, a retry, a
/// logging hook — edits `templates/client.mustache` and touches no Dart.
///
/// That division is the point. Neither half could do this alone: no template
/// can resolve a `$ref` or work out that `iso_numeric` is nullable, and no
/// amount of Dart string-building leaves a team a file they can safely edit.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';

import 'src/context.dart';
import 'src/document.dart';

/// Generates a typed client and its models from an OpenAPI document.
final class OpenApiClient extends DmxMacro {
  /// Reads the document with `dart:convert`, so the example needs no pub
  /// dependency at all beyond `test`.
  const OpenApiClient();

  @override
  String get name => 'openApiClient';

  @override
  Future<DmxOutput> expand(DmxInvocation invocation) async =>
      switch (_read(invocation.stringArg('spec'))) {
        Err(:final error) => error,
        Ok(value: final document) => await _generate(invocation, document),
      };

  /// One file per schema, the client, the barrel, and the seed's manifest —
  /// every one of them a template render [dartmacros.render].
  Future<DmxOutput> _generate(
    DmxInvocation invocation,
    Document document,
  ) async {
    final templates = _Templates(invocation.templates);
    final files = <DmxGeneratedFile>[];
    final schemaFiles = <String, String>{};
    for (final schema in document.schemas) {
      final fileName = '${dmxSnakeCase(schema.name)}.dart';
      switch (await templates.render('model', modelContext(schema))) {
        case Err(:final error):
          return error;
        case Ok(value: final text):
          files.add(DmxGeneratedFile(fileName, text));
          schemaFiles[schema.name] = fileName;
      }
    }

    final clientName = '${dmxPascalCase(invocation.declaration.name)}Client';
    final clientFile = '${dmxSnakeCase(clientName)}.dart';
    switch (await templates.render(
      'client',
      clientContext(document, clientName),
    )) {
      case Err(:final error):
        return error;
      case Ok(value: final text):
        files.add(DmxGeneratedFile(clientFile, text));
    }

    switch (await templates.render(
      'barrel',
      barrelContext(
        [for (final file in files) file.name]..sort(),
        document.title,
      ),
    )) {
      case Err(:final error):
        return error;
      case Ok(value: final text):
        files.add(DmxGeneratedFile(barrelName, text));
    }

    return switch (await templates.render(
      'manifest',
      manifestContext(document, schemaFiles, invocation.memberIndent),
    )) {
      Err(:final error) => error,
      Ok(value: final text) => DmxFragment(
          text,
          introduced: const [
            'title',
            'apiVersion',
            'baseUrl',
            'operationIds',
            'schemaNames',
            'schemaFiles',
          ],
          files: files,
        ),
    };
  }
}

/// The barrel every generated file is exported from.
const String barrelName = 'api.dart';

/// The templates this macro renders with, read from disk and rendered by dmx
/// [dartmacros.render].
///
/// Reading them from `templates/` rather than embedding them in this file is
/// deliberate: a template someone can open, diff, and edit is the whole reason
/// to use one. dmx supplies the engine, so what a project's macro renders and
/// what a built-in renders go through the same Mustache, the same
/// standalone-tag handling, and the same whitespace normalizer.
final class _Templates {
  /// Builds a reader over dmx's renderer.
  _Templates(this._engine);

  /// dmx's Mustache, reached over the worker protocol.
  final DmxTemplates _engine;

  /// Templates already read off disk, so a document with forty schemas reads
  /// `model.mustache` once.
  final Map<String, String> _sources = {};

  /// `templates/<name>.mustache` rendered against `context`.
  Future<Result<String, DmxRefusal>> render(
    String name,
    Map<String, Object?> context,
  ) async {
    final source = _sources[name] ?? _read(name);
    if (source == null) {
      return Err(
        DmxRefusal(
          'DMX3923',
          'the macro has no template `$name.mustache`. It is read from '
              '`templates/` beside `tool/dmx/macros.dart`.',
        ),
      );
    }
    _sources[name] = source;
    return _engine.render(source, context, name: '$name.mustache');
  }

  /// One template off disk, or null when it is not there.
  String? _read(String name) {
    final file = File.fromUri(
      Platform.script.resolve('templates/$name.mustache'),
    );
    return file.existsSync() ? file.readAsStringSync() : null;
  }
}

/// The OpenAPI document to read, without being told where it is.
///
/// The project keeps its document beside this worker, so that is where the
/// macro looks. Two of them is the one case the convention cannot settle, and
/// it says so rather than picking.
Result<Document, DmxRefusal> _read(String? override) {
  final directory = Directory.fromUri(Platform.script.resolve('.'));
  final found = override != null
      ? [override]
      : [
          for (final entity in directory.listSync())
            if (entity is File && entity.path.endsWith('.openapi.json'))
              entity.path,
        ]
    ..sort();
  return switch (found) {
    [final only] => _parse(only),
    [] => Err(
        DmxRefusal(
          'DMX3924',
          'no `*.openapi.json` beside `${directory.path}`. This macro '
              'generates from a vendored OpenAPI document, so the document is '
              'part of the project.',
        ),
      ),
    _ => Err(
        DmxRefusal(
          'DMX3925',
          '${found.length} OpenAPI documents beside `${directory.path}`, so '
              "the convention cannot pick one. Say which with {'spec': '…'}.",
        ),
      ),
  };
}

/// One document off disk and through the reader.
Result<Document, DmxRefusal> _parse(String path) {
  final file = File(path);
  if (!file.existsSync()) {
    return Err(
      DmxRefusal(
        'DMX3924',
        "`@dmx('openApiClient', {'spec': '$path'})`: no document there. Paths "
            'are relative to the directory `dmx` runs in.',
      ),
    );
  }
  return readDocument(jsonDecode(file.readAsStringSync()));
}

/// Serves the macro until `dmx` closes the pipe [dartmacros.protocol].
Future<void> main() => dmxServeMacros([const OpenApiClient()]);
