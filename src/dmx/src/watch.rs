//! Debounced, incremental file watching for `dmx watch` [execution.modes].
//!
//! What counts as a source, and where one is found, is [`crate::sources`].
//! This module is the loop: register the scopes, batch the events a burst
//! produces, decide what each event stands for, and regenerate.

use anyhow::{Context as _, Result, bail};
use lspkit::{EngineApi as _, Progress, RescanScope};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::Options;
use crate::engine::{Engine, FileOutcome, Pass, Query};
use crate::sources::{Scope, Sweep, collect_path};

/// How long a save burst is allowed to keep arriving before it is answered.
const DEBOUNCE: Duration = Duration::from_millis(150);

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
///
/// A Mustache template stands for its definition and NOT for itself: nothing
/// is ever generated from a `.mustache` file, so a pass that named one would
/// report writing a file it did not write [typediagram.standalone].
fn claim(path: &Path, scopes: &[Scope]) -> Batch {
    let named = BTreeSet::from([path.to_owned()]);
    match (
        scopes.iter().any(|scope| scope.accepts(path)),
        scopes.iter().any(|scope| scope.covers_directory(path)),
    ) {
        (true, _) => Batch {
            sources: match crate::typediagram::definition_of(path) {
                Some(definition) => BTreeSet::from([definition]),
                None => named
                    .into_iter()
                    .chain(crate::emit::seed_of(path))
                    .collect(),
            },
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
        .watches(path)
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

// A separate file only because the loop and its tests together are long.
#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
