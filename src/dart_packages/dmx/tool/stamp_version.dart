/// Writes the version a release is publishing into this package's pubspec
/// [release.version].
///
/// The git tag is the version. `pubspec.yaml` carries the placeholder `0.0.0`
/// and the release stamps the tag into its own checkout, so nobody bumps a
/// number by hand and no tag is refused for a file somebody forgot.
///
/// The pubspec is REPARSED, never pattern-matched: `package:yaml` reads it and
/// reports the source span of the `version` scalar, and this replaces exactly
/// those bytes. Every comment, every blank line and every other value survives
/// byte for byte — the same splice-between-known-offsets the generator itself
/// uses to emit inline [emission.inline-backend].
///
/// The pub half of a release is the irreversible half: a published version can
/// only be retracted, never replaced. So this reports what it could not do and
/// exits non-zero rather than guessing.
///
///     dart run tool/stamp_version.dart 1.2.3
library;

import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:yaml/yaml.dart';

/// The version the pubspec carries when no release has stamped it.
const String placeholder = '0.0.0';

void main(List<String> arguments) {
  final result = switch (arguments) {
    [final version] => _stampFile(File('pubspec.yaml'), version),
    _ => const Err<String, String>(
        'usage: dart run tool/stamp_version.dart <major.minor.patch>',
      ),
  };
  switch (result) {
    case Err(:final error):
      stderr.writeln('stamp_version: $error');
      exitCode = 1;
    case Ok(:final value):
      stdout.writeln('pubspec.yaml stamped $value');
  }
}

/// Reads [pubspec], stamps [version] into it, and writes it back. Returns the
/// version on success.
Result<String, String> _stampFile(File pubspec, String version) {
  if (!pubspec.existsSync()) {
    return Err('${pubspec.path} does not exist — run this from the package');
  }
  return switch (stampedPubspec(pubspec.readAsStringSync(), version)) {
    Err(:final error) => Err(error),
    Ok(value: final stamped) => _write(pubspec, stamped, version),
  };
}

/// Writes [contents] to [pubspec] and reports [version] back to the caller.
Result<String, String> _write(File pubspec, String contents, String version) {
  pubspec.writeAsStringSync(contents);
  return Ok(version);
}

/// [source] with its `version:` value replaced by [version], and every other
/// byte exactly where it was.
///
/// Exposed for the tests: a stamper proven only by running the release is a
/// stamper proven once, on the one input nobody can re-run.
Result<String, String> stampedPubspec(String source, String version) {
  if (!isReleaseVersion(version)) {
    return Err('`$version` is not a version (expected major.minor.patch)');
  }
  return switch (loadYamlNode(source)) {
    YamlMap(:final nodes) => switch (nodes['version']) {
        YamlScalar(:final span) => Ok(
            source.replaceRange(span.start.offset, span.end.offset, version),
          ),
        null => const Err('pubspec.yaml declares no `version`'),
        _ =>
          const Err('pubspec.yaml declares a `version` that is not a scalar'),
      },
    _ => const Err('pubspec.yaml is not a YAML mapping'),
  };
}

/// Whether [version] is `major.minor.patch` and nothing else.
///
/// The same rule the bundle half of the release applies, and for the same
/// reason: the marketplace accepts no pre-release suffix, so a tag it would
/// reject has to be refused before either half publishes anything.
bool isReleaseVersion(String version) {
  final parts = version.split('.');
  if (parts.length != 3) {
    return false;
  }
  return parts.every(_isNumber);
}

/// Whether [part] is a bare decimal number with no redundant leading zero.
bool _isNumber(String part) {
  if (part.isEmpty || (part.length > 1 && part.startsWith('0'))) {
    return false;
  }
  return part.codeUnits.every((unit) => unit >= 0x30 && unit <= 0x39);
}
