//! Binding typeDiagram definitions to Mustache templates inside a Markdown
//! document [typediagram.binding].
//!
//! The document is read as a `CommonMark` AST and never as text: a fence is a
//! node, its info string is that node's, and adjacency is a fact about the
//! node list rather than about how many blank lines somebody left. Prose,
//! headings, quotes, lists, and unrelated fences are documentation, and
//! nothing here can rewrite them — this module only reads.
//!
//! A group is one `typeDiagram` fence followed immediately by one or more
//! dmx-enabled `mustache` fences. Everything else in the document is ignored,
//! which is what keeps an ordinary typeDiagram document renderable by the
//! tooling that has always rendered it.

use anyhow::{Result, bail};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use super::binding::{self, BoundTemplate, Fence, Group, Metadata, Origin};

/// The info-string language that opens a definition, compared case-insensitively.
const DEFINITION_LANGUAGE: &str = "typediagram";

/// The info-string language a bound template uses.
const TEMPLATE_LANGUAGE: &str = "mustache";

/// The spelling a reader copies when their fence metadata is refused.
const EXAMPLE: &str = "```mustache {\"dmx\": {\"output\": \"lib/models.dart\"}}";

/// Every generation group in `source`, in document order.
///
/// # Errors
///
/// Fails on malformed fence metadata (`DMX8001`) or on a bound template with
/// no definition above it (`DMX8002`). Two templates claiming one output is
/// [`super::binding::refuse_duplicate_outputs`], which both front ends run.
pub fn groups(source: &str) -> Result<Vec<Group>> {
    let nodes = top_level_fences(source)?;
    let mut groups: Vec<Group> = Vec::new();
    let mut index = 0usize;
    while let Some(node) = nodes.get(index) {
        index = index.saturating_add(1);
        match node {
            Node::Definition(definition) => {
                let mut templates = Vec::new();
                while let Some(Node::Template(template)) = nodes.get(index) {
                    templates.push(template.clone());
                    index = index.saturating_add(1);
                }
                if !templates.is_empty() {
                    groups.push(Group {
                        origin: Origin::Document,
                        ordinal: groups.len().saturating_add(1),
                        definition: definition.clone(),
                        templates,
                    });
                }
            }
            // A bound template with nothing above it has no model to render,
            // and guessing which definition it meant is exactly the implicit
            // global state [typediagram.binding] forbids.
            Node::Template(template) => bail!(
                "DMX8002 [typediagram.binding]: the Mustache template on line {} generating \
                 `{}` is not bound to a typeDiagram definition; put a ```typeDiagram fence \
                 immediately above it",
                template.fence.line,
                template.output
            ),
            Node::Other => {}
        }
    }
    Ok(groups)
}

/// What one top-level fenced block turned out to be.
#[derive(Clone, Debug)]
enum Node {
    /// A renderable typeDiagram definition.
    Definition(Fence),
    /// A Mustache fence carrying dmx metadata.
    Template(BoundTemplate),
    /// Anything else: another language, an example, ordinary prose.
    Other,
}

/// Every top-level block in `source`, classified, in document order.
///
/// A block nested inside a list item or a quote is not in this sequence: its
/// *container* is, as [`Node::Other`]. That is the reading [typediagram.binding]
/// asks for — adjacency is a property of the document's own structure — and it
/// keeps a fence quoted inside an explanation from binding to anything.
///
/// # Errors
///
/// Fails when a Mustache fence carries metadata that was meant to be dmx's and
/// is not usable (`DMX8001`).
fn top_level_fences(source: &str) -> Result<Vec<Node>> {
    let starts = line_starts(source);
    let mut nodes = Vec::new();
    let mut depth = 0usize;
    let mut fences = 0usize;
    let mut open: Option<(String, usize, usize)> = None;
    let mut body = String::new();
    for (event, range) in Parser::new_ext(source, Options::empty()).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    match info_string(&tag) {
                        Some(info) => {
                            fences = fences.saturating_add(1);
                            open = Some((info, fences, line_of(&starts, range.start)));
                            body.clear();
                        }
                        None => nodes.push(Node::Other),
                    }
                }
                depth = depth.saturating_add(1);
            }
            Event::End(end) => {
                depth = depth.saturating_sub(1);
                if depth == 0
                    && end == TagEnd::CodeBlock
                    && let Some((info, ordinal, line)) = open.take()
                {
                    let fence = Fence {
                        ordinal,
                        line,
                        body: std::mem::take(&mut body),
                    };
                    nodes.push(classify(&info, fence)?);
                }
            }
            Event::Text(text) if open.is_some() && depth == 1 => body.push_str(&text),
            Event::Rule if depth == 0 => nodes.push(Node::Other),
            _ => {}
        }
    }
    Ok(nodes)
}

/// The info string of a fenced code block, or `None` for anything else — an
/// indented code block included, since it has no info string to bind with.
fn info_string(tag: &Tag<'_>) -> Option<String> {
    match tag {
        Tag::CodeBlock(CodeBlockKind::Fenced(info)) => Some(info.to_string()),
        _ => None,
    }
}

/// What a fence with this info string is.
///
/// # Errors
///
/// Fails when the fence's metadata was meant to be dmx's and cannot be used
/// (`DMX8001`).
fn classify(info: &str, fence: Fence) -> Result<Node> {
    let (language, meta) = split_info(info);
    match () {
        // The definition fence stays exactly what typeDiagram's own Markdown
        // tooling renders: the bare language, nothing after it
        // [typediagram.documents].
        () if language.eq_ignore_ascii_case(DEFINITION_LANGUAGE) && meta.is_empty() => {
            Ok(Node::Definition(fence))
        }
        () if language.eq_ignore_ascii_case(TEMPLATE_LANGUAGE) => {
            Ok(match declared(meta, &fence)? {
                Some(template) => Node::Template(template),
                None => Node::Other,
            })
        }
        () => Ok(Node::Other),
    }
}

/// The dmx binding a Mustache fence declares, or `None` when it declares none.
///
/// # Errors
///
/// Fails (`DMX8001`) for every reason [`binding::in_document`] fails. A fence names
/// its own output or it is not a binding at all: a document has no convention
/// to fall back on, because a fence has no file name to derive one from.
fn declared(meta: &str, fence: &Fence) -> Result<Option<BoundTemplate>> {
    binding::in_document(
        meta,
        fence.clone(),
        &Metadata {
            located: format!("the Mustache fence on line {}", fence.line),
            example: EXAMPLE,
        },
    )
}

/// The language and the metadata halves of an info string.
fn split_info(info: &str) -> (&str, &str) {
    match info.trim().split_once(char::is_whitespace) {
        Some((language, meta)) => (language, meta.trim()),
        None => (info.trim(), ""),
    }
}

/// The byte offset every line in `source` begins at.
fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'\n')
                .map(|(index, _)| index.saturating_add(1)),
        )
        .collect()
}

/// The one-based line `offset` sits on.
fn line_of(starts: &[usize], offset: usize) -> usize {
    starts.partition_point(|start| *start <= offset).max(1)
}

#[cfg(test)]
mod tests {
    use super::binding::DEFAULT_TARGET;
    use super::groups;

    /// A document with `body` between two ordinary paragraphs, so every test
    /// also proves prose neither binds nor breaks.
    fn document(body: &str) -> String {
        format!("# Models\n\nSome prose.\n\n{body}\n\nMore prose.\n")
    }

    /// The canonical one-definition, one-template document.
    const ONE: &str = "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nclass {{name}} {}\n```";

    /// [typediagram.binding]: one definition binds to the template below it,
    /// and the fence bodies arrive exactly as written.
    #[test]
    fn a_definition_binds_to_the_template_below_it() {
        let found = groups(&document(ONE)).expect("bind");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].ordinal, 1);
        assert_eq!(found[0].definition.body, "type A { x: Int }\n");
        assert_eq!(found[0].definition.ordinal, 1);
        assert_eq!(found[0].templates.len(), 1);
        assert_eq!(found[0].templates[0].output, "lib/a.dart");
        assert_eq!(found[0].templates[0].target, DEFAULT_TARGET);
        assert_eq!(found[0].templates[0].fence.body, "class {{name}} {}\n");
        assert_eq!(found[0].templates[0].fence.ordinal, 2);
    }

    /// [typediagram.binding]: one definition may feed several templates, and a
    /// blank line between fences is not a node.
    #[test]
    fn one_definition_feeds_several_templates() {
        let found = groups(&document(
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/b.dart\",\"target\":\"dart\"}}\nb\n```",
        ))
        .expect("bind");
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0]
                .templates
                .iter()
                .map(|t| t.output.as_str())
                .collect::<Vec<_>>(),
            ["lib/a.dart", "lib/b.dart"]
        );
    }

    /// [typediagram.binding]: prose between the fences ends the group, so the
    /// template below it is an orphan rather than a silent rebinding.
    #[test]
    fn any_other_node_ends_the_group() {
        let error = groups(&document(
            "```typeDiagram\ntype A { x: Int }\n```\n\nA note.\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```",
        ))
        .expect_err("prose ends the group");
        assert!(format!("{error:#}").contains("DMX8002"), "{error:#}");
    }

    /// [typediagram.binding]: a definition nobody templates is documentation,
    /// and a Mustache fence with no dmx metadata is an example.
    #[test]
    fn documentation_only_fences_generate_nothing() {
        assert!(
            groups(&document("```typeDiagram\ntype A { x: Int }\n```"))
                .expect("bind")
                .is_empty()
        );
        assert!(
            groups(&document(
                "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache\n{{name}}\n```"
            ))
            .expect("bind")
            .is_empty()
        );
        assert!(
            groups(&document(
                "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"other\":true}\n{{name}}\n```"
            ))
            .expect("bind")
            .is_empty()
        );
        assert!(
            groups(&document("```dart\nclass A {}\n```"))
                .expect("bind")
                .is_empty()
        );
    }

    /// [typediagram.documents]: dmx metadata on the definition fence would
    /// stop typeDiagram's own tooling rendering it, so such a fence is not a
    /// definition at all.
    #[test]
    fn a_definition_fence_carries_no_metadata() {
        assert!(
            groups(&document(
                "```typeDiagram {\"dmx\":{}}\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```"
            ))
            .expect_err("an annotated definition fence binds nothing")
            .to_string()
            .contains("DMX8002")
        );
    }

    /// [typediagram.binding]: metadata meant for dmx is held to its shape,
    /// rather than silently generating nothing.
    #[test]
    fn unusable_dmx_metadata_is_refused() {
        for (meta, detail) in [
            ("{\"dmx\": }", "it is not a JSON object"),
            ("{\"dmx\": \"lib/a.dart\"}", "`dmx` is not an object"),
            (
                "{\"dmx\": {}}",
                "`dmx.output` must be a non-empty output path",
            ),
            (
                "{\"dmx\": {\"output\": \"  \"}}",
                "`dmx.output` must be a non-empty output path",
            ),
            (
                "{\"dmx\": {\"output\": 7}}",
                "`dmx.output` must be a non-empty output path",
            ),
            (
                "{\"dmx\": {\"output\": \"a.dart\", \"target\": 7}}",
                "`dmx.target` must be a target name",
            ),
            (
                "{\"dmx\": {\"output\": \"a.dart\", \"ouput\": \"typo\"}}",
                "`dmx.ouput` is not a setting dmx knows",
            ),
        ] {
            let source = document(&format!(
                "```typeDiagram\ntype A {{ x: Int }}\n```\n\n```mustache {meta}\na\n```"
            ));
            let error = format!("{:#}", groups(&source).expect_err(meta));
            assert!(error.contains("DMX8001"), "{meta}: {error}");
            assert!(error.contains(detail), "{meta}: {error}");
        }
    }

    /// [typediagram.binding]: two templates may not claim one path, and the
    /// refusal names each fence by the line its reader will scroll to.
    #[test]
    fn one_output_has_one_template() {
        let found = groups(&document(
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nb\n```",
        ))
        .expect("bind");
        let error = format!(
            "{:#}",
            super::binding::refuse_duplicate_outputs("docs/a.dmx.md", &found)
                .expect_err("duplicate output")
        );
        assert!(error.contains("DMX8003"), "{error}");
        assert!(error.contains("on line 9"), "{error}");
        assert!(error.contains("on line 13"), "{error}");
    }

    /// [typediagram.binding]: longer fences, CRLF, Unicode prose, and several
    /// independent groups all read the same.
    #[test]
    fn longer_fences_crlf_and_several_groups_all_bind() {
        let source = "# Título\r\n\r\n````typeDiagram\r\ntype A { x: Int }\r\n````\r\n\r\n````mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\r\na — é\r\n````\r\n\r\n> quoted\r\n\r\n```typeDiagram\r\ntype B { y: Int }\r\n```\r\n\r\n```mustache {\"dmx\":{\"output\":\"lib/b.dart\"}}\r\nb\r\n```\r\n";
        let found = groups(source).expect("bind");
        assert_eq!(found.len(), 2);
        assert_eq!(found[1].ordinal, 2);
        assert_eq!(found[0].templates[0].fence.body, "a — é\n");
        assert_eq!(found[1].definition.body, "type B { y: Int }\n");
        assert_eq!(found[1].definition.ordinal, 3);
    }

    /// [typediagram.binding]: a fence quoted inside a container is part of the
    /// explanation, not of any group.
    #[test]
    fn a_nested_fence_binds_to_nothing() {
        let source = "- an example:\n\n  ```typeDiagram\n  type A { x: Int }\n  ```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n";
        assert!(
            format!(
                "{:#}",
                groups(source).expect_err("the nested fence binds nothing")
            )
            .contains("DMX8002")
        );
    }

    /// [typediagram.diagnostics]: the reported line is the document line the
    /// offending fence opens on.
    #[test]
    fn diagnostics_name_the_document_line() {
        let error = format!(
            "{:#}",
            groups("intro\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n")
                .expect_err("orphan")
        );
        assert!(error.contains("line 3"), "{error}");
    }
}
