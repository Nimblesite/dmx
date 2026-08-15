//! The harness every macro's tests share [suite].
//!
//! A macro test is always the same three moves: parse a scrap of Dart, expand
//! one declaration in it, and say what must or must not appear in the result.
//! Written out per module that is eleven copies of the same parse-and-expand,
//! and a `contains` assertion repeated for every fragment — which is not just
//! duplication, it is duplication that buries the one thing each test is
//! actually about [CI-DESLOP].
//!
//! Taking the macro's own `expand` as an argument keeps that exact: a test goes
//! through the same function the registry calls for that annotation and no
//! other, so a fragment asserted here is a fragment that macro really emits.

use anyhow::Result;

use crate::frontend::{Annotated as _, Frontend, RawDecl};

/// A macro's entry point: the declaration it is on, and the file it is in.
type Expand = fn(&RawDecl, &[RawDecl]) -> Result<String>;

/// The declarations in `source`, parsed the way the pipeline parses them.
fn declarations(source: &str) -> Vec<RawDecl> {
    Frontend::new()
        .expect("front end")
        .declarations(source)
        .expect("parse")
}

/// What `expand` renders for the first declaration in `source`.
pub fn rendered(expand: Expand, source: &str) -> String {
    let file = declarations(source);
    expand(&file[0], &file).expect("expand")
}

/// What `expand` renders for the declaration in `source` carrying `annotation`.
///
/// Not every macro sits on the first declaration in its file: a REST client is
/// written after the interface it implements, and a fixture after the type it
/// fakes [frontend.name-index].
pub fn rendered_on(expand: Expand, source: &str, annotation: &str) -> String {
    let file = declarations(source);
    let decl = file
        .iter()
        .find(|decl| decl.annotation(annotation).is_some())
        .unwrap_or_else(|| panic!("no declaration carries `@{annotation}`"));
    expand(decl, &file).expect("expand")
}

/// The diagnostic `expand` refuses the first declaration in `source` with.
///
/// Returning it rather than asserting on it keeps the test naming the code it
/// cares about — the refusal and its reason are two different claims.
pub fn refusal(expand: Expand, source: &str) -> String {
    let file = declarations(source);
    let error = expand(&file[0], &file).expect_err("expected a refusal");
    format!("{error:#}")
}

/// One case: the claim it makes, the Dart it makes it about, and the fragments
/// that prove it. The claim carries the spec ID, so `grep` still finds the
/// tests for a section [SPEC-IDS].
pub type Case<'a> = (&'a str, &'a str, &'a [&'a str]);

/// Every case in `cases`, each rendered and checked in turn.
///
/// A macro's suite is a list of claims about one function, and written as a
/// test per claim it is the same four lines repeated with the strings moved
/// [CI-DESLOP]. As data it is the strings alone — and naming the claim keeps a
/// failure here exactly as diagnosable as a function per case was, which is the
/// only thing that made the repetition worth anything.
pub fn each(expand: Expand, cases: &[Case<'_>]) {
    for (claim, source, wanted) in cases {
        let out = rendered(expand, source);
        for fragment in *wanted {
            assert!(
                out.contains(fragment),
                "{claim}\nmissing `{fragment}` in:\n{out}"
            );
        }
    }
}

/// Every fragment in `wanted` appears in `out`.
///
/// The whole output is the failure message: a fragment that is missing is only
/// diagnosable beside what was emitted instead of it.
pub fn emits(out: &str, wanted: &[&str]) {
    for fragment in wanted {
        assert!(out.contains(fragment), "missing `{fragment}` in:\n{out}");
    }
}

/// No fragment in `unwanted` appears in `out`.
pub fn omits(out: &str, unwanted: &[&str]) {
    for fragment in unwanted {
        assert!(!out.contains(fragment), "unwanted `{fragment}` in:\n{out}");
    }
}
