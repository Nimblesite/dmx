//! What dmx generates from, and where it looks for it
//! [surface.zero-config], [typediagram.standalone], [typediagram.documents].
//!
//! One place decides three questions that have to agree: which files a
//! recursive sweep discovers, which files a watch answers an event about, and
//! which files could be an output a pass no longer produces. They are not the
//! same set — a `.mustache` template is watched but never discovered, and a
//! Markdown file is discovered only when it is named — and a copy of any one
//! of them that drifted would show up as a generator that had quietly stopped
//! working.

use anyhow::{Context as _, Result, bail};
use notify::RecursiveMode;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
/// What one watch argument turned out to be.
pub(crate) enum Scope {
    /// A directory, watched recursively.
    Directory(PathBuf),
    /// One Dart source, watched through its parent directory.
    File(PathBuf),
}

impl Scope {
    /// Resolves one command-line path, refusing what cannot be watched.
    pub(crate) fn from_path(path: &Path) -> Result<Self> {
        let absolute = path
            .canonicalize()
            .with_context(|| format!("DMX1002 [cli]: cannot watch {}", path.display()))?;
        match (absolute.is_dir(), absolute.is_file()) {
            (true, false) => Ok(Self::Directory(absolute)),
            (false, true) if Sweep::Sources.wants_named(&absolute) => Ok(Self::File(absolute)),
            (false, true) => bail!(
                "DMX1002 [cli]: watch target is not a Dart source or a Markdown document: {}",
                path.display()
            ),
            _ => bail!(
                "DMX1002 [cli]: watch target is not a file or directory: {}",
                path.display()
            ),
        }
    }

    /// Whether this scope's tree contains `path`, by name alone.
    ///
    /// Nothing here touches the filesystem: it answers where a path sits, and
    /// the callers below add what it has to BE.
    pub(crate) fn contains(&self, path: &Path) -> bool {
        match self {
            Self::File(file) => path == file,
            Self::Directory(directory) => path
                .strip_prefix(directory)
                .is_ok_and(|relative| relative.components().all(visible_component)),
        }
    }

    /// Whether an event about this path is one this scope wants.
    pub(crate) fn accepts(&self, path: &Path) -> bool {
        let named = matches!(self, Self::File(file) if file == path);
        // Recursive discovery takes `*.dmx.md`; a Markdown file named directly
        // is watched whatever it is called [typediagram.documents].
        let wanted = if named {
            Sweep::Sources.wants_named(path)
        } else {
            Sweep::Sources.watches(path)
        };
        !path.is_symlink() && path.is_file() && wanted && self.contains(path)
    }

    /// Whether `path` is a directory inside this scope's tree.
    ///
    /// A directory that appears inside a watched tree can already hold sources
    /// whose own creation events never arrive. A recursive watch on Linux is
    /// one inotify registration per directory, added when the directory is
    /// seen, so anything written into a new directory before that registration
    /// lands is never announced. macOS reports a whole tree from a single
    /// registration and never shows this, which is why it has to be handled
    /// here rather than left to whichever platform notices first
    /// [execution.modes].
    pub(crate) fn covers_directory(&self, path: &Path) -> bool {
        matches!(self, Self::Directory(_))
            && !path.is_symlink()
            && path.is_dir()
            && self.contains(path)
    }

    /// The canonical path this scope covers, which is what the engine rescans.
    pub(crate) fn root(&self) -> PathBuf {
        match self {
            Self::Directory(path) | Self::File(path) => path.clone(),
        }
    }

    /// The path to register with the watcher, and how deeply.
    pub(crate) fn registration(&self) -> Result<(PathBuf, RecursiveMode)> {
        match self {
            Self::Directory(path) => Ok((path.clone(), RecursiveMode::Recursive)),
            Self::File(path) => path
                .parent()
                .map(|parent| (parent.to_owned(), RecursiveMode::NonRecursive))
                .ok_or_else(|| {
                    anyhow::anyhow!("DMX1002 [cli]: {} has no parent directory", path.display())
                }),
        }
    }
}

/// What one sweep of the tree is looking for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Sweep {
    /// Everything dmx generates from: Dart files and Markdown documents.
    Sources,
    /// Anything carrying an extension some generation target writes — the
    /// candidates a generated output could be hiding among when a pass
    /// collects what it no longer produces [typediagram.output].
    Outputs,
}

impl Sweep {
    /// Whether a file *recursive discovery* found is one this sweep wants.
    pub(crate) fn wants(self, path: &Path) -> bool {
        match self {
            Self::Sources => {
                is_dart_source(path)
                    || crate::typediagram::is_document(path)
                    || crate::typediagram::is_definition(path)
            }
            Self::Outputs => crate::typediagram::target::extensions()
                .any(|extension| has_extension(path, extension)),
        }
    }

    /// Whether an event about `path` is one a watch answers.
    ///
    /// Wider than what a sweep discovers, by exactly one kind of file: a
    /// `.mustache` template is never a source in its own right — nothing is
    /// generated *from* it — but editing one changes what the definition
    /// beside it generates, so a watch that ignored it would go quiet on half
    /// the edits a template author makes [typediagram.standalone].
    pub(crate) fn watches(self, path: &Path) -> bool {
        self.wants(path) || (self == Self::Sources && crate::typediagram::is_template(path))
    }

    /// Whether a file *named directly* is one this sweep wants.
    ///
    /// Wider again, by one more kind: recursive discovery takes `*.dmx.md` and
    /// nothing else, and naming a Markdown file is how any other one is
    /// generated from [typediagram.documents].
    pub(crate) fn wants_named(self, path: &Path) -> bool {
        self.watches(path) || (self == Self::Sources && crate::typediagram::is_markdown(path))
    }
}

/// Every source dmx generates from at or under `paths` — Dart files and
/// Markdown documents alike [surface.zero-config], [typediagram.documents].
///
/// # Errors
///
/// Fails when a directory cannot be read.
pub fn collect_sources(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    collect(paths, Sweep::Sources)
}

/// Every file at or under `paths` that some generation target could have
/// written [typediagram.output].
///
/// # Errors
///
/// Fails when a directory cannot be read.
pub fn collect_outputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    collect(paths, Sweep::Outputs)
}

/// Every file `sweep` accepts at or under `paths`, deduplicated and ordered.
fn collect(paths: &[PathBuf], sweep: Sweep) -> Result<Vec<PathBuf>> {
    paths
        .iter()
        .map(|path| collect_path(path, sweep, Sweep::wants_named))
        .collect::<Result<Vec<_>>>()
        .map(|groups| {
            groups
                .into_iter()
                .flatten()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        })
}

/// Every source at or under one path, with `accept` deciding what a *file*
/// there has to be — which differs between a path somebody named and one
/// discovery walked into.
pub(crate) fn collect_path(
    path: &Path,
    sweep: Sweep,
    accept: fn(Sweep, &Path) -> bool,
) -> Result<Vec<PathBuf>> {
    match (path.is_symlink(), path.is_dir(), path.is_file()) {
        (false, true, _) => collect_directory(path, sweep),
        (false, false, true) if accept(sweep, path) => Ok(vec![path.to_owned()]),
        // A symlink is never followed [surface.zero-config], and anything that
        // is not a source is not dmx's to read.
        _ => Ok(Vec::new()),
    }
}

/// Every source under one directory, hidden entries excluded.
fn collect_directory(directory: &Path, sweep: Sweep) -> Result<Vec<PathBuf>> {
    std::fs::read_dir(directory)
        .with_context(|| {
            format!(
                "DMX1002 [surface.zero-config]: cannot read {}",
                directory.display()
            )
        })?
        .filter_map(|entry| match entry {
            Ok(entry) if visible_name(&entry.file_name()) => {
                Some(collect_path(&entry.path(), sweep, Sweep::wants))
            }
            Ok(_) => None,
            Err(error) => Some(Err(anyhow::Error::from(error).context(format!(
                "DMX1002 [surface.zero-config]: cannot inspect {}",
                directory.display()
            )))),
        })
        .collect::<Result<Vec<_>>>()
        .map(|groups| groups.into_iter().flatten().collect())
}

/// A Dart source dmx owns — not a `.g.dart` somebody else generates.
fn is_dart_source(path: &Path) -> bool {
    has_extension(path, "dart")
        && path
            .file_name()
            .is_some_and(|name| !name.to_string_lossy().ends_with(".g.dart"))
}

/// Whether `path` carries `extension`, however it is cased.
fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

/// Whether a directory entry is one the zero-config rules look at.
fn visible_name(name: &OsStr) -> bool {
    !name.to_string_lossy().starts_with('.')
}

/// The same rule, applied to one component of a relative path.
fn visible_component(component: Component<'_>) -> bool {
    match component {
        Component::Normal(name) => visible_name(name),
        _ => true,
    }
}
