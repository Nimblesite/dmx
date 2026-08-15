//! Recovering a region a human has gutted [emission.inline-backend.region-recovery].
//!
//! Deleting generated members almost never leaves valid Dart: an orphaned `};`
//! closes the class early, and tree-sitter then reports the class's real closing
//! brace as an error *outside* the region. Validating the whole file up front
//! therefore made the one case that most needs regenerating — a region someone
//! just deleted from — the one case that could never recover.
//!
//! The boundary these tests pin down: damage inside the divider is dmx's to
//! repair, damage outside it is never dmx's to touch.

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

const BUILD: Options = Options {
    insert_regions: false,
    check: false,
};

fn model(region_body: &str) -> String {
    format!(
        "import 'package:dmx/dmx.dart';

@dmx('model')
class Address {{
  const Address({{required this.street, required this.city}});

  final String street;
  final String city;

  //#region
{region_body}
  //#endregion
}}
"
    )
}

/// Regenerating a healthy file and a gutted one must land on the same bytes.
fn generated(src: &str) -> String {
    process_source(src, &BUILD)
        .unwrap_or_else(|e| panic!("process_source failed: {e:#}"))
        .output
        .unwrap_or_else(|| src.to_owned())
}

/// The exact shape of a hand-deleted region: a dangling map tail and a `};`
/// that closes the class body early.
#[test]
fn a_region_gutted_by_hand_is_regenerated() {
    let damaged = model("\n        'city': city,\n      };\n");
    let repaired = generated(&damaged);

    assert!(
        repaired.contains("static Result<Address, DecodeError> fromJson"),
        "the gutted region was not regenerated:\n{repaired}"
    );
    // `'city': city,` is legitimate inside the regenerated `toJson`, so the
    // tell for a surviving fragment is a *second* occurrence, not any at all.
    assert_eq!(
        repaired.matches("'city': city,").count(),
        1,
        "the orphaned fragment survived alongside the regenerated one:\n{repaired}"
    );
}

/// Recovery must converge on exactly the same output as the healthy path, or
/// the repaired file would differ from a freshly generated one forever.
#[test]
fn recovery_produces_the_same_bytes_as_a_healthy_regeneration() {
    let healthy = generated(&model(""));
    let gutted = generated(&model("\n        'city': city,\n      };\n"));
    assert_eq!(healthy, gutted);
}

/// An empty region is valid Dart already, so this exercises the ordinary path
/// and proves the recovery branch did not disturb it.
#[test]
fn an_empty_region_still_generates_without_recovery() {
    assert!(generated(&model("")).contains("static Result<Address, DecodeError> fromJson"));
}

/// Damage the author caused outside the divider stays theirs. Emptying the
/// region cannot make this parse, so the original error is reported and the
/// file is left alone.
#[test]
fn code_broken_outside_the_region_is_refused() {
    let broken = model("").replace("final String city;", "final String city  ===;");
    let error = process_source(&broken, &BUILD)
        .expect_err("dmx rewrote a file whose damage was outside the region");
    let text = format!("{error:#}");
    assert!(
        text.contains("DMX4001"),
        "expected the original parse error, got: {text}"
    );
}

/// A file with no region at all and broken code must behave exactly as before:
/// stripping finds nothing, so the original error stands.
#[test]
fn a_broken_file_with_no_region_is_refused() {
    let error = process_source("class Broken { final int  ===; }", &BUILD)
        .expect_err("dmx accepted a broken file that has no region");
    assert!(format!("{error:#}").contains("DMX4001"));
}

/// Regeneration must be a fixed point: repairing an already-repaired file
/// reports no change, so the watcher cannot loop writing to itself.
#[test]
fn a_repaired_file_is_a_fixed_point() {
    let repaired = generated(&model("\n        'city': city,\n      };\n"));
    assert!(
        process_source(&repaired, &BUILD)
            .expect("re-processing a repaired file failed")
            .output
            .is_none(),
        "a repaired file was rewritten a second time"
    );
}

/// The author's own labelled folds are not machine-owned, so a fold that is
/// broken inside is the author's problem and must not be silently emptied.
#[test]
fn a_broken_labelled_fold_is_refused() {
    let src = "import 'package:dmx/dmx.dart';

@dmx('model')
class Address {
  const Address({required this.street, required this.city});

  final String street;
  final String city;

  //#region Helpers
        'city': city,
      };
  //#endregion

  //#region
  //#endregion
}
";
    let error =
        process_source(src, &BUILD).expect_err("dmx emptied a labelled fold it does not own");
    assert!(format!("{error:#}").contains("DMX4001"));
}
