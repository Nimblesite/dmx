//! Whole-file emission for a Markdown generation group [typediagram.output].
//!
//! One thing happens here that the Dart-macro backend does not need: an output
//! path arrives from a *document* rather than from dmx, so it is checked before
//! anything is written — inside the workspace, no traversal, and no symbolic
//! link that leaves the tree. Whether the path is the right *kind* of file is
//! the target's question and is answered by the macro that named it.
//!
//! Everything after that is the shared protocol in [`crate::emit`]: never
//! overwrite an unmarked file, write atomically, skip a no-op, report drift
//! without writing under `--check`, and collect what a removed template used to
//! produce.

use std::path::{Component, Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::emit::{collect_stale, write_owned};

/// The absolute path `declared` resolves to under `workspace`
/// [typediagram.output].
///
/// # Errors
///
/// Fails (`DMX8005`) when the path is absolute, escapes the workspace, or
/// reaches through a symbolic link that leaves it.
pub fn resolve_output(workspace: &Path, declared: &str) -> Result<PathBuf> {
    let relative = Path::new(declared);
    let fault =
        |detail: &str| anyhow::anyhow!("DMX8005 [typediagram.output]: `{declared}` {detail}");
    for component in relative.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => {
                return Err(fault("leaves the workspace; `..` is never an output path"));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(fault("is an absolute path; outputs are workspace-relative"));
            }
        }
    }
    let resolved = workspace.join(relative);
    refuse_symlink_escape(workspace, &resolved).map_err(|detail| fault(&detail))?;
    Ok(resolved)
}

/// Refuses a path whose nearest existing ancestor resolves outside the root.
///
/// A directory in the middle of an output path may be a symbolic link; the
/// question is only ever whether following it still lands inside the tree dmx
/// was asked to manage. Canonicalizing the deepest ancestor that exists answers
/// exactly that, and a path whose directories do not exist yet cannot have been
/// redirected by one.
fn refuse_symlink_escape(workspace: &Path, resolved: &Path) -> Result<(), String> {
    let Ok(root) = workspace.canonicalize() else {
        return Ok(());
    };
    let existing = resolved
        .ancestors()
        .skip(1)
        .find(|ancestor| ancestor.exists())
        .unwrap_or(workspace);
    match existing.canonicalize() {
        Ok(real) if real.starts_with(&root) => Ok(()),
        Ok(real) => Err(format!(
            "reaches outside the workspace through {} -> {}",
            existing.display(),
            real.display()
        )),
        Err(_) => Ok(()),
    }
}

/// Refuses an output path a document may not claim at all.
///
/// # Errors
///
/// Fails when the declared path is the document itself, which would replace the
/// source of truth with its own output.
pub fn refuse_self_overwrite(document: &str, declared: &str) -> Result<()> {
    if Path::new(document) == Path::new(declared) {
        bail!(
            "DMX8005 [typediagram.output]: `{declared}` is the document itself; a group never \
             overwrites its own source"
        );
    }
    Ok(())
}

/// Writes every output this document produced, then collects what it no longer
/// produces [typediagram.output].
///
/// Returns whether anything changed — or, under `check`, would have.
///
/// # Errors
///
/// Fails when an output exists without dmx's marker (`DMX8006`), or on I/O.
pub fn emit(
    document: &str,
    root: &Path,
    outputs: &[(PathBuf, String)],
    candidates: &[PathBuf],
    check: bool,
) -> Result<bool> {
    let mut changed = false;
    for (path, content) in outputs {
        changed |= write_owned(path, content, check, "DMX8006", "[typediagram.output]")
            .with_context(|| {
                format!(
                    "DMX8006 [typediagram.output]: generating {} from {document}",
                    display_relative(root, path)
                )
            })?;
    }
    let kept: Vec<PathBuf> = outputs.iter().map(|(path, _)| path.clone()).collect();
    let marker = super::ownership_marker(document);
    Ok(collect_stale(candidates, &marker, &kept, check)? || changed)
}

/// A path as a reader of the document would write it: relative to the
/// workspace when it is inside one, with forward slashes either way.
#[must_use]
pub fn display_relative(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The directory a document's outputs are resolved against
/// [typediagram.output].
///
/// It is the nearest ancestor of the document that carries a project marker —
/// `pubspec.yaml` for Dart — bounded by the workspace, and the workspace
/// itself when there is none. That is what makes a document portable: `lib/a.dart`
/// means *this package's* `lib`, whether dmx was run from the package, from the
/// repository root, or from an editor that opened the whole tree. Resolving
/// against the working directory instead would make the same document generate
/// somewhere else depending on where it was run from.
#[must_use]
pub fn output_root(workspace: &Path, document: &Path) -> PathBuf {
    let workspace = resolved(workspace);
    let document = match document.canonicalize() {
        Ok(absolute) => absolute,
        Err(_) => workspace.join(document),
    };
    document
        .ancestors()
        .skip(1)
        .take_while(|ancestor| ancestor.starts_with(&workspace))
        .find(|ancestor| {
            super::target::project_markers().any(|marker| ancestor.join(marker).is_file())
        })
        .map_or(workspace, Path::to_owned)
}

/// A path in the one form two spellings of it agree on.
fn resolved(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_owned())
}

/// The name a document is known by — in its ownership markers, its
/// diagnostics, and its templates' contexts [typediagram.output].
///
/// It is the path relative to the root its outputs land in, so the same
/// document generates the same bytes however it was named — relatively,
/// absolutely, or through a symbolic link. Resolving both sides is what makes
/// that true: a directory reached through `/tmp` and one reached through
/// `/private/tmp` are the same directory, and an output that recorded the
/// difference would rewrite itself every time somebody ran dmx the other way.
#[must_use]
pub fn document_name(root: &Path, path: &Path) -> String {
    let root = resolved(root);
    let absolute = path.canonicalize().unwrap_or_else(|_| root.join(path));
    display_relative(&root, &absolute)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{display_relative, refuse_self_overwrite, resolve_output};

    /// [typediagram.output]: an output path is workspace-relative and does not
    /// traverse out of the tree.
    #[test]
    fn unsafe_output_paths_are_refused() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"));
        for (declared, detail) in [
            ("/etc/passwd.dart", "absolute path"),
            ("../outside/a.dart", "leaves the workspace"),
            ("lib/../../a.dart", "leaves the workspace"),
        ] {
            let error = format!(
                "{:#}",
                resolve_output(workspace, declared).expect_err(declared)
            );
            assert!(error.contains("DMX8005"), "{declared}: {error}");
            assert!(error.contains(detail), "{declared}: {error}");
        }
        assert_eq!(
            resolve_output(workspace, "lib/models/a.dart").expect("safe path"),
            workspace.join("lib").join("models").join("a.dart")
        );
        assert_eq!(
            resolve_output(workspace, "./lib/a.dart").expect("safe path"),
            workspace.join("lib").join("a.dart")
        );
    }

    /// [typediagram.output]: a group never writes over the document it was
    /// read from.
    #[test]
    fn a_document_is_never_its_own_output() {
        refuse_self_overwrite("docs/a.dmx.md", "lib/a.dart").expect("a different file");
        let error = format!(
            "{:#}",
            refuse_self_overwrite("docs/a.dmx.md", "docs/a.dmx.md").expect_err("self overwrite")
        );
        assert!(error.contains("DMX8005"), "{error}");
    }

    /// [typediagram.output]: a path relative to the workspace is what a reader
    /// of the document sees, whatever the platform separator is.
    #[test]
    fn paths_are_reported_the_way_the_document_writes_them() {
        let workspace = PathBuf::from("/work/space");
        assert_eq!(
            display_relative(&workspace, &workspace.join("lib").join("a.dart")),
            "lib/a.dart"
        );
        assert_eq!(
            display_relative(&workspace, Path::new("docs/a.dmx.md")),
            "docs/a.dmx.md"
        );
    }
}
