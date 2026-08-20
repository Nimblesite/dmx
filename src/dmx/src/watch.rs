//! Debounced, incremental file watching for `dmx watch` [execution.modes].

use anyhow::{Context as _, Result, bail};
use lspkit::{EngineApi as _, Progress, RescanScope};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::Options;
use crate::engine::{Engine, FileOutcome, Pass, Query};

/// How long a save burst is allowed to keep arriving before it is answered.
const DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Clone, Debug, Eq, PartialEq)]
/// What one watch argument turned out to be.
enum Scope {
    /// A directory, watched recursively.
    Directory(PathBuf),
    /// One Dart source, watched through its parent directory.
    File(PathBuf),
}

impl Scope {
    /// Resolves one command-line path, refusing what cannot be watched.
    fn from_path(path: &Path) -> Result<Self> {
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
    fn contains(&self, path: &Path) -> bool {
        match self {
            Self::File(file) => path == file,
            Self::Directory(directory) => path
                .strip_prefix(directory)
                .is_ok_and(|relative| relative.components().all(visible_component)),
        }
    }

    /// Whether an event about this path is one this scope wants.
    fn accepts(&self, path: &Path) -> bool {
        let named = matches!(self, Self::File(file) if file == path);
        // Recursive discovery takes `*.dmx.md`; a Markdown file named directly
        // is watched whatever it is called [typediagram.documents].
        let wanted = if named {
            Sweep::Sources.wants_named(path)
        } else {
            Sweep::Sources.wants(path)
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
    fn covers_directory(&self, path: &Path) -> bool {
        matches!(self, Self::Directory(_))
            && !path.is_symlink()
            && path.is_dir()
            && self.contains(path)
    }

    /// The canonical path this scope covers, which is what the engine rescans.
    fn root(&self) -> PathBuf {
        match self {
            Self::Directory(path) | Self::File(path) => path.clone(),
        }
    }

    /// The path to register with the watcher, and how deeply.
    fn registration(&self) -> Result<(PathBuf, RecursiveMode)> {
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
enum Sweep {
    /// Everything dmx generates from: Dart files and Markdown documents.
    Sources,
    /// Anything carrying an extension some generation target writes — the
    /// candidates a generated output could be hiding among when a pass
    /// collects what it no longer produces [typediagram.output].
    Outputs,
}

impl Sweep {
    /// Whether a file *recursive discovery* found is one this sweep wants.
    fn wants(self, path: &Path) -> bool {
        match self {
            Self::Sources => is_dart_source(path) || crate::typediagram::is_document(path),
            Self::Outputs => crate::typediagram::target::extensions()
                .any(|extension| has_extension(path, extension)),
        }
    }

    /// Whether a file *named directly* is one this sweep wants.
    ///
    /// The two differ in exactly one place: recursive discovery takes
    /// `*.dmx.md` and nothing else, and naming a Markdown file is how any
    /// other one is generated from [typediagram.documents].
    fn wants_named(self, path: &Path) -> bool {
        self.wants(path) || (self == Self::Sources && crate::typediagram::is_markdown(path))
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
fn collect_path(
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

/// Runs the debounced, incremental watch execution mode [execution.modes], [cli].
///
/// # Errors
///
/// Fails when a path cannot be watched, when the watcher loses coverage, or
/// when a generated file cannot be written.
pub fn run(paths: &[PathBuf], opts: &Options) -> Result<()> {
    let scopes = paths
        .iter()
        .map(|path| Scope::from_path(path))
        .collect::<Result<Vec<_>>>()?;
    // The pipeline runs behind `lspkit::EngineApi` [engine.api]; the watcher
    // owns the only thread that drives it, so a current-thread runtime is the
    // whole of the async machinery this binary needs.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .context("DMX1005 [engine.api]: cannot start the engine runtime")?;
    let engine = Engine::new(scopes.iter().map(Scope::root).collect(), *opts);
    let (sender, receiver) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            // The receiver only disappears while the watch command is ending.
            drop(sender.send(event));
        },
        Config::default().with_follow_symlinks(false),
    )
    .context("DMX1004 [execution.modes]: cannot create file watcher")?;

    for (path, mode) in registrations(&scopes)? {
        watcher.watch(&path, mode).with_context(|| {
            format!("DMX1004 [execution.modes]: cannot watch {}", path.display())
        })?;
    }

    regenerate(&runtime, &engine, RescanScope::All)?;

    println!("dmx: watching {} path(s)", scopes.len());
    io::stdout()
        .flush()
        .context("DMX1004 [execution.modes]: cannot flush watcher readiness")?;

    loop {
        let batch = receive_batch(&receiver, &scopes)?;
        regenerate(
            &runtime,
            &engine,
            RescanScope::Paths(batch.into_iter().collect()),
        )?;
    }
}

/// The watcher registrations the scopes add up to, deduplicated.
fn registrations(scopes: &[Scope]) -> Result<Vec<(PathBuf, RecursiveMode)>> {
    scopes
        .iter()
        .map(Scope::registration)
        .collect::<Result<Vec<_>>>()
        .map(|items| {
            items
                .into_iter()
                .fold(BTreeMap::new(), |mut registrations, (path, mode)| {
                    // One directory watched both ways is watched recursively:
                    // the wider registration already covers the narrower one.
                    let current = registrations.entry(path).or_insert(mode);
                    if mode == RecursiveMode::Recursive {
                        *current = mode;
                    }
                    registrations
                })
                .into_iter()
                .collect()
        })
}

/// One batch under construction.
///
/// `trees` is the reason this is a struct rather than a set of paths: a
/// directory has to be read AFTER the burst goes quiet, never when its event
/// arrives. The event for a new directory routinely beats the write of the file
/// inside it, so reading on arrival finds an empty directory and answers a
/// question nobody asked [execution.modes].
#[derive(Default)]
struct Batch {
    /// Sources named outright, which stand for themselves.
    sources: BTreeSet<PathBuf>,
    /// Paths to re-read once the burst settles.
    trees: BTreeSet<PathBuf>,
    /// Sources missing when their event arrived, each against the watched
    /// directory to re-read if they are still missing once the burst is over.
    /// Every rename — an editor saving, this watcher's own atomic write — has
    /// a moment where the destination is gone, so answering now costs a tree.
    vanished: BTreeMap<PathBuf, PathBuf>,
}

impl Batch {
    /// Nothing this watcher generates from changed.
    fn is_empty(&self) -> bool {
        self.sources.is_empty() && self.trees.is_empty() && self.vanished.is_empty()
    }

    /// Absorbs another batch.
    fn absorb(&mut self, other: Self) {
        self.sources.extend(other.sources);
        self.trees.extend(other.trees);
        self.vanished.extend(other.vanished);
    }

    /// The sources this batch stands for, read now that the burst is over.
    ///
    /// A file that is back is the rename it always was. One still missing was
    /// deleted, and its directory is re-read instead: a generated file names
    /// its seed nowhere any more, and re-running that seed writes it again.
    ///
    /// # Errors
    ///
    /// Fails when a directory that is still there cannot be read.
    fn resolve(mut self) -> Result<BTreeSet<PathBuf>> {
        let (back, gone): (BTreeMap<_, _>, BTreeMap<_, _>) = self
            .vanished
            .into_iter()
            .partition(|(path, _)| path.is_file());
        self.sources.extend(back.into_keys());
        self.trees.extend(gone.into_values());
        self.trees
            .iter()
            .filter(|path| path.exists())
            .map(|path| collect_path(path, Sweep::Sources, Sweep::wants))
            .collect::<Result<Vec<_>>>()
            .map(|groups| {
                groups
                    .into_iter()
                    .flatten()
                    .chain(self.sources)
                    .collect::<BTreeSet<_>>()
            })
    }
}

/// One debounce window on from `start`, or `start` itself where the clock
/// cannot represent it — a deadline in the past just answers sooner.
fn debounce_from(start: Instant) -> Instant {
    start.checked_add(DEBOUNCE).unwrap_or(start)
}

/// The next batch of changed paths, once the burst has settled.
fn receive_batch(
    receiver: &Receiver<notify::Result<Event>>,
    scopes: &[Scope],
) -> Result<BTreeSet<PathBuf>> {
    receive_relevant(receiver, scopes)
        .and_then(|first| receive_until(receiver, scopes, first))
        .and_then(Batch::resolve)
}

/// Blocks until an event names at least one path a scope wants.
fn receive_relevant(receiver: &Receiver<notify::Result<Event>>, scopes: &[Scope]) -> Result<Batch> {
    loop {
        let event = receiver
            .recv()
            .map_err(|_| anyhow::anyhow!("DMX1004 [execution.modes]: file watcher disconnected"))?;
        let batch = batch_from_event(event, scopes)?;
        if !batch.is_empty() {
            return Ok(batch);
        }
    }
}

/// Keeps extending the batch until the burst goes quiet.
fn receive_until(
    receiver: &Receiver<notify::Result<Event>>,
    scopes: &[Scope],
    initial: Batch,
) -> Result<Batch> {
    let mut pending = initial;
    let mut deadline = debounce_from(Instant::now());
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Ok(pending);
        }
        match receiver.recv_timeout(deadline.saturating_duration_since(now)) {
            Ok(event) => {
                let batch = batch_from_event(event, scopes)?;
                if !batch.is_empty() {
                    pending.absorb(batch);
                    deadline = debounce_from(Instant::now());
                }
            }
            Err(RecvTimeoutError::Timeout) => return Ok(pending),
            Err(RecvTimeoutError::Disconnected) => {
                bail!("DMX1004 [execution.modes]: file watcher disconnected")
            }
        }
    }
}

/// What one event is about, filtered to what the scopes want.
fn batch_from_event(event: notify::Result<Event>, scopes: &[Scope]) -> Result<Batch> {
    match event {
        // An overflowed watcher says nothing about WHICH paths it dropped, so
        // every scope is re-read — after the burst, like any other tree.
        Ok(event) if event.need_rescan() => Ok(Batch {
            trees: scopes.iter().map(Scope::root).collect(),
            ..Batch::default()
        }),
        Ok(event) if actionable(event.kind) => Ok(event
            .paths
            .into_iter()
            .filter(|path| !path.is_symlink())
            .filter_map(|path| resolve(&path))
            .fold(Batch::default(), |mut batch, path| {
                batch.absorb(claim(&path, scopes));
                batch
            })),
        Ok(_) => Ok(Batch::default()),
        Err(error) => Err(anyhow::anyhow!(
            "DMX1004 [execution.modes]: file watcher lost coverage: {error}"
        )),
    }
}

/// One event path as an absolute path, even when what it names is gone.
///
/// A deleted file cannot be canonicalized, and dropping the event there is how
/// deleting a generated file used to be a change nobody answered. Its directory
/// is still there, so the path is rebuilt from that.
fn resolve(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().or_else(|| {
        let parent = path.parent()?.canonicalize().ok()?;
        Some(parent.join(path.file_name()?))
    })
}

/// What one event path stands for: itself, everything under it, or nothing.
///
/// A file a macro authored stands for the file that authored it: the marker on
/// its first line names that seed [dartmacros.files]. Editing generated code
/// re-runs what generates it, which is the only way an edit there can be
/// answered — the generated file has no annotation of its own.
fn claim(path: &Path, scopes: &[Scope]) -> Batch {
    let named = BTreeSet::from([path.to_owned()]);
    match (
        scopes.iter().any(|scope| scope.accepts(path)),
        scopes.iter().any(|scope| scope.covers_directory(path)),
    ) {
        (true, _) => Batch {
            sources: named
                .into_iter()
                .chain(crate::emit::seed_of(path))
                .collect(),
            ..Batch::default()
        },
        (false, true) => Batch {
            trees: named,
            ..Batch::default()
        },
        // Whatever it was, it is not there NOW — which a rename is too, for as
        // long as it takes to land. Held until the burst settles, where being
        // gone still means gone [dartmacros.files].
        (false, false) => Batch {
            vanished: vanished_in(path, scopes).into_iter().collect(),
            ..Batch::default()
        },
    }
}

/// A missing source against the watched directory it was in.
fn vanished_in(path: &Path, scopes: &[Scope]) -> Option<(PathBuf, PathBuf)> {
    let parent = Sweep::Sources
        .wants(path)
        .then(|| path.parent())
        .flatten()?;
    scopes
        .iter()
        .any(|scope| scope.covers_directory(parent))
        .then(|| (path.to_owned(), parent.to_owned()))
}

/// Whether an event kind can have changed a file dmx generates from.
///
/// Removal counts: a generated file someone deleted is out of date in the most
/// complete way there is [dartmacros.files].
fn actionable(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

/// Rescans `scope` through the engine and reports what it did [engine.api].
///
/// The pass is read back as a snapshot rather than returned by `rescan`, so the
/// generation the ticket promised and the generation the report carries can be
/// compared — that comparison is how any consumer of the engine, this watcher
/// included, tells a fresh answer from a superseded one.
fn regenerate(runtime: &Runtime, engine: &Engine, scope: RescanScope) -> Result<()> {
    let ticket = runtime.block_on(engine.rescan(scope, Progress::noop()))?;
    let snapshot = runtime.block_on(engine.report(Query::LastPass, CancellationToken::new()))?;
    if snapshot.generation == ticket.generation() {
        announce(&snapshot.data);
    }
    Ok(())
}

/// One line per file that changed, one per file that failed, silence for the
/// rest — the account a human keeps half an eye on while editing.
fn announce(pass: &Pass) {
    for (path, outcome) in &pass.files {
        match outcome {
            FileOutcome::Written => println!("wrote: {}", path.display()),
            FileOutcome::Unchanged => {}
            FileOutcome::Refused(message) => eprintln!("error: {}: {message}", path.display()),
        }
    }
}

// A separate file only because watch.rs is near the 500-line ceiling.
#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
