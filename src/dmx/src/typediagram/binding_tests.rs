//! What a binding is, in the terms each front end writes it
//! [typediagram.binding].
//!
//! Both halves are asserted together on purpose: the whole reason `binding` is
//! its own module is that a definition bound inside a document and a
//! definition bound to the file beside it have to reach the pipeline as the
//! same thing, and differ only in the sentence a human is shown.

use super::{
    Fence, Group, Metadata, Origin, Source, in_document, in_file, refuse_duplicate_outputs,
};

/// A whole-file fence, the way a standalone pair builds one.
fn file(body: &str) -> Fence {
    Fence {
        ordinal: 1,
        line: 0,
        body: body.to_owned(),
    }
}

/// The metadata rules a standalone template is read under.
fn beside() -> Metadata<'static> {
    Metadata {
        located: "the template models/a.mustache".to_owned(),
        example: "{{! dmx output=lib/a.dart }}",
    }
}

/// The metadata rules a fence inside a document is read under.
fn fenced() -> Metadata<'static> {
    Metadata {
        located: "the Mustache fence on line 7".to_owned(),
        example: "```mustache {\"dmx\": {\"output\": \"lib/models.dart\"}}",
    }
}

/// One standalone template, bound the way a definition file binds it.
fn bound(meta: &str) -> super::BoundTemplate {
    in_file(
        meta,
        file("x"),
        "models/a.mustache".to_owned(),
        "a",
        &beside(),
    )
    .expect(meta)
}

/// [typediagram.standalone]: a template file with no metadata at all is
/// bound anyway — the convention answers both questions it could ask, and
/// the target answers where its language keeps sources and what they are
/// called.
#[test]
fn a_standalone_template_needs_no_metadata() {
    let template = bound("");
    assert_eq!(template.output, "lib/a.dart");
    assert_eq!(template.target, "dart");
    assert_eq!(template.source.label(), Some("models/a.mustache"));
}

/// [typediagram.standalone]: metadata overrides the convention, one key at
/// a time, and the keys are exactly the document's keys.
#[test]
fn standalone_metadata_overrides_the_convention() {
    assert_eq!(
        bound("output=lib/models/a.dart").output,
        "lib/models/a.dart"
    );
    let targeted = bound("target=dart");
    assert_eq!(targeted.output, "lib/a.dart");
    assert_eq!(targeted.target, "dart");
}

/// [typediagram.standalone]: a target nothing generates is refused where
/// it was named, not later and not silently.
#[test]
fn an_unknown_target_is_refused() {
    let error = format!(
        "{:#}",
        in_file(
            "target=kotlin",
            file("x"),
            "models/a.mustache".to_owned(),
            "a",
            &beside(),
        )
        .expect_err("unknown target")
    );
    assert!(error.contains("kotlin"), "{error}");
}

/// [typediagram.binding]: inside a document a template that declares no
/// output declared no binding, and is left alone as an example.
#[test]
fn a_document_template_without_metadata_is_not_bound() {
    for meta in ["", "{\"lang\": \"dart\"}"] {
        assert!(
            in_document(meta, file("x"), &fenced())
                .expect(meta)
                .is_none(),
            "{meta}"
        );
    }
    let error = format!(
        "{:#}",
        in_document("{\"dmx\": {\"target\": \"dart\"}}", file("x"), &fenced())
            .expect_err("no output")
    );
    assert!(
        error.contains("`dmx.output` must be a non-empty output path"),
        "{error}"
    );
}

/// [typediagram.binding]: a refusal names where the metadata was written,
/// whichever front end wrote it, and offers the spelling that works.
#[test]
fn a_refusal_names_the_place_and_the_fix() {
    const BAD: &str = "{\"dmx\": {\"ouput\": \"lib/a.dart\"}}";
    let refusals = [
        format!(
            "{:#}",
            in_file(
                "ouput=lib/a.dart",
                file("x"),
                "models/a.mustache".to_owned(),
                "a",
                &beside()
            )
            .expect_err("unknown key")
        ),
        format!(
            "{:#}",
            in_document(BAD, file("x"), &fenced()).expect_err("unknown key")
        ),
    ];
    for (error, place, example) in refusals
        .iter()
        .zip([("models/a.mustache", "{{!"), ("on line 7", "```mustache")])
        .map(|(error, (place, example))| (error, place, example))
    {
        assert!(error.contains("DMX8001"), "{error}");
        assert!(
            error.contains("`dmx.ouput` is not a setting dmx knows"),
            "{error}"
        );
        assert!(error.contains(place), "{error}");
        assert!(error.contains(example), "{error}");
    }
}

/// [typediagram.binding]: two templates cannot claim one output, and the
/// refusal names both of them the way their front end names them.
#[test]
fn one_output_has_one_template() {
    let templates = ["models/a.mustache", "models/a.wire.mustache"]
        .into_iter()
        .map(|source| in_file("", file("x"), source.to_owned(), "a", &beside()).expect(source))
        .collect();
    let group = Group {
        origin: Origin::Files,
        ordinal: 1,
        definition: file("type A { x: Int }"),
        templates,
    };
    let error = format!(
        "{:#}",
        refuse_duplicate_outputs("models/a.td", std::slice::from_ref(&group))
            .expect_err("one output, two templates")
    );
    assert!(error.contains("DMX8003"), "{error}");
    assert!(error.contains("the template models/a.mustache"), "{error}");
    assert!(
        error.contains("the template models/a.wire.mustache"),
        "{error}"
    );
    assert!(error.contains("`lib/a.dart`"), "{error}");
}

/// [typediagram.diagnostics]: a file is located by its name, a fence by
/// its position, and neither borrows the other's sentence.
#[test]
fn each_origin_is_located_in_its_own_terms() {
    let template = bound("");
    let files = Group {
        origin: Origin::Files,
        ordinal: 1,
        definition: file("type A { x: Int }"),
        templates: vec![template.clone()],
    };
    assert_eq!(files.definition_at("models/a.td"), "models/a.td");
    assert_eq!(
        files.located("models/a.td", &template),
        "in models/a.td, rendered through models/a.mustache"
    );
    assert_eq!(
        files.identity(&template),
        "rendered through models/a.mustache"
    );
    assert_eq!(files.heading(), "the definition file");
    assert_eq!(template.heading(), "template models/a.mustache");

    let fenced = Group {
        origin: Origin::Document,
        ordinal: 2,
        definition: Fence {
            ordinal: 3,
            line: 12,
            body: "type A { x: Int }".to_owned(),
        },
        templates: vec![super::BoundTemplate {
            fence: Fence {
                ordinal: 4,
                line: 17,
                body: "x".to_owned(),
            },
            output: "lib/a.dart".to_owned(),
            target: "dart".to_owned(),
            source: Source::Fence,
        }],
    };
    assert_eq!(
        fenced.definition_at("docs/a.dmx.md"),
        "docs/a.dmx.md (fence 3, line 12)"
    );
    assert_eq!(
        fenced.located("docs/a.dmx.md", &fenced.templates[0]),
        "in docs/a.dmx.md group 2, definition fence on line 12, template fence on line 17"
    );
    assert_eq!(fenced.identity(&fenced.templates[0]), "group 2, fences 3/4");
    assert_eq!(fenced.heading(), "typeDiagram fence 3 on line 12");
    assert_eq!(fenced.templates[0].heading(), "fence 4 on line 17");
    assert_eq!(
        fenced.templates[0].located("docs/a.dmx.md"),
        "the Mustache template in docs/a.dmx.md on line 17"
    );
}
