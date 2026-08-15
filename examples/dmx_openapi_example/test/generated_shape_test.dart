/// [dartmacros.files]: the generated tree tracks the OpenAPI document.
///
/// Every assertion here is read out of the vendored document at runtime and
/// checked against what the macro produced. Nothing is hard-coded that the
/// document could tell us, so adding a path or a schema to the document and
/// forgetting to rebuild fails this suite rather than passing quietly.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dmx_openapi_example/api.dart';
import 'package:dmx_openapi_example/frankfurter.dart';
import 'package:test/test.dart';

/// The document the macro generated from.
Map<String, Object?> document() {
  final Object? decoded = jsonDecode(
    File('tool/dmx/frankfurter.openapi.json').readAsStringSync(),
  );
  return decoded is Map<String, Object?> ? decoded : const {};
}

/// A JSON value as an object.
Map<String, Object?> object(Object? json) =>
    json is Map<String, Object?> ? json : const {};

void main() {
  final spec = document();
  final schemas = object(object(spec['components'])['schemas']);
  final paths = object(spec['paths']);

  group('the manifest describes the document', () {
    test('title, version, and server come from the document', () {
      final info = object(spec['info']);
      expect(Frankfurter.title, info['title']);
      expect(Frankfurter.apiVersion, info['version']);
      expect(
        Frankfurter.baseUrl,
        object((spec['servers'] as List<Object?>).first)['url'],
      );
    });

    test('every operationId in the document has a client method', () {
      final declared = <String>[
        for (final entry in paths.values)
          for (final operation in object(entry).values)
            if (object(operation)['operationId'] case final String id) id,
      ];
      expect(declared, isNotEmpty);
      expect(Frankfurter.operationIds, equals(declared));
      expect(FrankfurterClient.operationIds, equals(declared));
    });

    test('every named schema has a generated class and file', () {
      for (final name in schemas.keys) {
        expect(
          Frankfurter.schemaNames,
          contains(name),
          reason: '`$name` is in the document but no class was generated',
        );
        expect(Frankfurter.schemaFiles, contains(name));
      }
    });

    test('every file the manifest names exists and is dmx-owned', () {
      for (final file in Frankfurter.schemaFiles.values) {
        final generated = File('lib/$file');
        expect(generated.existsSync(), isTrue, reason: 'lib/$file is missing');
        expect(
          generated.readAsLinesSync().first,
          startsWith('// dmx: generated from frankfurter.dart'),
          reason: 'lib/$file carries no ownership marker',
        );
      }
    });

    test('the macro named classes the document never did', () {
      // `Rate.providers[]` and `CurrencyDetail.peg` are objects written inline,
      // with no name anywhere in the document. The macro named them after
      // where it found them — work no Mustache template could do.
      expect(Frankfurter.schemaNames, contains('RateProvider'));
      expect(Frankfurter.schemaNames, contains('CurrencyDetailPeg'));
      expect(schemas.keys, isNot(contains('RateProvider')));
      expect(schemas.keys, isNot(contains('CurrencyDetailPeg')));
    });
  });

  group('the generated classes match their schemas', () {
    test('each class declares exactly the schema properties, in order', () {
      final byName = <String, List<String>>{
        Rate.schemaName: Rate.propertyNames,
        Currency.schemaName: Currency.propertyNames,
        CurrencyDetail.schemaName: CurrencyDetail.propertyNames,
        Provider.schemaName: Provider.propertyNames,
      };
      for (final MapEntry(:key, :value) in byName.entries) {
        expect(
          value,
          equals(object(object(schemas[key])['properties']).keys.toList()),
          reason: '`$key` does not match its schema',
        );
      }
    });

    test('an enumerated property carries the values the document allows', () {
      final cadence = object(
        object(object(schemas['Provider'])['properties'])['publish_cadence'],
      );
      final declared = <String>[
        for (final value in cadence['enum'] as List<Object?>)
          if (value is String) value,
      ];
      expect(Provider.publishCadenceValues, equals(declared));
    });
  });

  group('the templates are what shaped the output', () {
    test('every template the macro renders is on disk and editable', () {
      for (final name in ['model', 'client', 'barrel', 'manifest']) {
        final template = File('tool/dmx/templates/$name.mustache');
        expect(
          template.existsSync(),
          isTrue,
          reason: 'tool/dmx/templates/$name.mustache is missing',
        );
        expect(template.readAsStringSync(), contains('{{'));
      }
    });

    test('the client carries the layout its template gives it', () {
      // Proof the shape came from `client.mustache` rather than from Dart
      // string-building: these lines exist in the template verbatim.
      final template =
          File('tool/dmx/templates/client.mustache').readAsStringSync();
      final generated = File('lib/frankfurter_client.dart').readAsStringSync();
      expect(template, contains('final DmxTransport transport;'));
      expect(generated, contains('final DmxTransport transport;'));
      expect(template, contains('Future<Result<T, ApiError>> _send<T>('));
      expect(generated, contains('Future<Result<T, ApiError>> _send<T>('));
    });

    test('the barrel exports every generated file', () {
      final barrel = File('lib/api.dart').readAsStringSync();
      for (final file in Frankfurter.schemaFiles.values) {
        expect(barrel, contains("export '$file';"));
      }
      expect(barrel, contains("export 'frankfurter_client.dart';"));
    });
  });
}
