//! Stage 6: hygiene [hygiene].
//!
//! Generated Dart obeys the same rules the hand-written Dart in this
//! repository does: it never throws, never casts with `as`, and never asserts
//! away a null with `!`. A built-in macro keeps that promise through its
//! template, which is reviewed. A user-authored template is not reviewed by
//! anyone, so the promise has to be *checked* — and checked on the tree-sitter
//! CST, because `throw` inside a string literal is a string and `x!` is a
//! different thing from `!x`.

use anyhow::{Result, bail};
use tree_sitter::Node;

use crate::frontend::Frontend;

/// A construct generated Dart may not contain, and how to say so.
struct Forbidden {
    /// The CST node kind that identifies it.
    kind: &'static str,
    /// What the author has to do instead.
    advice: &'static str,
}

/// What generated code never throws with.
const NO_THROW: &str = "generated code never throws — return a `Result` instead";

/// What generated code never casts with.
const NO_CAST: &str = "generated code never casts — test with `is` and use the smart cast";

/// What generated code never asserts a null away with.
const NO_ASSERT: &str = "generated code never asserts non-null — handle the null case";

/// Everything [hygiene] forbids in generated Dart.
///
/// Every entry is a node kind the Dart grammar produces only for the construct
/// it names, which is why this is a table and not a scan for characters:
/// `postfix_expression` would also match `i++`, and matching text would also
/// match a comment.
const FORBIDDEN: &[Forbidden] = &[
    Forbidden {
        kind: "throw_expression",
        advice: NO_THROW,
    },
    Forbidden {
        kind: "rethrow_statement",
        advice: NO_THROW,
    },
    Forbidden {
        kind: "type_cast_expression",
        advice: NO_CAST,
    },
    Forbidden {
        kind: "cast_pattern",
        advice: NO_CAST,
    },
    Forbidden {
        kind: "null_assertion_expression",
        advice: NO_ASSERT,
    },
    Forbidden {
        kind: "cascade_null_assertion_expression",
        advice: NO_ASSERT,
    },
    Forbidden {
        kind: "null_assert_pattern",
        advice: NO_ASSERT,
    },
    Forbidden {
        kind: "null_check_pattern",
        advice: NO_ASSERT,
    },
];

/// Refuses generated Dart that contains a construct [hygiene] forbids.
///
/// # Errors
///
/// Fails naming the construct, its line and column, and what to write instead.
/// Also fails when `source` cannot be parsed at all, which the caller has
/// normally already ruled out.
pub fn check(source: &str, origin: &str) -> Result<()> {
    let tree = Frontend::new()?.parse(source)?;
    let mut cursor = tree.walk();
    loop {
        if let Some(found) = offence(cursor.node()) {
            let position = cursor.node().start_position();
            bail!(
                "DMX4003 [hygiene]: {origin} is not valid generated Dart at line {}, column {}: {}",
                position.row.saturating_add(1),
                position.column.saturating_add(1),
                found
            );
        }
        if cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return Ok(());
            }
        }
    }
}

/// What is wrong with this node, if anything.
fn offence(node: Node<'_>) -> Option<&'static str> {
    FORBIDDEN
        .iter()
        .find(|forbidden| forbidden.kind == node.kind())
        .map(|forbidden| forbidden.advice)
}

#[cfg(test)]
mod tests {
    use super::check;

    /// A whole Dart file around one statement, so the parse is a real file.
    fn file(body: &str) -> String {
        format!("Object? probe(Object? value) {{\n  {body}\n}}\n")
    }

    /// [hygiene]: the four hazards are refused, each naming its position.
    #[test]
    fn generated_dart_never_throws_casts_or_asserts() {
        for (body, expected) in [
            ("throw StateError('no');", "never throws"),
            ("return value as String;", "never casts"),
            ("return value!;", "never asserts non-null"),
            (
                "value!..toString();\n  return value;",
                "never asserts non-null",
            ),
            (
                "return switch (value) { String() && final s? => s, _ => null };",
                "never asserts non-null",
            ),
            (
                "switch (value) { case var s as String: return s; default: return null; }",
                "never casts",
            ),
        ] {
            let error = format!("{:#}", check(&file(body), "test").expect_err(body));
            assert!(error.contains("DMX4003"), "{body}: {error}");
            assert!(error.contains(expected), "{body}: {error}");
            assert!(error.contains("line 2"), "{body}: {error}");
        }
    }

    /// [hygiene]: a `rethrow` inside a `catch` is still a throw.
    #[test]
    fn rethrow_is_a_throw() {
        let source = file("try { } catch (e) { rethrow; }");
        assert!(
            format!("{:#}", check(&source, "test").expect_err("rethrow")).contains("never throws")
        );
    }

    /// [hygiene]: the words appearing inside a string or a comment are not
    /// constructs, and a prefix `!` is not a null assertion.
    #[test]
    fn only_real_constructs_are_refused() {
        for body in [
            "return 'throw x as y!';",
            "// throw, as, and ! in a comment\n  return value;",
            "return value == null ? null : !identical(value, 1);",
            "var i = 0; i++; return i;",
            "return value is String ? value : null;",
            "return switch (value) { final String s => s, _ => null };",
        ] {
            check(&file(body), "test").unwrap_or_else(|e| panic!("{body}: {e:#}"));
        }
    }

    /// [hygiene]: what the catalogue's own templates emit passes, so the gate
    /// can be applied to every backend without a rewrite.
    #[test]
    fn the_built_in_catalogue_output_is_hygienic() {
        let source = crate::process_source(
            include_str!("../tests/golden/plain.dart"),
            &crate::Options {
                insert_regions: true,
                check: false,
            },
        )
        .expect("pipeline")
        .output
        .expect("output");
        check(&source, "golden/plain.dart").expect("built-in output is hygienic");
    }
}
