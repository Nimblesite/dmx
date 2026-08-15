/// The macro worker's end of the protocol [dartmacros.protocol].
///
/// One long-lived process per worker file, newline-delimited JSON over
/// stdin/stdout [extensions.worker-protocol]. Requests travel in both
/// directions: the driver asks for an expansion, and the expansion may ask the
/// driver to render a template [dartmacros.render]. A macro author never sees
/// any of it — `dmxServeMacros` is the whole of their `main`.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import '../../dmx.dart';
import 'api.dart';

/// The driver's end of the pipe, from the worker's side [dartmacros.render].
///
/// Requests flow both ways over the one connection: the driver asks for an
/// expansion, and while that expansion runs the worker may ask the driver to
/// render a template. Replies are matched by id, so the two conversations
/// never confuse each other.
final class _Driver extends DmxTemplates {
  _Driver();

  /// Render requests still waiting on the driver, by frame id.
  final Map<String, Completer<Result<String, DmxRefusal>>> _pending = {};

  /// Monotonic request counter, so every id is this worker's alone.
  int _requests = 0;

  @override
  Future<Result<String, DmxRefusal>> render(
    String template,
    Map<String, Object?> context, {
    String name = 'template',
  }) {
    _requests += 1;
    final id = 'r$_requests';
    final pending = Completer<Result<String, DmxRefusal>>();
    _pending[id] = pending;
    _reply({
      'v': 1,
      'op': 'render',
      'id': id,
      'name': name,
      'template': template,
      'context': context,
    });
    return pending.future;
  }

  /// Whether `frame` was a render reply this worker was waiting for.
  ///
  /// A frame carrying no `op` is an answer rather than a request, and the only
  /// thing a worker asks the driver for is a render.
  bool settle(Map<String, Object?> frame) {
    final id = frame['id'];
    final pending = id is String ? _pending.remove(id) : null;
    if (pending == null) {
      return false;
    }
    final text = frame['text'];
    final error = frame['error'];
    pending.complete(
      text is String
          ? Ok<String, DmxRefusal>(text)
          : Err<String, DmxRefusal>(
              DmxRefusal(
                'DMX7009',
                error is String ? error : 'the driver rendered no text',
              ),
            ),
    );
    return true;
  }
}

/// Serves `macros` over the worker protocol until the driver closes stdin
/// [dartmacros.protocol]. This is the whole of `main` for a macro worker.
///
/// The frames are drained by a listener rather than an `await for` loop: an
/// expansion that awaits a render [dartmacros.render] is waiting on a frame
/// that has not arrived yet, and a loop suspended inside its own body would
/// never read it.
///
/// `name` and `version` are this worker's own identity in the handshake, not
/// dmx's — the driver reads neither, and they exist so a worker can say what
/// it is in a log or a `--verbose` trace. `version` defaults to the version of
/// the `dmx` package the worker was built against, which is generated from
/// `pubspec.yaml` rather than written down [release.version].
Future<void> dmxServeMacros(
  List<DmxMacro> macros, {
  String name = 'macros',
  String version = DmxPackage.version,
}) async {
  final byName = {for (final macro in macros) macro.name: macro};
  final driver = _Driver();
  final closed = Completer<void>();
  // Expansions answer in the order they were asked for, whatever each one
  // awaits along the way [dartmacros.pipeline].
  var queue = Future<void>.value();
  final frames = stdin.transform(utf8.decoder).transform(const LineSplitter());
  frames.listen(
    (frame) {
      final Object? message = jsonDecode(frame);
      if (message is! Map<String, Object?>) {
        return;
      }
      switch (message['op']) {
        case 'hello':
          _reply({
            'v': 1,
            'name': name,
            'version': version,
            'contextVersion': 1,
            'ops': ['expand'],
            'macros': byName.keys.toList(),
          });
        case 'expand':
          queue = queue.then(
            (_) async => _reply(await _expand(byName, message, driver)),
          );
        default:
          driver.settle(message);
      }
    },
    onDone: () {
      if (!closed.isCompleted) {
        closed.complete();
      }
    },
  );
  await closed.future;
  await queue;
}

/// One `expand` frame answered [dartmacros.protocol].
Future<Map<String, Object?>> _expand(
  Map<String, DmxMacro> byName,
  Map<String, Object?> message,
  DmxTemplates templates,
) async {
  final id = message['id'];
  final macroName = message['macro'];
  final macro = macroName is String ? byName[macroName] : null;
  if (macro == null) {
    return {
      'v': 1,
      'id': id,
      'diagnostics': ['DMX7002: no macro named `$macroName` here'],
    };
  }
  return switch (await macro.expand(
    _invocation(message['invocation'], templates),
  )) {
    DmxFragment(:final text, :final introduced, :final files) => {
        'v': 1,
        'id': id,
        'text': text,
        'introduced': introduced,
        'files': [
          for (final file in files) {'name': file.name, 'text': file.text},
        ],
        'diagnostics': <Object>[],
      },
    DmxRefusal(:final code, :final message) => {
        'v': 1,
        'id': id,
        'refusal': {'code': code, 'message': message},
      },
  };
}

/// The invocation JSON, decoded into the typed view a macro receives, with the
/// live connection to dmx's renderer attached [dartmacros.render].
DmxInvocation _invocation(Object? json, DmxTemplates templates) {
  if (json is! Map<String, Object?>) {
    return DmxInvocation(
      const DmxDeclaration('', kind: 'class'),
      templates: templates,
    );
  }
  final declaration = json['declaration'];
  final indent = json['memberIndent'];
  return DmxInvocation(
    declaration is Map<String, Object?>
        ? _declaration(declaration)
        : const DmxDeclaration('', kind: 'class'),
    args: _stringMap(json['args']),
    memberIndent: indent is String ? indent : '  ',
    templates: templates,
  );
}

/// One declaration object decoded.
DmxDeclaration _declaration(Map<String, Object?> json) {
  final name = json['name'];
  final kind = json['kind'];
  final typeParams = json['typeParams'];
  final extendsName = json['extends'];
  return DmxDeclaration(
    name is String ? name : '',
    kind: kind is String ? kind : 'class',
    modifiers: _strings(json['modifiers']),
    typeParams: typeParams is String ? typeParams : '',
    extendsName: extendsName is String ? extendsName : null,
    interfaces: _strings(json['interfaces']),
    fields: [
      for (final field in _maps(json['fields'])) _field(field),
    ],
    values: [
      for (final value in _maps(json['values']))
        if (value['name'] case final String valueName) valueName,
    ],
  );
}

/// One field object decoded.
DmxField _field(Map<String, Object?> json) {
  final name = json['name'];
  final type = json['type'];
  final typeNonNull = json['typeNonNull'];
  final defaultValue = json['defaultValue'];
  return DmxField(
    name is String ? name : '',
    type: type is String ? type : '',
    typeNonNull: typeNonNull is String ? typeNonNull : '',
    nullable: json['nullable'] == true,
    defaultValue: defaultValue is String ? defaultValue : null,
    annotations: [
      for (final annotation in _maps(json['annotations']))
        if (annotation['name'] case final String annotationName)
          DmxAnnotation(
            annotationName,
            isDmx: annotation['dmx'] == true,
            args: _stringMap(annotation['args']),
          ),
    ],
  );
}

/// A JSON value as a list of strings, dropping anything else.
List<String> _strings(Object? json) => [
      if (json is List<Object?>)
        for (final item in json)
          if (item is String) item,
    ];

/// A JSON value as a list of objects, dropping anything else.
List<Map<String, Object?>> _maps(Object? json) => [
      if (json is List<Object?>)
        for (final item in json)
          if (item is Map<String, Object?>) item,
    ];

/// A JSON object as a string-to-string map, dropping anything else.
Map<String, String> _stringMap(Object? json) => {
      if (json is Map<String, Object?>)
        for (final MapEntry(:key, :value) in json.entries)
          if (value is String) key: value,
    };

/// One protocol frame onto stdout — the worker's output contract.
void _reply(Map<String, Object?> frame) => stdout.writeln(jsonEncode(frame));
