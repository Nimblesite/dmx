//! The built-in `typeDiagram` macro [typediagram].
//!
//! typeDiagram definitions plus Mustache templates equal generated code. The
//! definitions live in an ordinary Markdown document that typeDiagram's own
//! tooling still renders; the templates live beside them; dmx owns everything
//! in between — parsing, resolution, context, rendering, validation, and safe
//! emission — and never runs typeDiagram's CLI, library, or language emitters
//! [typediagram.delivery.baseline].
//!
//! The pipeline is the ordinary one. The Markdown front end synthesizes one
//! [`Invocation`] per generation group and dispatches it through the same
//! macro registry an `@dmx('model')` annotation goes through
//! [typediagram.macro]; what comes back is whole files, emitted by the same
//! ownership protocol a Dart-authored macro's siblings use [dartmacros.files].

pub mod ast;
pub mod context;
pub mod diagnostic;
#[cfg(not(target_arch = "wasm32"))]
pub mod document;
#[cfg(not(target_arch = "wasm32"))]
pub mod emit;
pub mod json;
pub mod lexer;
pub mod markdown;
pub mod model;
pub mod parser;
pub mod target;

use anyhow::Result;

use diagnostic::Diagnostics;
use markdown::{BoundTemplate, Group};
use model::Model;

/// The file-name suffix that makes a Markdown document one dmx generates from
/// [typediagram.documents].
pub const DOCUMENT_SUFFIX: &str = ".dmx.md";

/// One synthesized `typeDiagram` macro invocation [typediagram.macro].
///
/// This is what the Markdown front end hands the registry, and it is
/// deliberately the *whole* input: the document it came from, the group's
/// fences and their bound outputs, and the resolved model. A macro that
/// received less would have to go back to the document, which is how a second
/// rendering path starts.
#[derive(Clone, Copy, Debug)]
pub struct Invocation<'a> {
    /// The document's path as a reader of it would write it.
    pub document: &'a str,
    /// The definition fence and every template bound to it.
    pub group: &'a Group,
    /// The definition, parsed and resolved.
    pub model: &'a Model,
}

/// Whether `path` is a Markdown document dmx generates from
/// [typediagram.documents].
///
/// Recursive discovery takes `*.dmx.md` and nothing else; a Markdown file named
/// explicitly on the command line is accepted whatever it is called, which is
/// [`is_markdown`]'s job.
#[must_use]
pub fn is_document(path: &std::path::Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().ends_with(DOCUMENT_SUFFIX))
}

/// Whether `path` is a Markdown file at all.
#[must_use]
pub fn is_markdown(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

/// The first line of every output a document owns — the shared ownership
/// marker, so the same predicate that protects a hand-written sibling from a
/// Dart macro protects one from a document [dartmacros.files].
#[must_use]
pub fn ownership_marker(document: &str) -> String {
    crate::emit::file_marker(document)
}

/// The second line: which group, which fences, and the content that produced
/// the file [typediagram.output].
///
/// The digests are what make drift visible without reading the whole document.
/// A definition or template edit changes them; prose outside the group does
/// not, which is exactly the dependency rule [typediagram.execution] states.
#[must_use]
pub fn identity_line(group: &Group, template: &BoundTemplate) -> String {
    format!(
        "// dmx: group {}, fences {}/{}, definition {}, template {}, context v{}, dmx {}.",
        group.ordinal,
        group.definition.ordinal,
        template.fence.ordinal,
        digest(&group.definition.body),
        digest(&template.fence.body),
        context::CONTEXT_VERSION,
        crate::VERSION,
    )
}

/// A short, stable content digest.
#[must_use]
pub fn digest(content: &str) -> String {
    blake3::hash(content.as_bytes())
        .to_hex()
        .chars()
        .take(16)
        .collect()
}

/// The complete text of one output: the two marker lines, then the render.
#[must_use]
pub fn file_text(document: &str, group: &Group, template: &BoundTemplate, body: &str) -> String {
    format!(
        "{}\n{}\n\n{body}\n",
        ownership_marker(document),
        identity_line(group, template)
    )
}

/// The resolved model for one group's definition fence.
///
/// # Errors
///
/// Fails (`DMX8004`) when the definition does not tokenize, parse, or resolve,
/// with every position rebased onto the document so the reported line is the
/// one the author's editor shows.
pub fn resolve(document: &str, group: &Group) -> Result<Model> {
    let fault = |found: Diagnostics| {
        anyhow::anyhow!(
            "DMX8004 [typediagram.model]: the typeDiagram definition in {document} (fence {}, \
             line {}) is not valid:\n{}",
            group.definition.ordinal,
            group.definition.line,
            found.in_document(group.definition.line)
        )
    };
    let diagram = parser::parse(&group.definition.body).map_err(fault)?;
    Model::resolve(diagram).map_err(fault)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::markdown::groups;
    use super::{DOCUMENT_SUFFIX, file_text, is_document, is_markdown, resolve};

    /// [typediagram.documents]: recursive discovery takes `*.dmx.md`; every
    /// other Markdown file is documentation until somebody names it.
    #[test]
    fn only_dmx_markdown_is_discovered() {
        assert!(is_document(Path::new("docs/models.dmx.md")));
        assert!(!is_document(Path::new("docs/README.md")));
        assert!(!is_document(Path::new("docs/models.dmx.markdown")));
        assert!(is_markdown(Path::new("docs/README.md")));
        assert!(is_markdown(Path::new("docs/README.MD")));
        assert!(!is_markdown(Path::new("lib/a.dart")));
        assert!(is_markdown(Path::new(&format!("a{DOCUMENT_SUFFIX}"))));
    }

    /// [typediagram.model]: a definition fault is reported at the line the
    /// author's editor shows, not at a fence-relative one.
    #[test]
    fn definition_faults_are_reported_in_document_lines() {
        let document = "# Models\n\nprose\n\n```typeDiagram\ntype A { x: Int }\ntype B { y }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n";
        let bound = groups(document).expect("bind");
        let error = format!(
            "{:#}",
            resolve("docs/a.dmx.md", &bound[0]).expect_err("bad definition")
        );
        assert!(error.contains("DMX8004"), "{error}");
        assert!(error.contains("docs/a.dmx.md"), "{error}");
        // The fence opens on line 5, so its second definition line is line 7.
        assert!(error.contains("line 7, column 12"), "{error}");
    }

    /// [typediagram.output]: the marker lines identify the document, the
    /// fences, and the content — and the body follows them exactly once.
    #[test]
    fn the_file_text_carries_both_marker_lines() {
        let bound = groups("```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n")
            .expect("bind");
        let text = file_text(
            "docs/a.dmx.md",
            &bound[0],
            &bound[0].templates[0],
            "final class A {}",
        );
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("docs/a.dmx.md"));
        assert!(lines[1].contains("group 1, fences 1/2"));
        assert!(lines[1].contains("context v1"));
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "final class A {}");
        assert!(text.ends_with('\n'));
    }
}
