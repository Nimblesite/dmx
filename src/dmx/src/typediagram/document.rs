//! One Markdown document through the whole pipeline
//! [typediagram.execution].
//!
//! This is a front end and nothing else: read the file, bind its fences, and
//! hand the groups to [`super::run`], which is the pipeline both front ends
//! share. The document itself is never rewritten — it is the source of truth,
//! and dmx only ever reads it [typediagram.output].

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use super::binding::Group;
use super::{emit, markdown, run};
use crate::{Options, Outcome};

/// Generates every group in `path`, writing what changed
/// [typediagram.execution].
///
/// `roots` is the scope this pass was asked to manage, and therefore the scope
/// stale outputs are collected from.
///
/// # Errors
///
/// Fails when the document cannot be read, when binding, resolution,
/// rendering, validation, or path safety refuses it, or on I/O.
pub fn process(path: &Path, roots: &[PathBuf], opts: &Options) -> Result<Outcome> {
    let (document, root, groups) = bind(path)?;
    run::generate(&document, &root, &groups, roots, opts)
}

/// What `dmx explain` prints for a Markdown document [typediagram.execution].
///
/// # Errors
///
/// Fails when the document cannot be read, or when binding or resolution
/// refuses it — the same failures generation would report.
pub fn explain(path: &Path) -> Result<String> {
    let (document, root, groups) = bind(path)?;
    run::report(&document, &root, &groups)
}

/// The document's name, the root its outputs resolve against, and its groups.
fn bind(path: &Path) -> Result<(String, PathBuf, Vec<Group>)> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("DMX1002: cannot read {}", path.display()))?;
    let workspace = std::env::current_dir().context("DMX1002: cannot resolve the workspace")?;
    let root = emit::output_root(&workspace, path);
    let document = emit::document_name(&root, path);
    let groups = markdown::groups(&source).with_context(|| format!("in {document}"))?;
    Ok((document, root, groups))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{explain, process};
    use crate::{Options, Outcome};

    /// The canonical worked document, in a workspace of its own.
    fn in_workspace<T>(document: &str, body: impl FnOnce(&std::path::Path) -> T) -> T {
        crate::typediagram::scratch::in_workspace(&[("docs/models.dmx.md", document)], body)
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
