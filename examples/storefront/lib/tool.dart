// GENERATED REGIONS ARE MACHINE-OWNED. Edit above the divider, run `dmx build`.
//
// `@dmx('cli')` [catalogue.cli] — argv in, a typed object or a usage error out.
//
// Command-line parsing is the purest boilerplate there is: the flags, the
// abbreviations, the defaults, the required checks, and the usage text are all
// restatements of one field list, and the usage text is the one that goes
// stale first because nothing checks it.
//
// The generated parser is recursive, not iterative — every step takes the tail
// of the argument list and the accumulated state and returns a new pair, so
// nothing in it is reassigned and nothing in it throws. A bad argument is a
// `UsageError` carrying the usage text, which is exactly what `main` wants to
// print before exiting non-zero.

import 'package:dmx/dmx.dart';

/// Options for the example's own build command.
@dmx('cli', {'name': 'storefront', 'description': 'Generate and check the storefront example.'})
@dmx('model', {'json': false, 'copyWith': false})
class BuildOptions {
  const BuildOptions({
    required this.out,
    this.check = false,
    this.verbose = false,
    this.help = false,
    this.jobs = 4,
    this.format = 'pretty',
    this.paths = const <String>[],
  });

  @dmx('opt', {'abbr': 'o', 'help': 'Directory to write into.', 'valueHelp': 'dir'})
  final String out;

  @dmx('flag', {'abbr': 'c', 'help': 'Report what would change and exit non-zero.'})
  final bool check;

  @dmx('flag', {'abbr': 'v', 'help': 'Log every file considered.'})
  final bool verbose;

  @dmx('flag', {'abbr': 'h', 'help': 'Print this usage information.'})
  final bool help;

  @dmx('opt', {'abbr': 'j', 'help': 'Parallel workers.', 'valueHelp': 'n'})
  final int jobs;

  @dmx('opt', {'help': 'Output style.', 'allowed': <String>['pretty', 'json']})
  final String format;

  @dmx('rest', {'help': 'Files or directories to process.'})
  final List<String> paths;

  //#region
  static const String usage = 'Usage: storefront [options] <paths...>\n'
      '\n'
      'Generate and check the storefront example.\n'
      '\n'
      '  -o, --out=<dir>            Directory to write into.\n'
      '  -c, --[no-]check           Report what would change and exit non-zero.\n'
      '  -v, --[no-]verbose         Log every file considered.\n'
      '  -h, --[no-]help            Print this usage information.\n'
      '  -j, --jobs=<n>             Parallel workers.\n'
      '                             (defaults to 4)\n'
      '      --format=<pretty|json> Output style.\n'
      '                             (defaults to "pretty")\n'
      '\n'
      '  <paths...>                 Files or directories to process.\n'
      ;

  static const Map<String, String> abbreviations = <String, String>{
    'o': 'out',
    'c': 'check',
    'v': 'verbose',
    'h': 'help',
    'j': 'jobs',
  };

  static const Set<String> flagNames = <String>{
    'check',
    'verbose',
    'help',
  };

  static const Set<String> optionNames = <String>{
    'out',
    'jobs',
    'format',
  };

  static const Set<String> allowedFormat = <String>{
    'pretty',
    'json',
  };

  /// The scan itself lives in the runtime and takes these tables as arguments,
  /// so two commands in one file cannot end up sharing one — which is a bug
  /// that a per-class copy of the scanner invites and this shape forbids.
  static Result<BuildOptions, UsageError> parse(List<String> argv) =>
      switch (dmxScanArguments(
        argv,
        flags: flagNames,
        options: optionNames,
        abbreviations: abbreviations,
        usage: usage,
      )) {
        Err(error: final e) => Err(e),
        Ok(
          value: (
            final Set<String> flags,
            final Map<String, String> options,
            final List<String> rest
          )
        ) =>
          switch ((
            options['out'],
            switch (options['jobs']) { null => 4, final String value => int.tryParse(value) },
            switch (options['format'] ?? 'pretty') { final String value when allowedFormat.contains(value) => value, _ => null },
          )) {
            (null, _, _) => Err(const UsageError('Option "--out" is required.', usage)),
            (_, null, _) => Err(const UsageError('"--jobs" must be a whole number.', usage)),
            (_, _, null) => Err(const UsageError('"--format" must be one of pretty, json.', usage)),
            (
              final String out,
              final int jobs,
              final String format,
            ) =>
              Ok(BuildOptions(
                out: out,
                check: flags.contains('check'),
                verbose: flags.contains('verbose'),
                help: flags.contains('help'),
                jobs: jobs,
                format: format,
                paths: rest,
              )),
          },
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is BuildOptions &&
          other.out == out &&
          other.check == check &&
          other.verbose == verbose &&
          other.help == help &&
          other.jobs == jobs &&
          other.format == format &&
          dmxDeepEquals(other.paths, paths));

  @override
  int get hashCode => Object.hash(
        runtimeType,
        out,
        check,
        verbose,
        help,
        jobs,
        format,
        dmxDeepHash(paths),
      );

  @override
  String toString() => 'BuildOptions(out: $out, check: $check, verbose: $verbose, help: $help, jobs: $jobs, format: $format, paths: $paths)';
  //#endregion
}

/// A second command in the same file, because real tools have more than one.
@dmx('cli', {'name': 'storefront-seed', 'description': 'Write sample data into a database.'})
@dmx('model', {'json': false, 'copyWith': false})
class SeedOptions {
  const SeedOptions({
    this.count = 10,
    this.database = 'storefront.db',
    this.dryRun = false,
  });

  @dmx('opt', {'abbr': 'n', 'help': 'How many products to write.', 'valueHelp': 'count'})
  final int count;

  @dmx('opt', {'abbr': 'd', 'help': 'Database file.', 'valueHelp': 'path'})
  final String database;

  @dmx('flag', {'help': 'Print the statements instead of running them.'})
  final bool dryRun;

  //#region
  static const String usage = 'Usage: storefront-seed [options]\n'
      '\n'
      'Write sample data into a database.\n'
      '\n'
      '  -n, --count=<count>   How many products to write.\n'
      '                        (defaults to 10)\n'
      '  -d, --database=<path> Database file.\n'
      '                        (defaults to "storefront.db")\n'
      '      --[no-]dry-run    Print the statements instead of running them.\n'
      ;

  static const Map<String, String> abbreviations = <String, String>{
    'n': 'count',
    'd': 'database',
  };

  static const Set<String> flagNames = <String>{
    'dry-run',
  };

  static const Set<String> optionNames = <String>{
    'count',
    'database',
  };

  /// The scan itself lives in the runtime and takes these tables as arguments,
  /// so two commands in one file cannot end up sharing one — which is a bug
  /// that a per-class copy of the scanner invites and this shape forbids.
  static Result<SeedOptions, UsageError> parse(List<String> argv) =>
      switch (dmxScanArguments(
        argv,
        flags: flagNames,
        options: optionNames,
        abbreviations: abbreviations,
        usage: usage,
      )) {
        Err(error: final e) => Err(e),
        Ok(
          value: (
            final Set<String> flags,
            final Map<String, String> options,
            _
          )
        ) =>
          switch ((
            switch (options['count']) { null => 10, final String value => int.tryParse(value) },
          )) {
            (null,) => Err(const UsageError('"--count" must be a whole number.', usage)),
            (
              final int count,
            ) =>
              Ok(SeedOptions(
                count: count,
                database: options['database'] ?? 'storefront.db',
                dryRun: flags.contains('dry-run'),
              )),
          },
      };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is SeedOptions &&
          other.count == count &&
          other.database == database &&
          other.dryRun == dryRun);

  @override
  int get hashCode => Object.hash(
        runtimeType,
        count,
        database,
        dryRun,
      );

  @override
  String toString() => 'SeedOptions(count: $count, database: $database, dryRun: $dryRun)';
  //#endregion
}
