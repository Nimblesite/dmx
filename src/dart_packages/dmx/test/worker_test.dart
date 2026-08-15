/// [dartmacros.protocol], [dartmacros.render]: the worker loop, driven as a
/// driver drives it.
///
/// Black-box: a real `dart` process, spoken to over a real pipe, asserted on
/// the frames that come back. Nothing here reaches into `dmxServeMacros`.
///
/// The point is the direction that is new. A macro that renders sends a
/// request *up* the pipe while the driver is waiting for a reply *down* it,
/// which is the classic way to deadlock a connection that used to carry one
/// conversation. Every read below is under a timeout, so a hang fails the
/// suite in seconds instead of parking CI forever.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';

/// How long any single frame gets. Generous for a cold `dart run`, short
/// enough that a deadlock is a failed test rather than a stuck pipeline.
const Duration _patience = Duration(seconds: 30);

/// Frames the worker has sent, buffered so a test never misses one it did not
/// happen to be awaiting yet.
///
/// Hand-rolled rather than `package:async`'s `StreamQueue`, so this suite
/// needs no dependency the package does not already have.
final class _Frames {
  _Frames(Stream<Map<String, Object?>> source) {
    source.listen((frame) {
      if (_waiting.isEmpty) {
        _arrived.add(frame);
      } else {
        _waiting.removeAt(0).complete(frame);
      }
    });
  }

  final List<Map<String, Object?>> _arrived = [];
  final List<Completer<Map<String, Object?>>> _waiting = [];

  /// The next frame, awaiting one if none has arrived.
  Future<Map<String, Object?>> next() {
    if (_arrived.isNotEmpty) {
      return Future.value(_arrived.removeAt(0));
    }
    final completer = Completer<Map<String, Object?>>();
    _waiting.add(completer);
    return completer.future;
  }
}

/// A running worker, and the frames it has sent.
final class Worker {
  Worker._(this._process, this._frames);

  final Process _process;
  final _Frames _frames;

  /// Starts the fixture worker.
  static Future<Worker> start() async {
    final process = await Process.start('dart', [
      'run',
      'test/fixtures/rendering_worker.dart',
    ]);
    // Worker stderr is a macro author's crash report; surface it rather than
    // swallowing it, or a broken fixture looks like a timeout.
    process.stderr.transform(utf8.decoder).listen(stderr.write);
    final frames = _Frames(
      process.stdout
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .map((line) {
        final Object? decoded = jsonDecode(line);
        return decoded is Map<String, Object?> ? decoded : <String, Object?>{};
      }),
    );
    return Worker._(process, frames);
  }

  /// Sends one frame.
  void send(Map<String, Object?> frame) =>
      _process.stdin.writeln(jsonEncode(frame));

  /// The next frame, or a failure if the worker goes quiet.
  Future<Map<String, Object?>> next() => _frames.next().timeout(
        _patience,
        onTimeout: () => fail(
          'the worker sent no frame within ${_patience.inSeconds}s — the pipe is '
          'deadlocked, which is the failure this suite exists to catch',
        ),
      );

  /// The handshake, which every session opens with.
  Future<Map<String, Object?>> hello() async {
    send({'v': 1, 'op': 'hello'});
    return next();
  }

  /// Asks for one expansion of `macro` over a declaration called `name`.
  void expand(String id, String macro, String name) => send({
        'v': 1,
        'op': 'expand',
        'id': id,
        'macro': macro,
        'invocation': {
          'declaration': {'name': name, 'kind': 'class', 'fields': <Object>[]},
          'args': <String, Object?>{},
          'memberIndent': '  ',
        },
      });

  /// Closes the pipe and returns how the process exited.
  Future<int> stop() async {
    await _process.stdin.close();
    return _process.exitCode.timeout(
      _patience,
      onTimeout: () {
        _process.kill();
        return -1;
      },
    );
  }
}

void main() {
  late Worker worker;

  setUp(() async => worker = await Worker.start());
  tearDown(() async => worker.stop());

  test('the handshake declares the macros the worker serves', () async {
    final hello = await worker.hello();
    expect(hello['macros'], containsAll(<String>['renders', 'direct']));
    // The render op travels worker to driver, so it is not an op this worker
    // serves and must not be advertised as one.
    expect(hello['ops'], equals(<String>['expand']));
    expect(hello['contextVersion'], 1);
  });

  test('a macro that renders asks the driver, then answers', () async {
    await worker.hello();
    worker.expand('e1', 'renders', 'Order');

    final request = await worker.next();
    expect(request['op'], 'render');
    expect(request['name'], 'fixture.mustache');
    expect(request['template'], contains('{{name}}'));
    expect(request['context'], equals({'name': 'Order'}));
    expect(request['id'], isA<String>());

    worker.send({'v': 1, 'id': request['id'], 'text': '  // Order\n'});

    final reply = await worker.next();
    expect(reply['id'], 'e1');
    expect(reply['text'], '  // Order\n');
    expect(reply['introduced'], equals(<String>['name']));
    expect(reply['diagnostics'], isEmpty);
  });

  test('a render the driver could not do comes back as a refusal', () async {
    await worker.hello();
    worker.expand('e1', 'renders', 'Order');

    final request = await worker.next();
    worker.send({
      'v': 1,
      'id': request['id'],
      'error': 'DMX7009: macro template `fixture.mustache` does not compile',
    });

    final reply = await worker.next();
    expect(reply['id'], 'e1');
    expect(reply['refusal'], isA<Map<String, Object?>>());
    final refusal = reply['refusal'];
    if (refusal is Map<String, Object?>) {
      expect(refusal['code'], 'DMX7009');
      expect(refusal['message'], contains('does not compile'));
    }
    // A refusal is a value: the worker is still alive and still serving.
    worker.expand('e2', 'direct', 'Second');
    expect((await worker.next())['id'], 'e2');
  });

  test('a reply carrying no text is a refusal, not a hang', () async {
    await worker.hello();
    worker.expand('e1', 'renders', 'Order');
    final request = await worker.next();
    worker.send({'v': 1, 'id': request['id']});
    final reply = await worker.next();
    expect(reply['id'], 'e1');
    expect(reply['refusal'], isNotNull);
  });

  test('a macro that builds its own Dart never asks to render', () async {
    await worker.hello();
    worker.expand('e1', 'direct', 'Order');
    final reply = await worker.next();
    expect(reply['id'], 'e1');
    expect(reply['text'], '  // direct Order\n');
    expect(
      reply['op'],
      isNull,
      reason: 'the synchronous path must not round-trip to the driver',
    );
  });

  test('expansions are answered in the order they were asked for', () async {
    await worker.hello();
    // Both requests go out before either is answered. A driver is entitled to
    // pipeline, and the worker must not reorder or interleave them.
    worker.expand('e1', 'renders', 'First');
    worker.expand('e2', 'renders', 'Second');

    // The exact frame sequence is the assertion — no clock involved. If the
    // worker started the second expansion early, `Second`'s render request
    // would arrive here instead of `e1`'s reply.
    final first = await worker.next();
    expect(first['op'], 'render');
    expect(first['context'], equals({'name': 'First'}));

    worker.send({'v': 1, 'id': first['id'], 'text': '  // First\n'});

    final firstReply = await worker.next();
    expect(
      firstReply['id'],
      'e1',
      reason: 'the second expansion ran before the first was answered',
    );
    expect(firstReply['text'], '  // First\n');

    final second = await worker.next();
    expect(second['op'], 'render');
    expect(second['context'], equals({'name': 'Second'}));
    worker.send({'v': 1, 'id': second['id'], 'text': '  // Second\n'});

    final secondReply = await worker.next();
    expect(secondReply['id'], 'e2');
    expect(secondReply['text'], '  // Second\n');
  });

  test('render ids are the worker\'s own and do not repeat', () async {
    await worker.hello();
    final ids = <Object?>[];
    for (final name in ['A', 'B', 'C']) {
      worker.expand('e$name', 'renders', name);
      final request = await worker.next();
      ids.add(request['id']);
      worker.send({'v': 1, 'id': request['id'], 'text': '  // $name\n'});
      await worker.next();
    }
    expect(ids.toSet(), hasLength(3));
  });

  test('an unknown macro is a diagnostic, and the worker survives', () async {
    await worker.hello();
    worker.expand('e1', 'nosuch', 'Order');
    final reply = await worker.next();
    expect(reply['id'], 'e1');
    expect(reply['diagnostics'], isNotEmpty);

    worker.expand('e2', 'direct', 'Order');
    expect((await worker.next())['id'], 'e2');
  });

  test('a frame that is not an object is ignored, not fatal', () async {
    await worker.hello();
    worker.send(<String, Object?>{'v': 1, 'op': 'unknown-op'});
    worker.expand('e1', 'direct', 'Order');
    expect((await worker.next())['id'], 'e1');
  });

  test('closing the pipe ends the worker cleanly', () async {
    await worker.hello();
    worker.expand('e1', 'direct', 'Order');
    await worker.next();
    expect(
      await worker.stop(),
      0,
      reason: 'the worker must exit cleanly when the driver closes stdin, '
          'not hang on a listener that never completes',
    );
  });
}
