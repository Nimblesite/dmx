//! A typeDiagram definition file and the Mustache files beside it
//! [typediagram.standalone].
//!
//! Files and no wrapper:
//!
//! ```text
//! models/shipping.td               the definition — pure typeDiagram
//! models/shipping.wire.mustache    a template     — pure Mustache
//! lib/shipping.dart                the canonical output
//! lib/shipping_wire.dart           the template's output
//! ```
//!
//! Nothing is extracted from anything. The `.td` file is byte-for-byte what
//! typeDiagram's own tooling reads, a `.mustache` file is byte-for-byte what
//! any Mustache engine renders, and the binding between them is their names.
//! A definition with nothing beside it renders through the canonical model
//! template [typediagram.canonical]; `shipping.mustache` would take that
//! template's place, and `shipping.wire.mustache` renders the same definition
//! a second way into a second file. A template that wants a different
//! destination says so in a leading Mustache comment, which is the one place a
//! template can carry metadata without ceasing to be a template.
//!
//! This is a front end and nothing else: bind the files, hand the group to
//! [`super::run`], and let the shared pipeline do the rest.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use super::binding::{self, BoundTemplate, Fence, Group, Metadata, Origin, Source};
use super::{emit, run, target};
use crate::{Options, Outcome, casing};

/// The extension that makes a file a typeDiagram definition
/// [typediagram.standalone].
pub const DEFINITION_EXTENSION: &str = "td";

/// The extension that makes a file a Mustache template.
pub const TEMPLATE_EXTENSION: &str = "mustache";

/// The word that makes a leading Mustache comment dmx's to read.
const MARKER: &str = "dmx";

/// The spelling a reader copies when their template metadata is refused.
const EXAMPLE: &str = "{{! dmx output=lib/models/shipping.dart }}";

/// Generates every template bound to the definition file `path`, writing what
/// changed [typediagram.standalone].
///
/// `roots` is the scope this pass was asked to manage, and therefore the scope
/// stale outputs are collected from.
///
/// # Errors
///
/// Fails when the definition or one of its templates cannot be read, when
/// binding, resolution, rendering, validation, or path safety refuses the
/// work, or on I/O.
pub fn process(path: &Path, roots: &[PathBuf], opts: &Options) -> Result<Outcome> {
    let (definition, root, groups) = bind(path)?;
    run::generate(&definition, &root, &groups, roots, opts)
}

/// What `dmx explain` prints for a definition file [typediagram.execution].
///
/// # Errors
///
/// Fails for the same reasons generation would.
pub fn explain(path: &Path) -> Result<String> {
    let (definition, root, groups) = bind(path)?;
    run::report(&definition, &root, &groups)
}

/// Whether `path` is a typeDiagram definition file [typediagram.standalone].
#[must_use]
pub fn is_definition(path: &Path) -> bool {
    has_extension(path, DEFINITION_EXTENSION)
}

/// Whether `path` is a Mustache template file.
#[must_use]
pub fn is_template(path: &Path) -> bool {
    has_extension(path, TEMPLATE_EXTENSION)
}

/// The definition file `template` renders, when there is one
/// [typediagram.standalone].
///
/// A template belongs to the most specific definition beside it: with both
/// `shipping.td` and `shipping.wire.td` present, `shipping.wire.mustache`
/// renders the second, because a name that matches two definitions matches the
/// longer one first. That single rule is what binds a definition to its
/// templates and what tells the watcher which definition to re-run when a
/// template changes — one rule, so the two can never disagree.
#[must_use]
pub fn definition_of(template: &Path) -> Option<PathBuf> {
    if !is_template(template) {
        return None;
    }
    let directory = template.parent().unwrap_or_else(|| Path::new("."));
    let mut stem = template.file_stem()?.to_str()?;
    loop {
        let candidate = directory.join(format!("{stem}.{DEFINITION_EXTENSION}"));
        if candidate.is_file() {
            return Some(candidate);
        }
        stem = stem.rsplit_once('.')?.0;
    }
}

/// The definition's name, the root its outputs resolve against, and the one
/// group its templates form.
///
/// A definition always renders: with nothing beside it, it renders through the
/// canonical model template [typediagram.canonical], and a `<name>.mustache`
/// beside it replaces that one. Every other template beside it — the
/// `<name>.<something>.mustache` files — is an output of its own.
fn bind(path: &Path) -> Result<(String, PathBuf, Vec<Group>)> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("DMX1002: cannot read {}", path.display()))?;
    let workspace = std::env::current_dir().context("DMX1002: cannot resolve the workspace")?;
    let root = emit::output_root(&workspace, path);
    let definition = emit::document_name(&root, path);
    let files = templates_beside(path)?;
    let mut templates = if files.iter().any(|file| replaces_canonical(file, path)) {
        Vec::new()
    } else {
        vec![canonical(&output_name(path)?)?]
    };
    for file in &files {
        let text = fs::read_to_string(file)
            .with_context(|| format!("DMX1002: cannot read {}", file.display()))?;
        let source = emit::document_name(&root, file);
        let name = output_name(file)?;
        let fence = Fence {
            // A file's body starts on line one, so a position inside it is
            // already a position in the file the author is editing.
            ordinal: templates.len().saturating_add(1),
            line: 0,
            body: text,
        };
        let at = Metadata {
            located: format!("the template {source}"),
            example: EXAMPLE,
        };
        let declared = metadata(&fence.body).to_owned();
        templates.push(binding::in_file(&declared, fence, source, &name, &at)?);
    }
    let groups = vec![Group {
        origin: Origin::Files,
        ordinal: 1,
        definition: Fence {
            ordinal: 1,
            line: 0,
            body,
        },
        templates,
    }];
    Ok((definition, root, groups))
}

/// The binding a definition gets from the canonical model template
/// [typediagram.canonical].
///
/// It lands where the convention puts any unnamed output — the target's source
/// root, under the definition's own name — because that is the file a reader
/// looking for `shipping.td`'s Dart would open.
fn canonical(name: &str) -> Result<BoundTemplate> {
    let target = target::find(binding::DEFAULT_TARGET)?;
    Ok(BoundTemplate {
        fence: Fence {
            ordinal: 1,
            line: 0,
            body: target.canonical.to_owned(),
        },
        output: format!("{}/{name}.{}", target.source_root, target.extension),
        target: binding::DEFAULT_TARGET.to_owned(),
        source: Source::Canonical,
    })
}

/// Whether this template takes the canonical one's place — the one whose name
/// is the definition's own, with nothing between them [typediagram.canonical].
fn replaces_canonical(template: &Path, definition: &Path) -> bool {
    template.file_stem() == definition.file_stem()
}

/// Every template file bound to `definition`, in the order their names sort.
///
/// Sorted rather than however the filesystem happened to return them: the
/// order is the order outputs are rendered and reported in, and a generation
/// that depended on directory iteration order would not be reproducible.
fn templates_beside(definition: &Path) -> Result<Vec<PathBuf>> {
    let directory = definition.parent().unwrap_or_else(|| Path::new("."));
    let entries = fs::read_dir(directory)
        .with_context(|| format!("DMX1002: cannot read {}", directory.display()))?;
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("DMX1002: cannot read {}", directory.display()))?
            .path();
        if definition_of(&path).is_some_and(|found| same_file(&found, definition)) {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

/// The base name a template's output takes when the template names none.
///
/// `shipping.mustache` generates `shipping`, and `shipping.wire.mustache`
/// generates `shipping_wire`: the dot that separates a template from its
/// definition is a word boundary, and the result is spelled the way the
/// target's own sources are [context.helpers].
fn output_name(template: &Path) -> Result<String> {
    let Some(stem) = template.file_stem().and_then(std::ffi::OsStr::to_str) else {
        bail!(
            "DMX8005 [typediagram.standalone]: {} has no name an output could be derived from",
            template.display()
        );
    };
    Ok(casing::snake(&stem.replace('.', "_")))
}

/// The dmx settings a template's first line carries, or `""` when it carries
/// none [typediagram.standalone].
///
/// A leading `{{! … }}` is a Mustache comment: every engine renders it to
/// nothing, so a template that carries one is still an ordinary template. dmx
/// reads what is inside it and otherwise leaves it exactly where it is — the
/// text handed to the renderer is the whole file, comment included, which is
/// what keeps the digest on the output's marker line a digest of the file the
/// author actually edits.
///
/// The settings are `key=value` rather than the JSON a fence's info string
/// carries, and that is not a style choice: a Mustache comment cannot contain
/// a `}` at all — the engine reads the first one as the start of the closing
/// braces and refuses the template — so an object could never survive here.
/// What the keys mean is decided in one place either way [typediagram.binding].
fn metadata(body: &str) -> &str {
    let first = body.lines().next().unwrap_or_default().trim();
    let Some(inside) = first
        .strip_prefix("{{!")
        .and_then(|rest| rest.strip_suffix("}}"))
    else {
        return "";
    };
    match inside.trim().split_once(char::is_whitespace) {
        Some((MARKER, settings)) => settings.trim(),
        // Somebody else's comment, or `{{! dmx }}` with nothing after it.
        _ => "",
    }
}

/// Whether two paths name one file, however each of them was spelled.
fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

/// Whether `path` carries `extension`, however it is cased.
fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

// A separate file only because standalone.rs is near the 500-line ceiling.
#[cfg(test)]
#[path = "standalone_tests.rs"]
mod tests;
