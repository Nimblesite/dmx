/// This package's own macro worker [dartmacros.discovery].
///
/// It serves one macro, `packageVersion`, and it exists so that `dmx` never
/// writes its own version down twice. `pubspec.yaml` declares the version;
/// `lib/src/version.dart` reports it; and the only thing joining them is this
/// macro, which reads the pubspec at generation time. Change the pubspec,
/// save, and the constant follows — there is no second number to forget, and
/// nothing to remember to run.
///
/// dmx generating the package that dmx ships is the point: the worker imports
/// `package:dmx/macros.dart`, the same API any project uses [dartmacros.api].
library;

import 'dart:io';

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';
import 'package:yaml/yaml.dart';

void main() => dmxServeMacros(
      const [PackageVersion()],
      name: 'dmx-package-version',
    );

/// Emits the enclosing package's declared version as a constant
/// [release.version].
final class PackageVersion extends DmxMacro {
  /// Reads `pubspec.yaml`, so the package needs no build step of its own.
  const PackageVersion();

  @override
  String get name => 'packageVersion';

  @override
  DmxOutput expand(DmxInvocation invocation) =>
      switch (_declaredVersion(_pubspec())) {
        Err(:final error) => error,
        Ok(value: final version) => _constant(invocation, version),
      };

  /// The one member the region holds.
  DmxFragment _constant(DmxInvocation invocation, String version) {
    final indent = invocation.memberIndent;
    return DmxFragment(
      [
        '$indent/// The version `pubspec.yaml` declares for this package.',
        "${indent}static const String version = '$version';",
        '',
      ].join('\n'),
      introduced: const ['version'],
    );
  }
}

/// The `pubspec.yaml` governing this worker, found by walking up from the
/// worker's own directory — the same upward walk that found the worker
/// [dartmacros.discovery].
Result<File, DmxRefusal> _pubspec() {
  var directory = Directory.fromUri(Platform.script.resolve('.'));
  while (true) {
    final candidate =
        File('${directory.path}${Platform.pathSeparator}pubspec.yaml');
    if (candidate.existsSync()) {
      return Ok(candidate);
    }
    final parent = directory.parent;
    if (parent.path == directory.path) {
      return const Err(
        DmxRefusal(
          'DMX3920',
          'no `pubspec.yaml` above this macro worker, so there is no version '
              'to read.',
        ),
      );
    }
    directory = parent;
  }
}

/// The `version:` a pubspec declares.
///
/// `loadYaml` is the only thing in this repository allowed to throw at us: it
/// is a third-party parser with no non-throwing entry point, so its failure is
/// caught here and turned into a refusal — a value, like every other outcome a
/// macro produces [dartmacros.api]. Nothing above this line sees an exception.
Result<String, DmxRefusal> _declaredVersion(Result<File, DmxRefusal> pubspec) =>
    switch (pubspec) {
      Err(:final error) => Err(error),
      Ok(value: final file) => _version(file),
    };

/// One pubspec read and parsed.
Result<String, DmxRefusal> _version(File file) {
  final Object? document;
  try {
    document = loadYaml(file.readAsStringSync());
  } on Object catch (failure) {
    return Err(
      DmxRefusal('DMX3921', '`${file.path}` is not readable YAML: $failure'),
    );
  }
  if (document is! YamlMap) {
    return Err(
      DmxRefusal('DMX3921', '`${file.path}` is not a YAML mapping.'),
    );
  }
  final Object? version = document['version'];
  return version is String
      ? Ok(version)
      : Err(
          DmxRefusal(
              'DMX3922', '`${file.path}` declares no `version:` string.'),
        );
}
