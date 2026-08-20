//! One Markdown document through the whole pipeline
//! [typediagram.execution].
//!
//! Bind → resolve → invoke the built-in macro → check the paths → emit. The
//! document itself is never rewritten: it is the source of truth, and dmx only
//! ever reads it [typediagram.output].
//!
//! `explain` walks the same path and stops before emission, printing what the
//! templates will actually see. It is the template author's only tool, so it
//! prints the exact context rather than a summary of it.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use serde_json::json;

use super::{Invocation, context, emit, markdown, resolve, target};
use crate::{Options, Outcome, macros};

/// Everything one document produced, resolved onto real paths.
struct Rendered {
    /// Each output's absolute path and complete text.
    outputs: Vec<(PathBuf, String)>,
}

/// Generates every group in `path`, writing what changed
/// [typediagram.execution].
///
/// `roots` is the scope this pass was asked to manage, and therefore the scope
/// stale outputs are collected from: an output that a removed template used to
/// produce is found by its ownership marker among the files dmx already walks.
///
/// # Errors
///
/// Fails when the document cannot be read, when binding, resolution, rendering,
/// validation, or path safety refuses it, or on I/O.
pub fn process(path: &Path, roots: &[PathBuf], opts: &Options) -> Result<Outcome> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("DMX1002: cannot read {}", path.display()))?;
    let workspace = std::env::current_dir().context("DMX1002: cannot resolve the workspace")?;
    let root = emit::output_root(&workspace, path);
    let document = emit::document_name(&root, path);
    let rendered = render(&document, &root, &source)?;
    let candidates = crate::watch::collect_outputs(roots)?;
    let changed = emit::emit(&document, &root, &rendered.outputs, &candidates, opts.check)?;
    Ok(if changed {
        Outcome::Updated
    } else {
        Outcome::Unchanged
    })
}

/// Every output `source` declares, rendered and validated but not written.
fn render(document: &str, root: &Path, source: &str) -> Result<Rendered> {
    let groups = markdown::groups(source).with_context(|| format!("in {document}"))?;
    let mut outputs = Vec::new();
    for group in &groups {
        let model = resolve(document, group)?;
        let files = macros::expand_group(&Invocation {
            document,
            group,
            model: &model,
        })?;
        // The macro renders one file per bound template, in template order, so
        // a path fault can name the fence that declared it.
        for (template, file) in group.templates.iter().zip(files) {
            let located = || {
                format!(
                    "in {document}, the Mustache template on line {}",
                    template.fence.line
                )
            };
            emit::refuse_self_overwrite(document, &file.name).with_context(located)?;
            let path = emit::resolve_output(root, &file.name).with_context(located)?;
            outputs.push((path, file.text));
        }
    }
    Ok(Rendered { outputs })
}

/// What `dmx explain` prints for a Markdown document
/// [typediagram.execution].
///
/// Nothing is rendered and nothing is written: this is the input side of the
/// pipeline, laid out so a template author can see the names they may place
/// before they place them.
///
/// # Errors
///
/// Fails when the document cannot be read, or when binding or resolution
/// refuses it — the same failures generation would report.
pub fn explain(path: &Path) -> Result<String> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("DMX1002: cannot read {}", path.display()))?;
    let workspace = std::env::current_dir().context("DMX1002: cannot resolve the workspace")?;
    let root = emit::output_root(&workspace, path);
    let document = emit::document_name(&root, path);
    let groups = markdown::groups(&source).with_context(|| format!("in {document}"))?;
    let mut out = format!(
        "{document}: {} generation group(s), outputs under {}\n",
        groups.len(),
        root.display()
    );
    for group in &groups {
        let model = resolve(&document, group)?;
        writeln!(
            out,
            "\ngroup {} — typeDiagram fence {} on line {}, {} declaration(s), digest {}",
            group.ordinal,
            group.definition.ordinal,
            group.definition.line,
            model.decls().len(),
            super::digest(&group.definition.body),
        )
        .map_err(report_fault)?;
        for template in &group.templates {
            let target = target::find(&template.target)?;
            writeln!(
                out,
                "  -> {} (target {}, fence {} on line {}, digest {})",
                template.output,
                target.name,
                template.fence.ordinal,
                template.fence.line,
                super::digest(&template.fence.body),
            )
            .map_err(report_fault)?;
            let ctx = context::build(&document, group, template, &model, target)?;
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{explain, process};
    use crate::{Options, Outcome};

    /// A scratch workspace holding one document, with the process working
    /// directory pointed at it.
    ///
    /// The working directory is process-wide, so these tests run under one
    /// mutex rather than in parallel — the alternative is a `workspace` option
    /// nothing but the tests would ever set.
    fn in_workspace<T>(document: &str, body: impl FnOnce(&std::path::Path) -> T) -> T {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let directory = scratch();
        fs::create_dir_all(directory.join("docs")).expect("docs directory");
        fs::write(directory.join("docs").join("models.dmx.md"), document).expect("document");
        let previous = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&directory).expect("enter workspace");
        let outcome = body(&directory);
        std::env::set_current_dir(previous).expect("leave workspace");
        drop(fs::remove_dir_all(&directory));
        drop(guard);
        outcome
    }

    /// A directory nobody else holds.
    fn scratch() -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!("dmx-td-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).expect("scratch directory");
        path
    }

    /// The canonical worked document.
    const DOCUMENT: &str = r#"# Store

```typeDiagram
type Product {
  id: Uuid
  name: String
}
```

```mustache {"dmx":{"output":"lib/models.dart"}}
{{#declarations}}
final class {{name}} {
  const {{name}}({{{constructorParameters}}});
{{#fields}}
  final {{{dartType}}} {{name}};
{{/fields}}
}
{{/declarations}}
```
"#;

    /// The document's one path, and the pipeline options for a real build.
    fn build_options() -> Options {
        Options {
            insert_regions: false,
            check: false,
        }
    }

    /// [typediagram.execution]: a build writes the declared file, a second
    /// build writes nothing, and the document is never rewritten.
    #[test]
    fn a_build_is_idempotent_and_never_touches_the_document() {
        in_workspace(DOCUMENT, |directory| {
            let path = directory.join("docs").join("models.dmx.md");
            let roots = vec![std::path::PathBuf::from("lib")];
            assert_eq!(
                process(&path, &roots, &build_options()).expect("first build"),
                Outcome::Updated
            );
            let generated = fs::read_to_string(directory.join("lib").join("models.dart"))
                .expect("generated file");
            assert!(generated.contains("final class Product {"), "{generated}");
            assert!(generated.contains("const Product({required this.id, required this.name});"));
            assert!(generated.starts_with("// dmx: generated from docs/models.dmx.md"));

            assert_eq!(
                process(&path, &roots, &build_options()).expect("second build"),
                Outcome::Unchanged
            );
            assert_eq!(
                fs::read_to_string(&path).expect("document"),
                DOCUMENT,
                "the document is the source of truth and is never rewritten"
            );
        });
    }

    /// [typediagram.execution]: `--check` reports drift and writes nothing.
    #[test]
    fn check_reports_drift_without_writing() {
        in_workspace(DOCUMENT, |directory| {
            let path = directory.join("docs").join("models.dmx.md");
            let roots = vec![std::path::PathBuf::from("lib")];
            let check = Options {
                insert_regions: false,
                check: true,
            };
            assert_eq!(
                process(&path, &roots, &check).expect("check"),
                Outcome::Updated
            );
            assert!(!directory.join("lib").join("models.dart").exists());
        });
    }

    /// [typediagram.output]: an output that exists without dmx's marker is a
    /// hand-written file and is never overwritten.
    #[test]
    fn a_hand_written_output_is_refused() {
        in_workspace(DOCUMENT, |directory| {
            fs::create_dir_all(directory.join("lib")).expect("lib");
            fs::write(directory.join("lib").join("models.dart"), "// mine\n").expect("existing");
            let path = directory.join("docs").join("models.dmx.md");
            let error = format!(
                "{:#}",
                process(&path, &[std::path::PathBuf::from("lib")], &build_options())
                    .expect_err("hand-written file")
            );
            assert!(error.contains("DMX8006"), "{error}");
            assert_eq!(
                fs::read_to_string(directory.join("lib").join("models.dart")).expect("untouched"),
                "// mine\n"
            );
        });
    }

    /// [typediagram.output]: a removed template takes its output with it.
    #[test]
    fn a_removed_template_collects_its_output() {
        in_workspace(DOCUMENT, |directory| {
            let path = directory.join("docs").join("models.dmx.md");
            let roots = vec![std::path::PathBuf::from("lib")];
            let _ = process(&path, &roots, &build_options()).expect("first build");
            assert!(directory.join("lib").join("models.dart").exists());

            fs::write(&path, "# Store\n\nNothing to generate any more.\n").expect("rewrite");
            assert_eq!(
                process(&path, &roots, &build_options()).expect("second build"),
                Outcome::Updated
            );
            assert!(
                !directory.join("lib").join("models.dart").exists(),
                "a dropped template means a dropped file"
            );
        });
    }

    /// [typediagram.execution]: `explain` prints the groups, their paths,
    /// their digests, and the exact context — and writes nothing.
    #[test]
    fn explain_prints_the_context_without_generating() {
        in_workspace(DOCUMENT, |directory| {
            let path = directory.join("docs").join("models.dmx.md");
            let report = explain(&path).expect("explain");
            assert!(
                report.contains("docs/models.dmx.md: 1 generation group(s)"),
                "{report}"
            );
            assert!(
                report.contains("-> lib/models.dart (target dart, fence 2 on line 10"),
                "{report}"
            );
            assert!(report.contains("\"modelVersion\": 1"), "{report}");
            assert!(report.contains("\"dartType\": \"String\""), "{report}");
            assert!(!directory.join("lib").join("models.dart").exists());
        });
    }
}
