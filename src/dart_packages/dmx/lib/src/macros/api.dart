/// The typed surface a dmx macro is written against [dartmacros.api].
///
/// Declarations only — no `dart:io`, no protocol. The wire that carries these
/// values lives in `worker.dart`, and the split is what keeps an author
/// reading the shape of their input rather than the shape of a frame.
library;

import 'dart:async';

import '../../dmx.dart';

/// One annotation as the author wrote it, arguments as raw Dart source.
final class DmxAnnotation {
  /// The macro name inside `@dmx(…)`, or a foreign annotation's own name.
  final String name;

  /// Whether this is a `@dmx(…)` trigger rather than a foreign annotation.
  final bool isDmx;

  /// Named arguments: label to the raw source of its value, unevaluated —
  /// `{'table': 'products'}` arrives as `{'table': "'products'"}`.
  final Map<String, String> args;

  /// Builds an annotation view.
  const DmxAnnotation(this.name, {required this.isDmx, this.args = const {}});

  /// The raw source of one argument, or null when absent.
  String? arg(String label) => args[label];

  /// One argument with surrounding string quotes stripped — the common case
  /// of a string-literal configuration value.
  String? stringArg(String label) {
    final source = args[label]?.trim();
    if (source == null) {
      return null;
    }
    final quoted = (source.startsWith("'") && source.endsWith("'")) ||
        (source.startsWith('"') && source.endsWith('"'));
    return quoted && source.length >= 2
        ? source.substring(1, source.length - 1)
        : source;
  }
}

/// One instance field of the annotated declaration.
final class DmxField {
  /// The field name, as the author wrote it.
  final String name;

  /// The type as written, e.g. `DateTime?`.
  final String type;

  /// The same type without its trailing `?`.
  final String typeNonNull;

  /// Whether the written type is nullable.
  final bool nullable;

  /// The constructor default, verbatim, when the author wrote one.
  final String? defaultValue;

  /// Everything attached to the field.
  final List<DmxAnnotation> annotations;

  /// Builds a field view.
  const DmxField(
    this.name, {
    required this.type,
    required this.typeNonNull,
    required this.nullable,
    this.defaultValue,
    this.annotations = const [],
  });
}

/// The annotated declaration, as dmx's front end read it.
final class DmxDeclaration {
  /// The declared name.
  final String name;

  /// `class` or `enum`.
  final String kind;

  /// `sealed`, `abstract`, `final`, and friends.
  final List<String> modifiers;

  /// Source text of the type parameter list, e.g. `<T>`.
  final String typeParams;

  /// The `extends` clause's bare type name, when present.
  final String? extendsName;

  /// Bare type names from `implements`.
  final List<String> interfaces;

  /// Instance fields, in source order.
  final List<DmxField> fields;

  /// Enum constant names, in source order. Empty for a class.
  final List<String> values;

  /// Builds a declaration view.
  const DmxDeclaration(
    this.name, {
    required this.kind,
    this.modifiers = const [],
    this.typeParams = '',
    this.extendsName,
    this.interfaces = const [],
    this.fields = const [],
    this.values = const [],
  });
}

/// dmx's own Mustache engine, offered to the macro [dartmacros.render].
///
/// A custom macro computes a model no template could work out for itself, then
/// hands that model to a template rather than to a `StringBuffer`. The driver
/// renders it with the engine, the standalone-tag handling, and the whitespace
/// normalizer the built-in catalogue goes through, so a project's own macro
/// and a built-in lay their output out by exactly the same rules — and the
/// template stays a file someone can edit without touching Dart.
abstract base class DmxTemplates {
  /// Const-constructible by implementations.
  const DmxTemplates();

  /// `template` rendered against `context`, or a refusal explaining why not.
  ///
  /// `context` is any JSON-shaped model: strings, numbers, booleans, lists,
  /// and maps. Mustache truthiness decides sections, so an empty list takes
  /// the `{{^…}}` branch. `name` names the template in diagnostics.
  ///
  /// The result is a value, never a throw: a template that does not compile
  /// comes back as `Err`, ready to return as a `DmxRefusal`.
  Future<Result<String, DmxRefusal>> render(
    String template,
    Map<String, Object?> context, {
    String name = 'template',
  });
}

/// The engine a macro reaches when nothing is driving it — a unit test that
/// builds a `DmxInvocation` by hand, say. It refuses rather than pretending.
final class _DetachedTemplates extends DmxTemplates {
  const _DetachedTemplates();

  @override
  Future<Result<String, DmxRefusal>> render(
    String template,
    Map<String, Object?> context, {
    String name = 'template',
  }) async =>
      Err<String, DmxRefusal>(
        DmxRefusal(
          'DMX7009',
          'no dmx driver is connected, so `$name` cannot be rendered. '
              'Templates render while `dmx` runs the worker.',
        ),
      );
}

/// Everything one expansion receives [dartmacros.api].
final class DmxInvocation {
  /// The declaration the `@dmx` sits on.
  final DmxDeclaration declaration;

  /// The `@dmx('name', {…})` map, labels to raw source, unevaluated.
  final Map<String, String> args;

  /// The indentation each emitted member line should carry.
  final String memberIndent;

  /// dmx's Mustache engine, for a macro that lays its output out with a
  /// template instead of by hand [dartmacros.render].
  final DmxTemplates templates;

  /// Builds an invocation.
  const DmxInvocation(
    this.declaration, {
    this.args = const {},
    this.memberIndent = '  ',
    this.templates = const _DetachedTemplates(),
  });

  /// The raw source of one `@dmx` argument, or null when absent.
  String? arg(String label) => args[label];

  /// One `@dmx` argument with surrounding string quotes stripped.
  String? stringArg(String label) =>
      DmxAnnotation('', isDmx: true, args: args).stringArg(label);
}

/// What an expansion returns: a fragment or a refusal — a value either way,
/// never a throw [dartmacros.api].
sealed class DmxOutput {
  /// Const-constructible by subclasses.
  const DmxOutput();
}

/// One whole Dart file this expansion authors, named by the macro
/// [dartmacros.files].
final class DmxGeneratedFile {
  /// A bare file name ending in `.dart` — the driver anchors it beside the
  /// annotated declaration's own file and refuses anything path-like.
  final String name;

  /// The file's complete Dart source. The driver prepends its ownership
  /// marker; the macro never writes one.
  final String text;

  /// Builds a file.
  const DmxGeneratedFile(this.name, this.text);
}

/// Generated members, ready for the region.
final class DmxFragment extends DmxOutput {
  /// The Dart text to place inside the declaration's region.
  final String text;

  /// Every identifier the text binds, for hygiene [hygiene].
  final List<String> introduced;

  /// Whole sibling files this expansion also authors, one per name the
  /// macro chooses [dartmacros.files].
  final List<DmxGeneratedFile> files;

  /// Builds a fragment.
  const DmxFragment(this.text,
      {this.introduced = const [], this.files = const []});
}

/// A declined declaration, in the author's terms [diagnostics].
final class DmxRefusal extends DmxOutput {
  /// The diagnostic code, `DMX…`.
  final String code;

  /// What is wrong and what to do about it.
  final String message;

  /// Builds a refusal.
  const DmxRefusal(this.code, this.message);
}

/// A user-defined macro: a name and a way to expand [dartmacros.api].
abstract base class DmxMacro {
  /// Const-constructible by subclasses.
  const DmxMacro();

  /// The `@dmx('name')` this macro answers to.
  String get name;

  /// The generated members for one annotated declaration.
  ///
  /// Returning a `Future` is what lets an expansion await a template render
  /// [dartmacros.render]; a macro that builds its Dart directly stays
  /// synchronous and pays nothing.
  FutureOr<DmxOutput> expand(DmxInvocation invocation);
}
