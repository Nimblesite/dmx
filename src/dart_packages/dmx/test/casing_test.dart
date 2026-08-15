/// [context.helpers]: the Dart casing helpers agree with the Rust ones.
///
/// `src/dart_packages/dmx/lib/src/macros/casing.dart` is a port of `src/dmx/src/casing.rs`, and a
/// port is a thing that drifts. Both sides are pinned to one generated corpus
/// — `src/dmx/tests/casing_corpus.json`, written by `src/dmx/src/casing.rs` —
/// rather than to
/// each other's source, so a change in either language fails a gate in that
/// language instead of surfacing as a strange identifier in somebody's
/// generated file.
///
/// Regenerate after a deliberate change: `UPDATE_GOLDEN=1 cargo test casing`.
library;

import 'dart:convert';
import 'dart:io';

import 'package:dmx/macros.dart';
import 'package:test/test.dart';

void main() {
  // The corpus is the Rust crate's, shared so both sides prove the same cases.
  // This package sits at src/dart_packages/dmx; the crate sits at src/dmx.
  final file = File('../../dmx/tests/casing_corpus.json');
  final Object? corpus = jsonDecode(file.readAsStringSync());
  final cases = switch (corpus) {
    {'cases': final List<Object?> cases} => cases,
    _ => const <Object?>[],
  };

  test('the corpus is present and populated', () {
    expect(
      cases,
      isNotEmpty,
      reason: 'src/dmx/tests/casing_corpus.json is missing or empty — '
          'regenerate it '
          'with `UPDATE_GOLDEN=1 cargo test casing`',
    );
  });

  for (final entry in cases) {
    if (entry is! Map<String, Object?>) {
      continue;
    }
    final input = entry['input'];
    if (input is! String) {
      continue;
    }
    test('`$input` cases as Rust cases it', () {
      expect(dmxWords(input), equals(entry['words']), reason: 'dmxWords');
      expect(dmxCamelCase(input), equals(entry['camel']), reason: 'camel');
      expect(dmxPascalCase(input), equals(entry['pascal']), reason: 'pascal');
      expect(dmxSnakeCase(input), equals(entry['snake']), reason: 'snake');
    });
  }
}
