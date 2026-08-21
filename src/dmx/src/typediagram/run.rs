//! Every generation group through the pipeline, however it was bound
//! [typediagram.execution].
//!
//! Resolve → invoke the built-in macro → check the paths → emit. The sources
//! are never rewritten: the definition and the template are the truth, and dmx
//! only ever reads them [typediagram.output].
//!
//! [`report`] walks the same path and stops before emission, printing what the
//! templates will actually see. It is the template author's only tool, so it
//! prints the exact context rather than a summary of it.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde_json::json;

use super::binding::{self, Group};
use super::{Invocation, context, emit, resolve, target};
use crate::{Options, Outcome, macros};

/// Generates every group `document` declares, writing what changed
/// [typediagram.execution].
///
/// `roots` is the scope this pass was asked to manage, and therefore the scope
/// stale outputs are collected from: an output that a removed template used to
/// produce is found by its ownership marker among the files dmx already walks.
///
/// # Errors
///
/// Fails when binding, resolution, rendering, validation, or path safety
/// refuses the work, or on I/O.
pub fn generate(
    document: &str,
    root: &Path,
    groups: &[Group],
    roots: &[PathBuf],
    opts: &Options,
) -> Result<Outcome> {
    binding::refuse_duplicate_outputs(document, groups)?;
    let outputs = render(document, root, groups)?;
    let candidates = crate::sources::collect_outputs(roots)?;
    let changed = emit::emit(document, root, &outputs, &candidates, opts.check)?;
    Ok(if changed {
        Outcome::Updated
    } else {
        Outcome::Unchanged
    })
}

/// Every output these groups declare, rendered and validated but not written.
fn render(document: &str, root: &Path, groups: &[Group]) -> Result<Vec<(PathBuf, String)>> {
    let mut outputs = Vec::new();
    for group in groups {
        let model = resolve(document, group)?;
        let files = macros::expand_group(&Invocation {
            document,
            group,
            model: &model,
        })?;
        // The macro renders one file per bound template, in binding order, so
        // a path fault can name the template that declared it.
        for (template, file) in group.templates.iter().zip(files) {
            let located = || format!("in {}", template.located(document));
            emit::refuse_self_overwrite(document, &file.name).with_context(located)?;
            let path = emit::resolve_output(root, &file.name).with_context(located)?;
            outputs.push((path, file.text));
        }
    }
    Ok(outputs)
}

/// What `dmx explain` prints for one set of groups [typediagram.execution].
///
/// Nothing is rendered and nothing is written: this is the input side of the
/// pipeline, laid out so a template author can see the names they may place
/// before they place them.
///
/// # Errors
///
/// Fails when binding or resolution refuses the work — the same failures
/// generation would report.
pub fn report(document: &str, root: &Path, groups: &[Group]) -> Result<String> {
    binding::refuse_duplicate_outputs(document, groups)?;
    let mut out = format!(
        "{document}: {} generation group(s), outputs under {}\n",
        groups.len(),
        root.display()
    );
    for group in groups {
        let model = resolve(document, group)?;
        writeln!(
            out,
            "\ngroup {} — {}, {} declaration(s), digest {}",
            group.ordinal,
            group.heading(),
            model.decls().len(),
            super::digest(&group.definition.body),
        )
        .map_err(report_fault)?;
        for template in &group.templates {
            let target = target::find(&template.target)?;
            writeln!(
                out,
                "  -> {} (target {}, {}, digest {})",
                template.output,
                target.name,
                template.heading(),
                super::digest(&template.fence.body),
            )
            .map_err(report_fault)?;
            let ctx = context::build(document, group, template, &model, target)?;
            writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&json!({ "context": ctx }))
                    .context("DMX2000: internal error — the context is not serializable")?
            )
            .map_err(report_fault)?;
        }
    }
    Ok(out)
}

/// A `String` that cannot be written to is not a condition this program can
/// act on, and saying so is better than a panic that says less.
fn report_fault(error: std::fmt::Error) -> anyhow::Error {
    anyhow::anyhow!("DMX2000: internal error — cannot format the explain report: {error}")
}
