//! The argument surface itself [cli].
//!
//! Everything here drives the real binary the way a shell does, because the
//! thing under test is what a person typing `dmx` gets back: the version a
//! shipped copy reports, the usage it offers when it cannot tell what was
//! asked, and the exit status each of those carries.

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

use std::process::{Command, Output};

/// The binary, run with `args`, from a directory with no Dart in it — so
/// nothing here can accidentally depend on the repo it was built in.
fn dmx(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dmx"))
        .args(args)
        .current_dir(env!("CARGO_TARGET_TMPDIR"))
        .output()
        .expect("run dmx")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// [release.version]: a binary inside a VSIX has to be able to say which build
/// it is. `dmx.path` lets a user point the extension at a copy of their own,
/// and the first question any report about one raises is which copy.
#[test]
fn version_reports_the_crate_version() {
    for flag in ["--version", "-V"] {
        let output = dmx(&[flag]);
        assert!(output.status.success(), "`dmx {flag}` failed");
        assert_eq!(
            stdout(&output).trim(),
            format!("dmx {}", env!("CARGO_PKG_VERSION")),
            "`dmx {flag}` must report the version it was built at"
        );
    }
}

#[test]
fn help_describes_every_subcommand_and_succeeds() {
    for flag in ["--help", "-h"] {
        let output = dmx(&[flag]);
        assert!(output.status.success(), "`dmx {flag}` failed");
        let text = stdout(&output);
        for expected in [
            "dmx build",
            "dmx watch",
            "dmx explain",
            "--insert-regions",
            "--check",
        ] {
            assert!(
                text.contains(expected),
                "`dmx {flag}` omitted `{expected}`:\n{text}"
            );
        }
    }
}

/// Usage goes to stderr and carries a failing status, so a script that mistypes
/// a subcommand stops rather than reading an empty stdout as success.
#[test]
fn an_unrecognised_invocation_fails_with_usage() {
    for args in [vec![], vec!["bulid"], vec!["--nope"]] {
        let output = dmx(&args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "`dmx {}` should have failed",
            args.join(" ")
        );
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(stderr.contains("usage:"), "no usage in:\n{stderr}");
        assert!(stdout(&output).is_empty(), "usage must not go to stdout");
    }
}

/// `watch` has no one-shot flags, and saying so beats watching forever with a
/// flag that was silently dropped [cli].
#[test]
fn watch_refuses_build_only_flags() {
    for flag in ["--insert-regions", "--check"] {
        let output = dmx(&["watch", flag]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "`dmx watch {flag}` should fail"
        );
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            stderr.contains("build-only flags"),
            "`dmx watch {flag}` gave no reason:\n{stderr}"
        );
    }
}

#[test]
fn an_unknown_flag_names_itself() {
    let output = dmx(&["build", "--frobnicate"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        stderr.contains("--frobnicate"),
        "the rejected flag is not in:\n{stderr}"
    );
}
