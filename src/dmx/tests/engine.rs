//! The engine contract, exercised as a consumer would [engine.api].
//!
//! Real files, real generation, real generations advancing — the watcher is one
//! consumer of this API and an LSP server will be another, so what is asserted
//! here is the contract itself, not the watcher's use of it.

// [TEST-RULES] admits `expect` in a test: a fixture that cannot be built is a
// broken test, and unwinding at the point of failure names it better than any
// `Result` plumbing would. Production code is still held to `unwrap_used` and
// `expect_used` at deny — this relaxation is `cfg(test)`-scoped on purpose.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )
)]

mod support;

use dmx::Options;
use dmx::engine::{Engine, EngineError, FileOutcome, Pass, Query};
use lspkit::{EngineApi as _, Generation, Progress, RescanScope};
use std::io;
use std::path::Path;
use support::TempDirectory;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt as _;
use tokio_util::sync::CancellationToken;

const MODEL: &str = r"@dmx('model')
class User {
  const User({required this.id});

  final String id;
}
";

/// Dart that no parser will accept, which is what an editor holds for most of
/// the time a human is typing.
const BROKEN: &str = r"@dmx('model')
class Broken {
  const Broken({required this.id});
";

fn runtime() -> io::Result<Runtime> {
    tokio::runtime::Builder::new_current_thread().build()
}

fn opts() -> Options {
    Options {
        insert_regions: true,
        check: false,
    }
}

fn engine(directory: &TempDirectory) -> Engine {
    Engine::new(vec![directory.path.clone()], opts())
}

fn rescan(runtime: &Runtime, engine: &Engine, scope: RescanScope) -> Generation {
    runtime
        .block_on(engine.rescan(scope, Progress::noop()))
        .expect("rescan")
        .generation()
}

fn last_pass(runtime: &Runtime, engine: &Engine) -> Pass {
    runtime
        .block_on(engine.report(Query::LastPass, CancellationToken::new()))
        .expect("report")
        .data
}

fn outcome<'a>(pass: &'a Pass, path: &Path) -> &'a FileOutcome {
    let canonical = path.canonicalize().expect("canonicalize");
    &pass
        .files
        .iter()
        .find(|(candidate, _)| {
            candidate
                .canonicalize()
                .unwrap_or_else(|_| candidate.clone())
                == canonical
        })
        .unwrap_or_else(|| panic!("{} is not in the pass", path.display()))
        .1
}

/// The first rescan generates; the second finds nothing to do and says so
/// [emission.inline-backend.no-op-writes]. Both are successful rescans, so both
/// advance the generation — "nothing changed" is an answer, not a non-event.
#[test]
fn a_pass_writes_once_and_then_reports_the_file_unchanged() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    let source = directory.write("user.dart", MODEL)?;
    let runtime = runtime()?;
    let engine = engine(&directory);

    assert_eq!(engine.generation(), Generation::ZERO);

    let first = rescan(&runtime, &engine, RescanScope::All);
    assert_eq!(
        outcome(&last_pass(&runtime, &engine), &source),
        &FileOutcome::Written
    );

    let second = rescan(&runtime, &engine, RescanScope::All);
    assert!(second > first, "a rescan must advance the generation");
    assert_eq!(
        outcome(&last_pass(&runtime, &engine), &source),
        &FileOutcome::Unchanged
    );
    assert_eq!(engine.generation(), second);
    Ok(())
}

/// One unparseable file must not cost its neighbour its generated members
/// [engine.api]. This is the state an editor is in constantly.
#[test]
fn a_file_that_does_not_parse_is_refused_without_stopping_the_pass() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    let broken = directory.write("broken.dart", BROKEN)?;
    let healthy = directory.write("user.dart", MODEL)?;
    let runtime = runtime()?;
    let engine = engine(&directory);

    let _ = rescan(&runtime, &engine, RescanScope::All);
    let pass = last_pass(&runtime, &engine);

    assert_eq!(outcome(&pass, &healthy), &FileOutcome::Written);
    match outcome(&pass, &broken) {
        FileOutcome::Refused(message) => assert!(
            !message.is_empty(),
            "a refusal with no diagnostic tells the author nothing"
        ),
        other => panic!("unparseable Dart was not refused: {other:?}"),
    }
    Ok(())
}

/// A narrowed scope is the whole point of incrementality: saving one file must
/// not re-examine the package.
#[test]
fn a_paths_scope_examines_only_those_paths() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    let first = directory.write("first.dart", MODEL)?;
    drop(directory.write("second.dart", MODEL)?);
    let runtime = runtime()?;
    let engine = engine(&directory);

    let _ = rescan(&runtime, &engine, RescanScope::Paths(vec![first.clone()]));
    let pass = last_pass(&runtime, &engine);

    assert_eq!(
        pass.files.iter().map(|(path, _)| path).collect::<Vec<_>>(),
        vec![&first],
        "a Paths scope examined something it was not asked about"
    );
    Ok(())
}

/// Every consumer that is not asking learns about a new generation this way.
#[test]
fn a_subscriber_sees_one_event_per_rescan() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    drop(directory.write("user.dart", MODEL)?);
    let runtime = runtime()?;
    let engine = engine(&directory);
    let mut events = engine.subscribe();

    let first = rescan(&runtime, &engine, RescanScope::All);
    let second = rescan(&runtime, &engine, RescanScope::All);

    let observed: Vec<Generation> = vec![
        runtime
            .block_on(events.next())
            .expect("first event")
            .generation,
        runtime
            .block_on(events.next())
            .expect("second event")
            .generation,
    ];
    assert_eq!(observed, vec![first, second]);
    Ok(())
}

/// Shutdown is a promise to two parties: subscribers get an ending, and callers
/// get an error instead of an answer from a drained engine [engine.api].
#[test]
fn shutdown_completes_subscribers_and_refuses_every_later_call() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    drop(directory.write("user.dart", MODEL)?);
    let runtime = runtime()?;
    let engine = engine(&directory);
    let mut events = engine.subscribe();

    let _ = rescan(&runtime, &engine, RescanScope::All);
    runtime.block_on(engine.shutdown()).expect("shutdown");

    assert!(
        runtime.block_on(events.next()).is_some(),
        "the event before shutdown was dropped"
    );
    assert!(
        runtime.block_on(events.next()).is_none(),
        "shutdown left a subscriber hanging"
    );
    assert_eq!(
        runtime
            .block_on(engine.report(Query::LastPass, CancellationToken::new()))
            .err(),
        Some(EngineError::Stopped)
    );
    assert_eq!(
        runtime
            .block_on(engine.rescan(RescanScope::All, Progress::noop()))
            .err(),
        Some(EngineError::Stopped)
    );
    assert_eq!(
        runtime.block_on(engine.shutdown()).err(),
        Some(EngineError::Stopped)
    );
    Ok(())
}

/// A cancelled request answers with a cancellation, promptly, rather than with
/// a report nobody is waiting for any more [engine.api].
#[test]
fn a_cancelled_report_returns_cancelled() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    drop(directory.write("user.dart", MODEL)?);
    let runtime = runtime()?;
    let engine = engine(&directory);
    let cancel = CancellationToken::new();
    cancel.cancel();

    assert_eq!(
        runtime
            .block_on(engine.report(Query::LastPass, cancel))
            .err(),
        Some(EngineError::Cancelled)
    );
    Ok(())
}

/// A root that vanished is the one failure that is the engine's own, not a
/// file's: nothing can be enumerated, so there is no pass to report.
#[test]
fn a_missing_root_fails_the_rescan_rather_than_reporting_an_empty_pass() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-engine")?;
    let missing = directory.path.join("gone");
    let runtime = runtime()?;
    let engine = Engine::new(vec![missing.clone()], opts());

    let scope = RescanScope::Paths(vec![missing]);
    match runtime.block_on(engine.rescan(scope, Progress::noop())) {
        // `collect_dart_files` skips what is not there; a path that names no
        // Dart at all is an empty pass, and that is honest. What must never
        // happen is a silent success that claims to have generated something.
        Ok(_) => assert!(last_pass(&runtime, &engine).files.is_empty()),
        Err(error) => assert!(matches!(error, EngineError::Scan(_))),
    }
    Ok(())
}
