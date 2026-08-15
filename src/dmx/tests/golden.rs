//! Golden tests for generated code [suite].
//!
//! Each `tests/golden/<name>.dart` is a hand-written source file with no
//! generated members. The pipeline runs over it and **only the bytes between
//! the dividers** are compared against `<name>.expected`. Everything outside a
//! divider is the author's own and is deliberately not part of the comparison —
//! byte-exactness out there is a separate guarantee [emission.inline-backend.byte-exactness], verified by
//! `content_outside_the_divider_is_never_part_of_the_golden`.
//!
//! To accept a deliberate change to the emitted shape:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test --test golden
//! ```

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
use std::fs;
use std::path::{Path, PathBuf};

use dmx::frontend::{REGION_END, REGION_START};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn opts() -> Options {
    Options {
        insert_regions: true,
        check: false,
    }
}

/// Everything the generator owns: the dividers and the lines between them, for
/// every class in the file, in source order.
fn generated_only(src: &str) -> String {
    let mut out = Vec::new();
    let mut inside = false;
    for line in src.lines() {
        match line.trim() {
            REGION_START if !inside => {
                inside = true;
                out.push(line);
            }
            REGION_END if inside => {
                inside = false;
                out.push(line);
            }
            _ if inside => out.push(line),
            _ => {}
        }
    }
    assert!(!inside, "unterminated divider in:\n{src}");
    format!("{}\n", out.join("\n"))
}

/// Compares `actual` to `<name>.expected`, or rewrites it under `UPDATE_GOLDEN`.
fn assert_golden(name: &str, actual: &str) {
    let path = dir().join(format!("{name}.expected"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&path, actual).expect("write golden");
        return;
    }
    let expected = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing golden {}: {e}\nrun: UPDATE_GOLDEN=1 cargo test --test golden",
            path.display()
        )
    });
    assert!(
        expected == actual,
        "golden mismatch for `{name}`\n\
         --- expected ---\n{expected}\n--- actual ---\n{actual}\n\
         run: UPDATE_GOLDEN=1 cargo test --test golden"
    );
}

fn samples() -> Vec<(String, String)> {
    let mut found: Vec<_> = fs::read_dir(dir())
        .expect("tests/golden")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "dart"))
        .collect();
    found.sort();
    assert!(
        found.len() >= 10,
        "the corpus should stay broad, found {}",
        found.len()
    );
    found
        .into_iter()
        .map(|p| {
            let name = p.file_stem().expect("stem").to_string_lossy().into_owned();
            (name, fs::read_to_string(&p).expect("read sample"))
        })
        .collect()
}

fn generate(src: &str) -> String {
    process_source(src, &opts())
        .expect("pipeline")
        .output
        .expect("sample should produce output")
}

/// The emitted shape of every sample, byte for byte.
#[test]
fn corpus_matches_golden() {
    for (name, src) in samples() {
        assert_golden(&name, &generated_only(&generate(&src)));
    }
}

/// Every checked-in example file is exactly what the current generator emits
/// [suite]. The example is the macro catalogue's acceptance criterion
/// [catalogue.macros], so a stale one is a claim nobody has checked.
#[test]
fn the_example_is_up_to_date() {
    // The crate lives at `src/dmx`; the worked examples stay at the repository root.
    let lib = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate manifest is two directories below the repository root")
        .join("examples/storefront/lib");
    let mut files: Vec<PathBuf> = fs::read_dir(&lib)
        .expect("examples/storefront/lib")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|x| x == "dart"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 10,
        "the worked example should stay broad, found {}",
        files.len()
    );
    for path in files {
        let src = fs::read_to_string(&path).expect("example file");
        assert!(
            process_source(&src, &opts())
                .expect("pipeline")
                .output
                .is_none(),
            "{} is stale — run `make example`",
            path.display()
        );
    }
}

/// Regenerating over generated output changes nothing [emission.inline-backend.no-op-writes].
#[test]
fn generation_is_idempotent() {
    for (name, src) in samples() {
        let once = generate(&src);
        assert!(
            process_source(&once, &opts())
                .expect("pipeline")
                .output
                .is_none(),
            "`{name}` is not idempotent: a second run wanted to rewrite it"
        );
    }
}

/// What the golden deliberately excludes: the author's own bytes [emission.inline-backend.byte-exactness].
#[test]
fn content_outside_the_divider_is_never_part_of_the_golden() {
    for (name, src) in samples() {
        let generated = generate(&src);
        let author = |text: &str| {
            let mut inside = false;
            text.lines()
                .filter(|line| match line.trim() {
                    REGION_START if !inside => {
                        inside = true;
                        false
                    }
                    REGION_END if inside => {
                        inside = false;
                        false
                    }
                    trimmed => !inside && !trimmed.is_empty(),
                })
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            author(&src),
            author(&generated),
            "`{name}` disturbed author bytes"
        );
    }
}

/// Every rule the generated code must obey, over the whole corpus [model.json-codec].
#[test]
fn generated_code_never_throws_or_casts() {
    for (name, src) in samples() {
        let region = generated_only(&generate(&src));
        for forbidden in ["throw ", " as ", "!", "_$"] {
            assert!(
                !region.contains(forbidden),
                "`{forbidden}` appears in generated code for `{name}`:\n{region}"
            );
        }
    }
}

/// A labelled `//#region Helpers` belongs to the author [emission.inline-backend.region-location].
#[test]
fn labelled_regions_are_left_alone() {
    let src = fs::read_to_string(dir().join("labelled_region.dart")).expect("sample");
    let out = generate(&src);
    assert!(
        out.contains("//#region Helpers"),
        "author's labelled fold was consumed"
    );
    assert!(
        out.contains("String get shout"),
        "author's member inside the fold was lost"
    );
    assert_eq!(
        out.matches("//#region\n").count() + out.matches("//#region\r\n").count(),
        1,
        "dmx must own exactly one unlabelled divider"
    );
}
