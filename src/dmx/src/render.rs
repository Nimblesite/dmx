//! Stage 5: render (spec §8) — mustache via ramhorns, plus the deterministic
//! normalizer (spec §11.5) that keeps template whitespace tidy.
//!
//! One function for every macro [catalogue]: a template is a string and a
//! context is a `Content`, so a new macro adds no rendering code at all.

use anyhow::{Context as _, Result};
use ramhorns::{Content, Template};
use serde_json::Value;
use std::cell::RefCell;

use crate::jsoncontent::Json;

std::thread_local! {
    /// A synchronous, call-scoped template override for the browser playground.
    static TEMPLATE_OVERRIDE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Runs one generation operation with `template` in place of the built-in
/// macro template [playground.wasm].
///
/// The override is thread-local and restored before this function returns, so
/// normal generation and concurrent native callers retain their own template.
///
/// # Errors
///
/// Returns the generation error produced by `operation`.
pub(crate) fn with_template<T>(template: &str, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    TEMPLATE_OVERRIDE.with(|slot| {
        let previous = slot.replace(Some(template.to_owned()));
        let result = operation();
        let _ = slot.replace(previous);
        result
    })
}

/// Renders one macro's template against its context [rendering].
///
/// # Errors
///
/// Fails when the built-in template does not compile, which is a bug in this
/// crate rather than anything the author did.
pub fn render<C: Content>(template: &str, ctx: &C) -> Result<String> {
    TEMPLATE_OVERRIDE.with(|slot| {
        let supplied = slot.borrow();
        let (source, diagnostic) = match supplied.as_deref() {
            Some(source) => (source, "DMX4000 [playground.wasm]: bad user template"),
            None => (template, "DMX4000: bad built-in template"),
        };
        let template = Template::new(strip_standalone(source)).context(diagnostic)?;
        Ok(normalize(&template.render(ctx)))
    })
}

/// Renders `template` against a model a macro worker computed
/// [dartmacros.render].
///
/// This is [`render`] with the context supplied as JSON instead of as a Rust
/// struct, so a macro written in Dart reaches the very engine, standalone-tag
/// handling, and normalizer the catalogue's own templates go through. The
/// playground's template override deliberately does not apply: it replaces one
/// inferred built-in's template [playground.wasm], and a project's macro
/// brought its own.
///
/// # Errors
///
/// Fails when `template` does not compile, naming the template the macro sent.
pub fn render_json(name: &str, template: &str, model: &Value) -> Result<String> {
    let compiled = Template::new(strip_standalone(template))
        .with_context(|| format!("DMX7009: macro template `{name}` does not compile"))?;
    Ok(normalize(&compiled.render(&Json(model))))
}

/// Mustache "standalone tag" semantics: a line holding nothing but a section
/// tag contributes no output line. Ramhorns keeps the newline, so strip it
/// before compiling.
fn strip_standalone(template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    for line in template.lines() {
        let t = line.trim();
        let inner = t
            .strip_prefix(['{'])
            .and_then(|rest| rest.strip_prefix('{'))
            .and_then(|rest| rest.strip_suffix("}}"));
        let is_standalone = ["{{#", "{{/", "{{^"].iter().any(|p| t.starts_with(p))
            && inner.is_some_and(|inner| !inner.contains("{{"));
        if is_standalone {
            out.push_str(t);
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Trims trailing whitespace, collapses runs of blank lines, and drops
/// leading/trailing blank lines — so templates can favor readability.
///
/// Collapsing runs is what lets [`crate::macros::expand`] join fragments with a
/// blank line and know the result is exactly one. User-macro fragments enter
/// the same normalizer [dartmacros.pipeline], so a Dart-authored fragment and
/// a built-in one obey identical whitespace law.
pub(crate) fn normalize(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() && out.last().is_none_or(|l| l.is_empty()) {
            continue;
        }
        out.push(line);
    }
    while out.last().is_some_and(|line| line.is_empty()) {
        out.truncate(out.len().saturating_sub(1));
    }
    out.join("\n")
}
