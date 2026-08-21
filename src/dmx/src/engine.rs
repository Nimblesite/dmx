//! Live generation state behind the `lspkit` engine contract [engine].
//!
//! `dmx watch` no longer calls the pipeline directly. It asks an [`Engine`] to
//! rescan a scope and reads the outcome back as a generation-tagged snapshot —
//! the same `lspkit::EngineApi` contract an LSP server or an MCP adapter
//! consumes, so the editor integration and the CLI watch the identical state
//! [engine.api]. The watcher is simply its first consumer.

use async_trait::async_trait;
use lspkit::{
    Cause, EngineApi, Generation, GenerationEvent, GenerationEventStream, Progress, RescanScope,
    RescanTicket, Snapshot,
};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt as _};
use tokio_util::sync::CancellationToken;

use crate::sources::collect_sources;
use crate::{Options, Outcome, process_path};

/// Generation events a slow subscriber may fall behind by before it starts
/// missing them. A missed event costs a subscriber one redundant re-query, not
/// correctness: the generation it eventually observes is still monotonic.
const EVENT_BACKLOG: usize = 64;

/// What the pipeline did to one file during a pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileOutcome {
    /// The file was rewritten — or, under `check`, would have been.
    Written,
    /// Already up to date [emission.inline-backend.no-op-writes].
    Unchanged,
    /// The pipeline refused the file, carrying the diagnostic for the author.
    Refused(String),
}

/// Everything one rescan learned, in deterministic path order.
#[derive(Clone, Debug, Default)]
pub struct Pass {
    /// Every file the pass examined, including the ones it left alone.
    pub files: Vec<(PathBuf, FileOutcome)>,
}

/// What a consumer can ask the engine for [engine.api].
#[derive(Clone, Debug)]
pub enum Query {
    /// Every outcome recorded by the most recent rescan.
    LastPass,
}

/// Engine-level failures. Per-file failures are not errors: they are recorded
/// as [`FileOutcome::Refused`] so one bad source cannot stop a pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EngineError {
    /// A call arrived after `shutdown` [engine.api].
    Stopped,
    /// The caller cancelled before the report was produced.
    Cancelled,
    /// A watched root could not be enumerated.
    Scan(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped => write!(f, "DMX1005 [engine.api]: the engine has shut down"),
            Self::Cancelled => write!(f, "DMX1005 [engine.api]: report cancelled"),
            Self::Scan(message) => write!(f, "DMX1002 [engine.api]: {message}"),
        }
    }
}

impl std::error::Error for EngineError {}

/// The engine: watched roots, the pipeline options they are generated with,
/// and the state one rescan advances.
#[derive(Debug)]
pub struct Engine {
    /// Everything a full rescan covers, as given to `new`.
    roots: Vec<PathBuf>,
    /// The pipeline options every pass runs with.
    opts: Options,
    /// What the last pass learned, and which generation it is.
    state: Mutex<State>,
}

/// Everything one engine remembers between rescans [engine].
#[derive(Debug)]
struct State {
    /// The current generation; monotonic, one step per successful rescan.
    generation: Generation,
    /// The pass that produced `generation`.
    last: Pass,
    /// `None` once `shutdown` has run. Dropping the sender is what completes
    /// every subscriber stream, so the flag and the fact are one thing.
    events: Option<broadcast::Sender<GenerationEvent>>,
}

impl Engine {
    /// An engine over `roots`, which may name directories or single files —
    /// whatever `dmx build` would accept [surface.zero-config].
    #[must_use]
    pub fn new(roots: Vec<PathBuf>, opts: Options) -> Self {
        let (events, _) = broadcast::channel(EVENT_BACKLOG);
        Self {
            roots,
            opts,
            state: Mutex::new(State {
                generation: Generation::ZERO,
                last: Pass::default(),
                events: Some(events),
            }),
        }
    }

    /// A poisoned lock means a previous caller panicked mid-pass. The state it
    /// left behind is a complete `Pass` or the previous one — never a torn
    /// half — so recovering beats taking the whole watcher down with it.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The sources a scope resolves to, with the zero-config exclusions
    /// applied [surface.zero-config].
    fn targets(&self, scope: &RescanScope) -> Result<Vec<PathBuf>, EngineError> {
        let paths = match scope {
            RescanScope::Paths(paths) => paths.as_slice(),
            // `RescanScope` is non-exhaustive: a scope this build does not know
            // degrades to the full sweep, which is always correct and only ever
            // slower than the narrower one it stood in for.
            _ => self.roots.as_slice(),
        };
        collect_sources(paths).map_err(|error| EngineError::Scan(format!("{error:#}")))
    }

    /// Records `pass` as the current state and publishes the new generation.
    fn advance(&self, pass: Pass, cause: Cause) -> Result<RescanTicket, EngineError> {
        let mut state = self.state();
        let Some(events) = state.events.clone() else {
            return Err(EngineError::Stopped);
        };
        let generation = state.generation.next();
        state.generation = generation;
        state.last = pass;
        // Fails only when nobody is subscribed, which is the common case for a
        // CLI watcher and not a problem for anyone.
        drop(events.send(GenerationEvent::new(generation, cause)));
        Ok(RescanTicket::new(generation))
    }
}

/// One file through the whole pipeline, with failure recorded rather than
/// propagated: a source that does not parse must not stop its neighbours.
fn run_one(path: &Path, roots: &[PathBuf], opts: Options) -> FileOutcome {
    match process_path(path, roots, &opts) {
        Ok(Outcome::Updated) => FileOutcome::Written,
        Ok(Outcome::Unchanged) => FileOutcome::Unchanged,
        Err(error) => FileOutcome::Refused(format!("{error:#}")),
    }
}

#[async_trait]
impl EngineApi for Engine {
    type Report = Pass;
    type Query = Query;
    type Error = EngineError;

    fn generation(&self) -> Generation {
        self.state().generation
    }

    async fn report(
        &self,
        query: Query,
        cancel: CancellationToken,
    ) -> Result<Snapshot<Pass>, EngineError> {
        let state = self.state();
        if state.events.is_none() {
            return Err(EngineError::Stopped);
        }
        // The whole report is a clone of state already in hand, so observing
        // cancellation once, up front, is the bounded delay the contract asks
        // for [engine.api].
        if cancel.is_cancelled() {
            return Err(EngineError::Cancelled);
        }
        match query {
            Query::LastPass => Ok(Snapshot::new(state.generation, state.last.clone())),
        }
    }

    async fn rescan(
        &self,
        scope: RescanScope,
        progress: Progress,
    ) -> Result<RescanTicket, EngineError> {
        let targets = self.targets(&scope)?;
        progress.report("dmx: regenerating", None);
        // Deliberately synchronous: the pipeline is file I/O and tree-sitter,
        // and its only driver blocks on this future from its own thread. An
        // engine that spawned work here would buy nothing and owe cancellation.
        let files = targets
            .into_iter()
            .map(|path| {
                let outcome = run_one(&path, &self.roots, self.opts);
                (path, outcome)
            })
            .collect();
        self.advance(Pass { files }, Cause::Rescan)
    }

    fn subscribe(&self) -> GenerationEventStream {
        match self.state().events.as_ref() {
            // Shutdown completes every subscriber stream, including one asked
            // for afterwards [engine.api].
            None => Box::pin(tokio_stream::empty()),
            // A subscriber that lagged past the backlog skips to the newest
            // event rather than erroring: generations are monotonic, so the
            // one it sees next is still the truth.
            Some(events) => Box::pin(stream(BroadcastStream::new(events.subscribe()))),
        }
    }

    async fn shutdown(&self) -> Result<(), EngineError> {
        match self.state().events.take() {
            Some(events) => {
                drop(events);
                Ok(())
            }
            None => Err(EngineError::Stopped),
        }
    }
}

/// A lagging subscriber's error becomes a skipped event rather than a broken
/// stream: generations are monotonic, so whatever it sees next is still true.
fn stream(
    events: BroadcastStream<GenerationEvent>,
) -> impl Stream<Item = GenerationEvent> + Send + 'static {
    events.filter_map(Result::ok)
}
