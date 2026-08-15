/// A worker for `worker_test.dart` [dartmacros.render].
///
/// Two macros, deliberately different in kind: one renders a template and so
/// must round-trip to the driver mid-expansion, and one returns its Dart
/// directly and must not. Driving both over one connection is what proves the
/// bidirectional path did not break the one-directional one.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx/macros.dart';

/// Renders through the driver, echoing whatever came back.
final class Renders extends DmxMacro {
  const Renders();

  @override
  String get name => 'renders';

  @override
  Future<DmxOutput> expand(DmxInvocation invocation) async =>
      switch (await invocation.templates.render(
        '  // {{name}}\n',
        {'name': invocation.declaration.name},
        name: 'fixture.mustache',
      )) {
        Ok(value: final text) => DmxFragment(text, introduced: const ['name']),
        Err(error: final refusal) => refusal,
      };
}

/// Builds its Dart without a template, and stays synchronous.
final class Direct extends DmxMacro {
  const Direct();

  @override
  String get name => 'direct';

  @override
  DmxOutput expand(DmxInvocation invocation) =>
      DmxFragment('  // direct ${invocation.declaration.name}\n');
}

Future<void> main() => dmxServeMacros([const Renders(), const Direct()]);
