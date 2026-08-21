//! The typeDiagram grammar, production by production [typediagram.model].
//!
//! Read against the published grammar: one test per form the language admits,
//! plus the positions a syntax error reports. The compatibility corpus in
//! `tests/typediagram_model.rs` proves the same inputs agree with upstream;
//! this proves the tree they parse to is the one the grammar describes.

use super::super::ast::Decl;
use super::parse;

/// The one declaration `source` parses to.
fn only(source: &str) -> Decl {
    let diagram = parse(source).expect("parse");
    assert_eq!(diagram.decls.len(), 1, "expected exactly one declaration");
    diagram.decls.into_iter().next().expect("one declaration")
}

/// [typediagram.model]: fields keep source order, and commas and line
/// breaks separate interchangeably.
#[test]
fn a_record_keeps_its_fields_in_order() {
    let Decl::Record(record) = only("type User<T> { id: Uuid, name: String\n extra: T, }") else {
        panic!("expected a record");
    };
    assert_eq!(record.name, "User");
    assert_eq!(record.generics, ["T"]);
    assert_eq!(
        record
            .fields
            .iter()
            .map(|f| (f.name.as_str(), f.ty.canonical()))
            .collect::<Vec<_>>(),
        [
            ("id", "Uuid".to_owned()),
            ("name", "String".to_owned()),
            ("extra", "T".to_owned()),
        ]
    );
}

/// [typediagram.model]: bare, record, tuple, and pinned variants in one
/// union, in source order.
#[test]
fn a_union_reads_every_variant_form() {
    let Decl::Union(union) = only(
        "untagged union Shape {\n  Circle { radius: Float }\n  Pair(Int, Int)\n  Point\n  Code = -32700\n}",
    ) else {
        panic!("expected a union");
    };
    assert!(union.untagged);
    assert_eq!(
        union
            .variants
            .iter()
            .map(|v| (v.name.as_str(), v.fields.len(), v.discriminant.clone()))
            .collect::<Vec<_>>(),
        [
            ("Circle", 1, None),
            ("Pair", 2, None),
            ("Point", 0, None),
            ("Code", 0, Some("-32700".to_owned())),
        ]
    );
    assert!(union.variants[1].is_tuple());
    assert!(!union.variants[0].is_tuple());
}

/// [typediagram.model]: nested generic arguments survive intact.
#[test]
fn nested_type_arguments_parse_to_the_written_shape() {
    let Decl::Alias(alias) = only("alias Index = Map<String, List<Option<Product>>>") else {
        panic!("expected an alias");
    };
    assert_eq!(
        alias.target.canonical(),
        "Map<String, List<Option<Product>>>"
    );
}

/// [typediagram.model]: the bare form takes the head's `async`; an
/// overload block spells it per signature and drops the head's, exactly as
/// upstream does.
#[test]
fn function_forms_carry_async_where_upstream_does() {
    let Decl::Function(bare) = only("async function fetch<T>(id: T) -> Bytes") else {
        panic!("expected a function");
    };
    assert!(bare.signatures[0].is_async);
    assert_eq!(bare.generics, ["T"]);

    let Decl::Function(block) = only(
        "async function read {\n (path: String) -> Bytes\n async (path: String, timeout: Float) -> Unit\n}",
    ) else {
        panic!("expected a function");
    };
    assert_eq!(block.signatures.len(), 2);
    assert!(!block.signatures[0].is_async);
    assert!(block.signatures[1].is_async);
}

/// [typediagram.model]: the optional header and `#` comments are not
/// declarations.
#[test]
fn the_header_and_comments_are_not_declarations() {
    let diagram = parse("typeDiagram\n\n# only a note\ntype A { x: Int }\n").expect("parse");
    assert_eq!(diagram.decls.len(), 1);
    assert!(parse("# nothing here\n").expect("parse").decls.is_empty());
}

/// [typediagram.model]: targeting annotations filter a declaration.
#[test]
fn targeting_annotations_attach_to_the_declaration_below_them() {
    let Decl::Record(record) = only("@targets(dart)\n@skipTargets(go)\ntype A { x: Int }") else {
        panic!("expected a record");
    };
    let targeting = record.targeting.expect("targeting");
    assert_eq!(
        targeting.targets.as_deref(),
        Some(["dart".to_owned()].as_slice())
    );
    assert_eq!(
        targeting.skip_targets.as_deref(),
        Some(["go".to_owned()].as_slice())
    );
}

/// [typediagram.diagnostics]: a syntax error names the position and what
/// was expected there.
#[test]
fn syntax_errors_carry_a_position_and_an_expectation() {
    for (source, expected, line, col) in [
        ("type { }", "expected a name", 1, 6),
        ("type A { id }", "expected ':'", 1, 13),
        ("type A { id: }", "expected a name", 1, 14),
        ("alias E String", "expected '='", 1, 9),
        ("union U { A = }", "expected a number", 1, 15),
        ("record A { }", "expected 'type'", 1, 1),
        ("untagged type A { }", "expected 'union'", 1, 10),
        ("@nope(x)\ntype A { }", "unknown annotation '@nope'", 1, 2),
    ] {
        let error = parse(source).expect_err(source);
        assert!(
            error.to_string().contains(expected),
            "{source}: {error} did not contain {expected}"
        );
        assert_eq!((error.0[0].line, error.0[0].col), (line, col), "{source}");
    }
}
