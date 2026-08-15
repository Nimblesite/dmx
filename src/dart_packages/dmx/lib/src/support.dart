/// The small value types generated code hands back to you.
///
/// Each one is what a macro returns *instead of* throwing: a change, a
/// violation, a mismatch, a usage error. They are plain, comparable data, so a
/// caller can pattern-match on them, put them in a list, or show them to a
/// person without unwrapping an exception first.
library;

import '../dmx.dart';

// ---------------------------------------------------------------------------
// @dmx('diff') [catalogue.diff]
// ---------------------------------------------------------------------------

/// One field that differs between two instances.
class DmxChange {
  final String field;
  final Object? before;
  final Object? after;

  const DmxChange(this.field, this.before, this.after);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is DmxChange &&
          other.field == field &&
          other.before == before &&
          other.after == after);

  @override
  int get hashCode => Object.hash(runtimeType, field, before, after);

  @override
  String toString() => 'DmxChange($field: $before -> $after)';
}

// ---------------------------------------------------------------------------
// @dmx('validate') [catalogue.validate]
// ---------------------------------------------------------------------------

/// One broken rule, named by the field it belongs to so a form can put the
/// message under the right control.
class Violation {
  final String field;
  final String message;

  const Violation(this.field, this.message);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Violation && other.field == field && other.message == message);

  @override
  int get hashCode => Object.hash(runtimeType, field, message);

  @override
  String toString() => '$field $message';
}

// ---------------------------------------------------------------------------
// @dmx('route') [catalogue.route]
// ---------------------------------------------------------------------------

/// A URI that does not belong to the route that was asked to parse it.
class RouteMismatch {
  final String pattern;
  final String uri;

  const RouteMismatch(this.pattern, this.uri);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is RouteMismatch && other.pattern == pattern && other.uri == uri);

  @override
  int get hashCode => Object.hash(runtimeType, pattern, uri);

  @override
  String toString() => 'RouteMismatch($uri does not match $pattern)';
}

/// Joins a built path to its query without the stray `?` that
/// `Uri(queryParameters: {})` leaves behind.
String dmxLocation(String path, Map<String, String> query) =>
    query.isEmpty ? path : Uri(path: path, queryParameters: query).toString();

/// Drops the entries whose value is absent, so an optional query parameter
/// simply is not in the URL.
Map<String, String> dmxQuery(Map<String, String?> entries) => <String, String>{
      for (final entry in entries.entries)
        if (entry.value case final String value) entry.key: value,
    };

// ---------------------------------------------------------------------------
// @dmx('cli') [catalogue.cli]
// ---------------------------------------------------------------------------

/// Why an argument vector could not be understood. Carries the usage text, so
/// the caller prints one thing and exits.
class UsageError {
  final String message;
  final String usage;

  const UsageError(this.message, this.usage);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is UsageError && other.message == message && other.usage == usage);

  @override
  int get hashCode => Object.hash(runtimeType, message, usage);

  @override
  String toString() => '$message\n\n$usage';
}

/// Splits `--name=value` into its two halves, and leaves everything else
/// alone. Generated parsers call this before they look at anything.
(String, String?) dmxSplitFlag(String argument) =>
    switch (argument.indexOf('=')) {
      -1 => (argument, null),
      final int at => (argument.substring(0, at), argument.substring(at + 1)),
    };

/// An argument vector, sorted into the flags that were set, the options that
/// were given values, and the positionals that were left over.
typedef DmxArguments = (
  Set<String> flags,
  Map<String, String> options,
  List<String> rest,
);

/// Folds an argument vector into [DmxArguments] against the tables a generated
/// parser declares.
///
/// Recursive rather than iterative: every step takes the tail of the vector and
/// the state accumulated so far and returns a new state, so nothing here is
/// reassigned and nothing here throws. The tables are parameters rather than
/// generated code so that two commands in one file cannot share a scanner by
/// accident — which is exactly the bug this shape prevents.
Result<DmxArguments, UsageError> dmxScanArguments(
  List<String> argv, {
  required Set<String> flags,
  required Set<String> options,
  required Map<String, String> abbreviations,
  required String usage,
  Set<String> setFlags = const <String>{},
  Map<String, String> givenOptions = const <String, String>{},
  List<String> rest = const <String>[],
}) =>
    switch (argv) {
      [] => Ok((setFlags, givenOptions, rest)),
      // `--` ends option parsing, as it does everywhere.
      ['--', ...final List<String> tail] => Ok((
          setFlags,
          givenOptions,
          <String>[...rest, ...tail],
        )),
      [final String head, ...final List<String> tail]
          when head.startsWith('-') && head != '-' =>
        _dmxScanOption(
          head,
          tail,
          flags: flags,
          options: options,
          abbreviations: abbreviations,
          usage: usage,
          setFlags: setFlags,
          givenOptions: givenOptions,
          rest: rest,
        ),
      [final String head, ...final List<String> tail] => dmxScanArguments(
          tail,
          flags: flags,
          options: options,
          abbreviations: abbreviations,
          usage: usage,
          setFlags: setFlags,
          givenOptions: givenOptions,
          rest: <String>[...rest, head],
        ),
    };

/// Resolves one `-x` / `--name` / `--name=value` / `--no-name` token.
Result<DmxArguments, UsageError> _dmxScanOption(
  String head,
  List<String> tail, {
  required Set<String> flags,
  required Set<String> options,
  required Map<String, String> abbreviations,
  required String usage,
  required Set<String> setFlags,
  required Map<String, String> givenOptions,
  required List<String> rest,
}) =>
    switch (dmxSplitFlag(head)) {
      (final String token, final String? inline) => switch (dmxCanonicalName(
          token,
          abbreviations,
        )) {
          null => Err(UsageError('Unknown option "$token".', usage)),
          final String name when flags.contains(name) => switch (inline) {
              null => dmxScanArguments(
                  tail,
                  flags: flags,
                  options: options,
                  abbreviations: abbreviations,
                  usage: usage,
                  setFlags: <String>{...setFlags, name},
                  givenOptions: givenOptions,
                  rest: rest,
                ),
              final String value => Err(
                  UsageError(
                    '"--$name" is a flag and takes no value, but got "$value".',
                    usage,
                  ),
                ),
            },
          final String name
              when name.startsWith('no-') &&
                  flags.contains(name.substring(3)) =>
            dmxScanArguments(
              tail,
              flags: flags,
              options: options,
              abbreviations: abbreviations,
              usage: usage,
              setFlags:
                  setFlags.where((flag) => flag != name.substring(3)).toSet(),
              givenOptions: givenOptions,
              rest: rest,
            ),
          final String name when options.contains(name) => switch (
                inline ?? tail.firstOrNull) {
              null => Err(UsageError('"--$name" needs a value.', usage)),
              // A bare `--out --check` is a missing value, not a value of
              // "--check": consuming the next option would hide the mistake.
              final String value when inline == null && value.startsWith('-') =>
                Err(
                  UsageError('"--$name" needs a value.', usage),
                ),
              final String value => dmxScanArguments(
                  inline == null ? tail.skip(1).toList() : tail,
                  flags: flags,
                  options: options,
                  abbreviations: abbreviations,
                  usage: usage,
                  setFlags: setFlags,
                  givenOptions: <String, String>{...givenOptions, name: value},
                  rest: rest,
                ),
            },
          final String name =>
            Err(UsageError('Unknown option "$name".', usage)),
        },
    };

/// `-o` and `--out` name the same option; anything else is unknown.
String? dmxCanonicalName(String token, Map<String, String> abbreviations) =>
    switch (token) {
      _ when token.startsWith('--') => token.substring(2),
      _ when token.startsWith('-') => abbreviations[token.substring(1)],
      _ => null,
    };

// ---------------------------------------------------------------------------
// @dmx('lerp') [catalogue.lerp]
// ---------------------------------------------------------------------------

double dmxLerpDouble(double a, double b, double t) => a + (b - a) * t;

/// Rounds rather than truncates, so a half-step between 0 and 1 lands on 1
/// instead of quietly staying at 0.
int dmxLerpInt(int a, int b, double t) => (a + (b - a) * t).round();

/// Interpolates a duration at microsecond resolution, the finest a `Duration`
/// has.
Duration dmxLerpDuration(Duration a, Duration b, double t) =>
    Duration(microseconds: dmxLerpInt(a.inMicroseconds, b.inMicroseconds, t));

/// Anything that is not a number snaps at the halfway point: there is no
/// meaningful blend of two strings, but there is a defensible choice.
T dmxLerpStep<T>(T a, T b, double t) => t < 0.5 ? a : b;
