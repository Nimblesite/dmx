//! The core workflow end to end [suite]: a file with an annotated class goes in;
//! the same file comes out with the generated members between the dividers —
//! and nothing else changed.
//!
//! Shape assertions live in `golden.rs`. These tests cover behaviour: what the
//! pipeline does with regions, drift, and files it does not own.

// [TEST-RULES] admits `expect` in a test: a fixture that cannot be built is a
// broken test, and unwinding at the point of failure names it better than any
// `Result` plumbing would. Production code is still held to `unwrap_used` and
// `expect_used` at deny — this relaxation is `cfg(test)`-scoped on purpose.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

use dmx::{Options, process_source};

const INPUT: &str = r"import 'package:dmx/dmx.dart';

@dmx('model')
class User {
  const User({required this.id, this.email, this.tags = const []});

  final String id;
  final String? email;
  final List<String> tags;
}
";

fn opts() -> Options {
    Options {
        insert_regions: true,
        check: false,
    }
}

fn generate(src: &str) -> String {
    process_source(src, &opts())
        .expect("pipeline")
        .output
        .expect("output produced")
}

#[test]
fn generates_all_members_into_the_same_file() {
    let out = generate(INPUT);
    for expected in [
        "//#region",
        // [model.json-codec]: the decode takes an untyped value and carries the
        // path it was reached by, so a nested failure names its own location.
        "static Result<User, DecodeError> fromJson(Object? json, [String path = 'User'])",
        "Map<String, dynamic> toJson()",
        "bool operator ==(Object other)",
        "int get hashCode => Object.hash(",
        "String toString() =>",
        "User copyWith({",
        "DmxPatch<String?> email = const DmxKeep()",
        "dmxDeepEquals(",
        "//#endregion",
    ] {
        assert!(out.contains(expected), "missing `{expected}` in:\n{out}");
    }
    // User code above the divider is untouched.
    assert!(out.starts_with(INPUT.trim_end_matches(['}', '\n'])));
}

/// [emission.inline-backend.no-op-writes]: regenerating over generated output writes nothing.
#[test]
fn second_run_is_a_no_op() {
    let out = generate(INPUT);
    assert!(
        process_source(&out, &opts())
            .expect("pipeline")
            .output
            .is_none(),
        "regeneration over generated output must be byte-identical"
    );
}

/// Generation is a pure function of the source, so an edited region is just
/// drift and is restored on the next run.
#[test]
fn an_edited_region_is_regenerated() {
    let edited = generate(INPUT).replace("other.id == id", "other.id == id /* tweak */");
    let restored = process_source(&edited, &opts())
        .expect("pipeline")
        .output
        .expect("drift should produce a rewrite");
    assert!(
        !restored.contains("/* tweak */"),
        "hand edit survived regeneration"
    );
    assert_eq!(
        restored,
        generate(INPUT),
        "regeneration is not deterministic"
    );
}

/// [execution]: `--check` reports drift without writing.
#[test]
fn check_mode_reports_drift_without_writing() {
    let stale = generate(INPUT).replace("other.id == id", "other.id == id /* stale */");
    let checked = Options {
        check: true,
        ..opts()
    };
    assert!(
        process_source(&stale, &checked)
            .expect("pipeline")
            .output
            .is_some(),
        "drift must be reported under --check"
    );
    assert!(
        process_source(&generate(INPUT), &checked)
            .expect("pipeline")
            .output
            .is_none(),
        "up-to-date sources must not report drift"
    );
}

/// [emission.inline-backend.region-location]: dividers are CST comment tokens, so text in a string is not one.
#[test]
fn markers_inside_string_literals_are_not_dividers() {
    let src = INPUT.replace(
        "  final List<String> tags;\n",
        "  final List<String> tags;\n  static const marker = '//#region';\n",
    );
    let plain = Options {
        insert_regions: false,
        ..opts()
    };
    let err = process_source(&src, &plain).unwrap_err().to_string();
    assert!(
        err.contains("DMX6002"),
        "expected missing-divider error, got: {err}"
    );
}

#[test]
fn missing_divider_without_flag_is_an_error() {
    let plain = Options {
        insert_regions: false,
        ..opts()
    };
    let err = process_source(INPUT, &plain).unwrap_err().to_string();
    assert!(err.contains("DMX6002") && err.contains("--insert-regions"));
}

/// [emission.inline-backend.insertion]: the blank line before `}` is the only whitespace insertion touches.
#[test]
fn insertion_tolerates_existing_blank_lines_before_the_brace() {
    let padded = INPUT.replace(
        "  final List<String> tags;\n}",
        "  final List<String> tags;\n\n}",
    );
    assert!(generate(&padded).contains("//#region"));
}

#[test]
fn two_classes_in_one_file_both_generate() {
    let src = format!(
        "{INPUT}\n@dmx('model')\nclass Address {{\n  const Address({{required this.city}});\n\n  final String city;\n}}\n"
    );
    let out = generate(&src);
    assert_eq!(out.matches("//#region").count(), 2);
    assert!(out.contains("static Result<User, DecodeError> fromJson"));
    assert!(out.contains("static Result<Address, DecodeError> fromJson"));
}

#[test]
fn files_without_models_are_untouched() {
    let src = "class Plain {\n  final int x = 1;\n}\n";
    assert!(
        process_source(src, &opts())
            .expect("pipeline")
            .output
            .is_none()
    );
}

/// [emission.inline-backend.region-location]: two unlabelled dividers in one class is ambiguous.
#[test]
fn duplicate_dividers_are_rejected() {
    let src = INPUT.replace(
        "  final List<String> tags;\n",
        "  final List<String> tags;\n\n  //#region\n  //#endregion\n\n  //#region\n  //#endregion\n",
    );
    let err = process_source(&src, &opts()).unwrap_err().to_string();
    assert!(
        err.contains("DMX6102"),
        "expected duplicate-divider error, got: {err}"
    );
}
