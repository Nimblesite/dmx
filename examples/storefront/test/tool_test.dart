/// [catalogue.cli]: argv in, a typed object or a usage error out.
library;

import 'package:dmx/dmx.dart';
import 'package:dmx_storefront_example/tool.dart';
import 'package:test/test.dart';

BuildOptions parsed(List<String> argv) => switch (BuildOptions.parse(argv)) {
      Ok(value: final options) => options,
      Err(error: final error) => fail('$error'),
    };

void main() {
  group('parsing', () {
    test('long options take a value after a space or an equals sign', () {
      expect(parsed(<String>['--out', 'build']).out, 'build');
      expect(parsed(<String>['--out=build']).out, 'build');
    });

    test('abbreviations name the same option', () {
      expect(parsed(<String>['-o', 'build']).out, 'build');
      expect(parsed(<String>['-o=build']).out, 'build');
    });

    test('flags default to false and set to true', () {
      expect(parsed(<String>['-o', 'build']).check, isFalse);
      expect(parsed(<String>['-o', 'build', '--check']).check, isTrue);
      expect(parsed(<String>['-o', 'build', '-v']).verbose, isTrue);
    });

    test('--no-flag turns one back off', () {
      expect(
        parsed(<String>['-o', 'build', '--check', '--no-check']).check,
        isFalse,
      );
    });

    test('typed options are converted', () {
      expect(parsed(<String>['-o', 'build', '--jobs=8']).jobs, 8);
      expect(parsed(<String>['-o', 'build']).jobs, 4);
    });

    test('positional arguments accumulate in order', () {
      expect(
        parsed(<String>['lib', '-o', 'build', 'test']).paths,
        <String>['lib', 'test'],
      );
    });

    test('everything after -- is positional, even if it looks like an option',
        () {
      expect(
        parsed(<String>['-o', 'build', '--', '--check', '-v']).paths,
        <String>['--check', '-v'],
      );
    });
  });

  group('refusing', () {
    // Every refusal is the same claim about a different command line: parsing
    // fails, and the message names what was wrong with it. Written out as a
    // test each, that claim is eight copies of itself with the strings moved.
    // As data it is the strings alone, and one `test` per row keeps a failure
    // naming its own case.
    const refusals = <(String, List<String>, String)>[
      ('a required option that is absent', <String>[], '--out'),
      ('an unknown option, by name', <String>['--colour=red'], 'colour'),
      (
        'a value given to a flag',
        <String>['-o', 'build', '--check=yes'],
        'takes no value'
      ),
      (
        'an option with no value at the end of the line',
        <String>['--out'],
        'needs a value'
      ),
      (
        'an option swallowing the next option instead of its value',
        <String>['--out', '--check'],
        'needs a value'
      ),
      (
        'a typed option that does not convert',
        <String>['-o', 'build', '--jobs=lots'],
        'whole number'
      ),
      (
        'a value outside the allowed set, listing what is allowed',
        <String>['-o', 'build', '--format=xml'],
        'pretty, json'
      ),
    ];

    for (final (what, argv, named) in refusals) {
      test(what, () {
        expect(
          BuildOptions.parse(argv),
          isA<Err<BuildOptions, UsageError>>()
              .having((e) => e.error.message, 'message', contains(named)),
        );
      });
    }

    // Apart from the table because it is a different claim: not what the
    // message says, but that the usage text travels with every refusal.
    test('every refusal carries the usage text with it', () {
      expect(
        BuildOptions.parse(const <String>[]),
        isA<Err<BuildOptions, UsageError>>()
            .having((e) => e.error.usage, 'usage', contains('--out=<dir>')),
      );
    });
  });

  group('usage', () {
    test('lists every option with its abbreviation', () {
      expect(BuildOptions.usage, contains('-o, --out=<dir>'));
      expect(BuildOptions.usage, contains('-c, --[no-]check'));
      expect(BuildOptions.usage, contains('--format=<pretty|json>'));
    });

    test('names defaults, so nobody has to read the source to find them', () {
      expect(BuildOptions.usage, contains('defaults to 4'));
      expect(BuildOptions.usage, contains('defaults to "pretty"'));
    });

    test('carries the command description', () {
      expect(
        BuildOptions.usage,
        contains('Generate and check the storefront example.'),
      );
    });
  });

  test('a second command in the same file parses independently', () {
    expect(
      SeedOptions.parse(const <String>['-n', '25', '--dry-run']),
      Ok<SeedOptions, UsageError>(
        const SeedOptions(count: 25, dryRun: true),
      ),
    );
  });
}
