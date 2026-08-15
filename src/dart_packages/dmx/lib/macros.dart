/// Author dmx macros in Dart [dartmacros.api].
///
/// A macro is a name and an `expand` function over a typed invocation. The
/// worker loop [dartmacros.protocol] is `dmxServeMacros` — an author never
/// sees a protocol frame:
///
/// ```dart
/// final class Audit extends DmxMacro {
///   @override
///   String get name => 'audit';
///
///   @override
///   DmxOutput expand(DmxInvocation invocation) =>
///       DmxFragment('  // ${invocation.declaration.name}\n');
/// }
///
/// void main() => dmxServeMacros([Audit()]);
/// ```
///
/// A macro may also hand its model to a Mustache template and let dmx render
/// it [dartmacros.render] — `invocation.templates.render(…)` reaches the very
/// engine the catalogue's own macros use, so the two ways of authoring
/// generated Dart are one system rather than a fork in the road.
///
/// This library is for macro authors and pulls in `dart:io`. Generated code
/// and the apps that hold it use `package:dmx/dmx.dart`, which does not
/// [dartmacros.api].
library;

export 'src/macros/api.dart';
export 'src/macros/casing.dart';
export 'src/macros/worker.dart' show dmxServeMacros;
