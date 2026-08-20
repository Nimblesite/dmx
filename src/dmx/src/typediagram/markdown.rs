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
use serde_json::Value;

/// The default generation target when a template does not name one.
pub const DEFAULT_TARGET: &str = "dart";

/// The info-string language that opens a definition, compared case-insensitively.
const DEFINITION_LANGUAGE: &str = "typediagram";

/// The info-string language a bound template uses.
const TEMPLATE_LANGUAGE: &str = "mustache";

/// One fenced code block dmx looked at [typediagram.documents].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fence {
    /// Its one-based position among the document's top-level fenced blocks.
    pub ordinal: usize,
    /// The one-based document line its opening marker sits on.
    pub line: usize,
    /// Its content, exactly as `CommonMark` reads it.
    pub body: String,
}

/// A template fence bound to the definition above it [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundTemplate {
    /// The fence itself.
    pub fence: Fence,
    /// The workspace-relative output path, as the author wrote it.
    pub output: String,
    /// The generation target, defaulting to [`DEFAULT_TARGET`].
    pub target: String,
}

/// One definition and every template bound to it [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    /// Its one-based position among the document's generation groups.
    pub ordinal: usize,
    /// The typeDiagram fence.
    pub definition: Fence,
    /// The templates it generates through, in document order.
    pub templates: Vec<BoundTemplate>,
}

/// Every generation group in `source`, in document order.
///
/// # Errors
///
/// Fails on malformed fence metadata (`DMX8001`), a bound template with no
/// definition above it (`DMX8002`), or two templates claiming one output path
/// (`DMX8003`).
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
    refuse_duplicate_outputs(&groups)?;
    Ok(groups)
}

/// Refuses two templates that would write the same file [typediagram.binding].
fn refuse_duplicate_outputs(groups: &[Group]) -> Result<()> {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for group in groups {
        for template in &group.templates {
            match seen.iter().find(|(path, _)| *path == template.output) {
                Some((path, line)) => bail!(
                    "DMX8003 [typediagram.binding]: the templates on lines {line} and {} both \
                     generate `{path}`; one output has one template",
                    template.fence.line
                ),
                None => seen.push((&template.output, template.fence.line)),
            }
        }
    }
    Ok(())
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
            Ok(match binding(meta, &fence)? {
                Some(template) => Node::Template(template),
                None => Node::Other,
            })
        }
        () => Ok(Node::Other),
    }
}

/// The dmx binding a Mustache fence declares, or `None` when it declares none.
///
/// Metadata that does not open with `{` belongs to somebody else's convention
/// and is left alone. Metadata that does is dmx's to read: a JSON object
/// without a `dmx` key is an ordinary example, and anything else is a mistake
/// worth reporting rather than silently generating nothing
/// [typediagram.binding].
///
/// # Errors
///
/// Fails when the metadata is not a JSON object, when `dmx` is not an object,
/// when `output` is missing or empty, or when an unrecognised key appears —
/// all `DMX8001`.
fn binding(meta: &str, fence: &Fence) -> Result<Option<BoundTemplate>> {
    if !meta.starts_with('{') {
        return Ok(None);
    }
    let fault = |detail: &str| {
        anyhow::anyhow!(
            "DMX8001 [typediagram.binding]: the Mustache fence on line {} has unusable dmx \
             metadata: {detail}\n\n  ```mustache {{\"dmx\": {{\"output\": \"lib/models.dart\"}}}}",
            fence.line
        )
    };
    let Ok(Value::Object(metadata)) = serde_json::from_str::<Value>(meta) else {
        return Err(fault("it is not a JSON object"));
    };
    let Some(dmx) = metadata.get("dmx") else {
        return Ok(None);
    };
    let Value::Object(dmx) = dmx else {
        return Err(fault("`dmx` is not an object"));
    };
    if let Some(unknown) = dmx.keys().find(|key| !DMX_KEYS.contains(&key.as_str())) {
        return Err(fault(&format!(
            "`dmx.{unknown}` is not a setting dmx knows"
        )));
    }
    let output = match dmx.get("output") {
        Some(Value::String(output)) if !output.trim().is_empty() => output.trim().to_owned(),
        _ => return Err(fault("`dmx.output` must be a non-empty output path")),
    };
    let target = match dmx.get("target") {
        None => DEFAULT_TARGET.to_owned(),
        Some(Value::String(target)) if !target.trim().is_empty() => target.trim().to_owned(),
        Some(_) => return Err(fault("`dmx.target` must be a target name")),
    };
    Ok(Some(BoundTemplate {
        fence: fence.clone(),
        output,
        target,
    }))
}

/// Every key a `dmx` metadata object may carry.
const DMX_KEYS: &[&str] = &["output", "target"];

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
    use super::{DEFAULT_TARGET, groups};

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

    /// [typediagram.binding]: two templates may not claim one path.
    #[test]
    fn one_output_has_one_template() {
        let error = groups(&document(
            "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\na\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\nb\n```",
        ))
        .expect_err("duplicate output");
        assert!(format!("{error:#}").contains("DMX8003"), "{error:#}");
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
