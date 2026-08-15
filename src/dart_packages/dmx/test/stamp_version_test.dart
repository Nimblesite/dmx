/// [release.version]: the pubspec stamper, driven with the versions a release
/// would hand it and the pubspec this package actually ships.
library;

import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:test/test.dart';

import '../tool/stamp_version.dart';

void main() {
  test('a release version is three numbers and nothing else', () {
    expect(isReleaseVersion('0.1.0'), isTrue);
    expect(isReleaseVersion('12.4.199'), isTrue);
    expect(isReleaseVersion(placeholder), isTrue);
    // pub would take some of these; the marketplace half of the same release
    // would not, so both halves refuse them and a tag fails before it builds.
    for (final bad in [
      '1.2',
      '1.2.3.4',
      'v1.2.3',
      '1.2.3-rc1',
      '1.2.3+build',
      '01.2.3',
      '1..3',
      '1.2.x',
      '',
      ' 1.2.3',
    ]) {
      expect(isReleaseVersion(bad), isFalse, reason: bad);
    }
  });

  test('stamping replaces the version and disturbs nothing else', () {
    const source = '''
name: dmx
# The comment above the version, which must survive.
version: 0.0.0
environment:
  sdk: ^3.0.0
''';
    expect(
      stampedPubspec(source, '1.2.3'),
      const Ok<String, String>('''
name: dmx
# The comment above the version, which must survive.
version: 1.2.3
environment:
  sdk: ^3.0.0
'''),
    );
  });

  /// The pubspec opens with a folded `description:` and carries comments
  /// between top-level keys. A stamper that reserialized the document would
  /// pass the test above and destroy this one.
  test('the shipped pubspec keeps every other byte', () {
    final pubspec = _pubspec();
    switch (stampedPubspec(pubspec, '4.5.6')) {
      case Err(:final error):
        fail(error);
      case Ok(value: final stamped):
        expect(stamped, contains('version: 4.5.6'));
        expect(stamped, isNot(contains('version: $placeholder')));
        expect(
          stamped.replaceAll('version: 4.5.6', 'version: $placeholder'),
          pubspec,
        );
    }
  });

  /// The stamp is the only edit a release makes to this file, so it has to be
  /// repeatable: re-running the release, or stamping a tree somebody already
  /// stamped, must not accumulate.
  test('stamping twice lands on the same bytes', () {
    switch (stampedPubspec(_pubspec(), '4.5.6')) {
      case Err(:final error):
        fail(error);
      case Ok(value: final once):
        expect(stampedPubspec(once, '4.5.6'), Ok<String, String>(once));
    }
  });

  test('the shipped pubspec carries the placeholder, ready to be stamped', () {
    expect(_pubspec(), contains('\nversion: $placeholder\n'));
  });

  test('a version the marketplace would reject is refused, not written', () {
    expect(
      stampedPubspec(_pubspec(), '1.2.3-rc1'),
      const Err<String, String>(
        '`1.2.3-rc1` is not a version (expected major.minor.patch)',
      ),
    );
  });

  test('a pubspec with no version is reported, never invented', () {
    expect(
      stampedPubspec('name: dmx\n', '1.2.3'),
      const Err<String, String>('pubspec.yaml declares no `version`'),
    );
    expect(
      stampedPubspec('- not\n- a mapping\n', '1.2.3'),
      const Err<String, String>('pubspec.yaml is not a YAML mapping'),
    );
    expect(
      stampedPubspec('name: dmx\nversion:\n  - 1.2.3\n', '1.2.3'),
      const Err<String, String>(
        'pubspec.yaml declares a `version` that is not a scalar',
      ),
    );
  });

  /// A quoted scalar's span covers the quotes, so a splice that took the
  /// scalar's VALUE and not its source range would leave `version: 1.2.3"`.
  test('a quoted version is replaced whole, quotes and all', () {
    expect(
      stampedPubspec('name: dmx\nversion: "0.0.0"\n', '1.2.3'),
      const Ok<String, String>('name: dmx\nversion: 1.2.3\n'),
    );
  });
}

/// The pubspec this package ships, read from disk so the tests are about the
/// real file rather than a copy of it that drifts.
String _pubspec() => File('pubspec.yaml').readAsStringSync();
