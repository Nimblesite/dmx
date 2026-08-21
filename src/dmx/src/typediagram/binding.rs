//! What a typeDiagram definition bound to a Mustache template *is*
//! [typediagram.binding].
//!
//! A binding is two pieces of text and a destination: the definition, the
//! template, and the path the render lands on. Nothing in this module knows
//! whether those pieces were written as fences inside one Markdown document
//! [typediagram.documents] or as two files beside each other
//! [typediagram.standalone] — that is the whole point of it existing. The two
//! front ends build the same [`Group`], so there is one context builder, one
//! macro, one validator, one ownership protocol, and one set of diagnostics.
//!
//! [`Origin`] is the only thing a group remembers about where it came from,
//! and it is remembered for exactly one reason: a human reading a diagnostic
//! needs to be told where to look, and "fence 2 on line 10" is the wrong
//! sentence for a file.

use anyhow::Result;
use serde_json::{Map, Value};

/// The default generation target when a template does not name one.
pub const DEFAULT_TARGET: &str = "dart";

/// Every key a `dmx` metadata object may carry.
const DMX_KEYS: &[&str] = &["output", "target"];

/// How a group's definition and templates were written down
/// [typediagram.binding].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    /// Fences inside one Markdown document [typediagram.documents].
    Document,
    /// A `.td` definition file and the `.mustache` files beside it
    /// [typediagram.standalone].
    Files,
}

/// One block of source text dmx bound [typediagram.binding].
///
/// It is a fenced code block in a Markdown document and a whole file in a
/// standalone pair; `line` is what tells the two apart, because it is the
/// offset a position inside `body` is rebased by. A fence's body starts on the
/// line after its opening marker, so the marker's line is that offset. A
/// file's body starts on line one, so its offset is zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fence {
    /// Its one-based position among the document's fenced blocks, or `1` for a
    /// file, which is the only block it has.
    pub ordinal: usize,
    /// The offset that turns a position inside `body` into one in the file the
    /// author is editing.
    pub line: usize,
    /// Its content, exactly as it was read.
    pub body: String,
}

/// How the canonical model template is named wherever a template is named
/// [typediagram.canonical].
pub const CANONICAL: &str = "the canonical model template";

/// Where a bound template's text came from [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    /// A Mustache fence inside a Markdown document, which has no file of its
    /// own [typediagram.documents].
    Fence,
    /// A `.mustache` file beside the definition, named relative to the output
    /// root [typediagram.standalone].
    File(String),
    /// The canonical model template dmx ships, which a definition renders
    /// through when nothing beside it says otherwise [typediagram.canonical].
    Canonical,
}

impl Source {
    /// How this source is named in the context and on an output's marker line,
    /// or `None` for a fence, which is named by its position instead.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Fence => None,
            Self::File(path) => Some(path),
            Self::Canonical => Some(CANONICAL),
        }
    }
}

/// A template bound to the definition it renders [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundTemplate {
    /// The template text.
    pub fence: Fence,
    /// The workspace-relative output path, as the author wrote it or as the
    /// convention derived it.
    pub output: String,
    /// The generation target, defaulting to [`DEFAULT_TARGET`].
    pub target: String,
    /// Where the template text came from.
    pub source: Source,
}

impl BoundTemplate {
    /// How a diagnostic names this template.
    #[must_use]
    pub fn located(&self, document: &str) -> String {
        match &self.source {
            Source::File(path) => format!("the template {path}"),
            Source::Canonical => CANONICAL.to_owned(),
            Source::Fence => format!(
                "the Mustache template in {document} on line {}",
                self.fence.line
            ),
        }
    }

    /// How `dmx explain` heads this template [typediagram.execution].
    #[must_use]
    pub fn heading(&self) -> String {
        match &self.source {
            Source::File(path) => format!("template {path}"),
            Source::Canonical => CANONICAL.to_owned(),
            Source::Fence => format!("fence {} on line {}", self.fence.ordinal, self.fence.line),
        }
    }
}

/// One definition and every template bound to it [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    /// How it was written down.
    pub origin: Origin,
    /// Its one-based position among the document's generation groups, or `1`
    /// for a definition file, which is the only group it has.
    pub ordinal: usize,
    /// The typeDiagram definition.
    pub definition: Fence,
    /// The templates it generates through, in binding order.
    pub templates: Vec<BoundTemplate>,
}

impl Group {
    /// How a diagnostic names this group's definition
    /// [typediagram.diagnostics].
    #[must_use]
    pub fn definition_at(&self, document: &str) -> String {
        match self.origin {
            Origin::Document => format!(
                "{document} (fence {}, line {})",
                self.definition.ordinal, self.definition.line
            ),
            Origin::Files => document.to_owned(),
        }
    }

    /// How a diagnostic names one whole binding — where the definition is and
    /// which template was rendering when it failed
    /// [typediagram.diagnostics].
    #[must_use]
    pub fn located(&self, document: &str, template: &BoundTemplate) -> String {
        match (self.origin, template.source.label()) {
            (Origin::Files, Some(name)) => format!("in {document}, rendered through {name}"),
            _ => format!(
                "in {document} group {}, definition fence on line {}, template fence on line {}",
                self.ordinal, self.definition.line, template.fence.line
            ),
        }
    }

    /// How `dmx explain` heads this group [typediagram.execution].
    #[must_use]
    pub fn heading(&self) -> String {
        match self.origin {
            Origin::Document => format!(
                "typeDiagram fence {} on line {}",
                self.definition.ordinal, self.definition.line
            ),
            Origin::Files => "the definition file".to_owned(),
        }
    }

    /// The second marker line's origin-specific half [typediagram.output].
    ///
    /// A document's outputs are identified by the group and the fences that
    /// produced them, because that is what a reader of the document can point
    /// at. A standalone pair's outputs name the template file instead: the
    /// definition is already on the line above, and the template is the other
    /// half of what a reader has to open to change the result.
    #[must_use]
    pub fn identity(&self, template: &BoundTemplate) -> String {
        match (self.origin, template.source.label()) {
            (Origin::Files, Some(name)) => format!("rendered through {name}"),
            _ => format!(
                "group {}, fences {}/{}",
                self.ordinal, self.definition.ordinal, template.fence.ordinal
            ),
        }
    }
}

/// Where a template's dmx metadata was written [typediagram.binding].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata<'a> {
    /// How a diagnostic names the place the metadata was written.
    pub located: String,
    /// The spelling a reader should copy when theirs is refused.
    pub example: &'a str,
}

/// The settings a `dmx` metadata object carried, with nothing filled in.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Declared {
    /// The output path it named, if it named one.
    output: Option<String>,
    /// The target it named, or [`DEFAULT_TARGET`].
    target: String,
}

impl Default for Declared {
    fn default() -> Self {
        Self {
            output: None,
            target: DEFAULT_TARGET.to_owned(),
        }
    }
}

/// The binding a Mustache fence inside a document declares, or `None` when it
/// declares none [typediagram.binding].
///
/// A fence names its own output or it is not a binding at all: a document has
/// no convention to fall back on, because a fence has no file name to derive
/// one from.
///
/// # Errors
///
/// Fails (`DMX8001`) when the metadata is not a JSON object, when `dmx` is not
/// an object, when an unrecognised key appears, or when `output` is missing or
/// empty.
pub fn in_document(meta: &str, fence: Fence, at: &Metadata<'_>) -> Result<Option<BoundTemplate>> {
    let Some(dmx) = dmx_object(meta, at)? else {
        return Ok(None);
    };
    let declared = settings(&dmx, at)?;
    let Some(output) = declared.output else {
        return Err(fault(at, "`dmx.output` must be a non-empty output path"));
    };
    Ok(Some(BoundTemplate {
        fence,
        output,
        target: declared.target,
        source: Source::Fence,
    }))
}

/// The binding a standalone template file declares, filled in from the
/// convention wherever it declared nothing [typediagram.standalone].
///
/// `declared` is the `key=value` text the template's leading comment carried,
/// or `""` when it carried none. `name` is the base name the output takes when
/// the template names none; the directory it lands in and the extension it
/// carries belong to the target, which is the only thing that knows where a
/// language keeps its sources.
///
/// # Errors
///
/// Fails (`DMX8001`) for the same metadata faults a fence fails on, and
/// (`DMX8007`) when the named target does not exist.
pub fn in_file(
    declared: &str,
    fence: Fence,
    source: String,
    name: &str,
    at: &Metadata<'_>,
) -> Result<BoundTemplate> {
    let declared = settings(&pairs(declared, at)?, at)?;
    let target = super::target::find(&declared.target)?;
    Ok(BoundTemplate {
        fence,
        output: declared
            .output
            .unwrap_or_else(|| format!("{}/{name}.{}", target.source_root, target.extension)),
        target: declared.target,
        source: Source::File(source),
    })
}

/// The `dmx` object `meta` carries, or `None` when it carries none.
///
/// Metadata that does not open with `{` belongs to somebody else's convention
/// and is left alone. Metadata that does is dmx's to read: a JSON object
/// without a `dmx` key is an ordinary example, and anything else is a mistake
/// worth reporting rather than silently generating nothing
/// [typediagram.binding].
fn dmx_object(meta: &str, at: &Metadata<'_>) -> Result<Option<Map<String, Value>>> {
    if !meta.starts_with('{') {
        return Ok(None);
    }
    let Ok(Value::Object(metadata)) = serde_json::from_str::<Value>(meta) else {
        return Err(fault(at, "it is not a JSON object"));
    };
    match metadata.get("dmx") {
        None => Ok(None),
        Some(Value::Object(dmx)) => Ok(Some(dmx.clone())),
        Some(_) => Err(fault(at, "`dmx` is not an object")),
    }
}

/// The settings a standalone template's leading comment declared, as the
/// object [`settings`] reads [typediagram.standalone].
///
/// `key=value`, separated by spaces, because a Mustache comment cannot contain
/// a `}` — the engine reads the first one as the start of the closing braces —
/// and therefore cannot contain the JSON object a fence's info string carries.
/// The keys mean the same thing either way, and so does every refusal, because
/// what reads them is the same function.
fn pairs(text: &str, at: &Metadata<'_>) -> Result<Map<String, Value>> {
    text.split_whitespace()
        .map(|token| match token.split_once('=') {
            Some((key, value)) if !key.is_empty() && !value.is_empty() => {
                Ok((key.to_owned(), Value::String(value.to_owned())))
            }
            _ => Err(fault(
                at,
                &format!("`{token}` is not a `key=value` setting"),
            )),
        })
        .collect()
}

/// The settings one `dmx` object declared, whichever front end read it.
fn settings(dmx: &Map<String, Value>, at: &Metadata<'_>) -> Result<Declared> {
    if let Some(unknown) = dmx.keys().find(|key| !DMX_KEYS.contains(&key.as_str())) {
        return Err(fault(
            at,
            &format!("`dmx.{unknown}` is not a setting dmx knows"),
        ));
    }
    let output = match dmx.get("output") {
        None => None,
        Some(Value::String(output)) if !output.trim().is_empty() => Some(output.trim().to_owned()),
        Some(_) => return Err(fault(at, "`dmx.output` must be a non-empty output path")),
    };
    let target = match dmx.get("target") {
        None => DEFAULT_TARGET.to_owned(),
        Some(Value::String(target)) if !target.trim().is_empty() => target.trim().to_owned(),
        Some(_) => return Err(fault(at, "`dmx.target` must be a target name")),
    };
    Ok(Declared { output, target })
}

/// One unusable-metadata refusal, naming the place and the spelling that works.
fn fault(at: &Metadata<'_>, detail: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "DMX8001 [typediagram.binding]: {} has unusable dmx metadata: {detail}\n\n  {}",
        at.located,
        at.example
    )
}

/// Refuses two templates that would write the same file [typediagram.binding].
///
/// # Errors
///
/// Fails (`DMX8003`) when two bindings claim one output path.
pub fn refuse_duplicate_outputs(document: &str, groups: &[Group]) -> Result<()> {
    let mut seen: Vec<(&str, String)> = Vec::new();
    for group in groups {
        for template in &group.templates {
            let here = template.located(document);
            match seen.iter().find(|(path, _)| *path == template.output) {
                Some((path, first)) => {
                    return Err(anyhow::anyhow!(
                        "DMX8003 [typediagram.binding]: {first} and {here} both generate \
                         `{path}`; one output has one template"
                    ));
                }
                None => seen.push((&template.output, here)),
            }
        }
    }
    Ok(())
}

// A separate file only because binding.rs is near the 500-line ceiling.
#[cfg(test)]
#[path = "binding_tests.rs"]
mod tests;
