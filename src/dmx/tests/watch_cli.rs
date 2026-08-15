//! `dmx watch` driven as a user drives it: the real binary, real files, and
//! the lines it prints [execution.modes], [cli].

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

use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};
use support::TempDirectory;

const INITIAL_SOURCE: &str = r"@dmx('model')
class User {
  const User({required this.id});

  final String id;
}
";

const READY_TIMEOUT: Duration = Duration::from_secs(5);
const REGENERATION_TIMEOUT: Duration = Duration::from_secs(5);
const QUIET_PERIOD: Duration = Duration::from_millis(750);

struct GeneratedFixture {
    directory: TempDirectory,
}

impl GeneratedFixture {
    fn create() -> io::Result<Self> {
        let directory = TempDirectory::create("dmx-watch-cli")?;
        let source_path = directory.write("user.dart", INITIAL_SOURCE)?;
        build_initial_region(&source_path)?;
        Ok(Self { directory })
    }

    fn source_path(&self) -> PathBuf {
        self.directory.path.join("user.dart")
    }
}

struct WatchedGeneratedFixture {
    generated: GeneratedFixture,
    initial_source: String,
    watcher: WatchProcess,
}

impl WatchedGeneratedFixture {
    fn create() -> io::Result<Self> {
        let generated = GeneratedFixture::create()?;
        let source_path = generated.source_path();
        let initial_source = fs::read_to_string(&source_path)?;
        let watcher = WatchProcess::spawn_ready(&source_path)?;
        Ok(Self {
            generated,
            initial_source,
            watcher,
        })
    }

    fn source_path(&self) -> PathBuf {
        self.generated.source_path()
    }
}

struct WatchProcess {
    child: Child,
    logs: Receiver<String>,
    observed: Vec<String>,
}

impl WatchProcess {
    fn spawn_ready(path: &Path) -> io::Result<Self> {
        let mut watcher = Self::spawn(path)?;
        watcher.wait_until_ready(1)?;
        Ok(watcher)
    }

    fn spawn(path: &Path) -> io::Result<Self> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_dmx"))
            .arg("watch")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("watch stdout was not piped"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("watch stderr was not piped"))?;
        let (sender, logs) = mpsc::channel();
        spawn_line_reader("stdout", stdout, sender.clone());
        spawn_line_reader("stderr", stderr, sender);
        Ok(Self {
            child,
            logs,
            observed: Vec::new(),
        })
    }

    fn wait_until_ready(&mut self, root_count: usize) -> io::Result<()> {
        let expected = format!("stdout: dmx: watching {root_count} path(s)");
        self.wait_for_log(READY_TIMEOUT, |line| line == expected, &expected)
    }

    fn wait_for_write(&mut self, path: &Path) -> io::Result<()> {
        let expected = write_log(path)?;
        self.wait_for_log(REGENERATION_TIMEOUT, |line| line == expected, &expected)
    }

    fn wait_for_error(&mut self, path: &Path, diagnostic: &str) -> io::Result<()> {
        let expected = error_log(path, diagnostic)?;
        self.wait_for_log(
            REGENERATION_TIMEOUT,
            |line| line.starts_with(&expected),
            &expected,
        )
    }

    fn wait_for_log(
        &mut self,
        timeout: Duration,
        matches: impl Fn(&str) -> bool,
        expected: &str,
    ) -> io::Result<()> {
        let baseline = self.observed.len();
        self.wait_for_observed(timeout, expected, |lines| {
            lines[baseline..].iter().any(|line| matches(line))
        })
    }

    fn wait_for_error_and_write(
        &mut self,
        invalid_path: &Path,
        diagnostic: &str,
        valid_path: &Path,
    ) -> io::Result<()> {
        let error = error_log(invalid_path, diagnostic)?;
        let write = write_log(valid_path)?;
        let expected = format!("`{error}…` and `{write}`");
        let baseline = self.observed.len();
        self.wait_for_observed(REGENERATION_TIMEOUT, &expected, |lines| {
            lines[baseline..]
                .iter()
                .any(|line| line.starts_with(&error))
                && lines[baseline..].iter().any(|line| line == &write)
        })
    }

    fn wait_for_observed(
        &mut self,
        timeout: Duration,
        expected: &str,
        complete: impl Fn(&[String]) -> bool,
    ) -> io::Result<()> {
        let deadline = Instant::now() + timeout;
        loop {
            if complete(&self.observed) {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.logs.recv_timeout(remaining) {
                Ok(line) => self.observed.push(line),
                Err(RecvTimeoutError::Timeout) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!(
                            "watcher never emitted `{expected}`; output:\n{}",
                            self.output()
                        ),
                    ));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        format!(
                            "watcher exited before emitting `{expected}`; output:\n{}",
                            self.output()
                        ),
                    ));
                }
            }
        }
    }

    fn observe_for(&mut self, duration: Duration) {
        let deadline = Instant::now() + duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            match self.logs.recv_timeout(remaining) {
                Ok(line) => self.observed.push(line),
                Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn writes(&self) -> Vec<&str> {
        self.observed
            .iter()
            .map(String::as_str)
            .filter(|line| line.starts_with("stdout: wrote: "))
            .collect()
    }

    fn output(&self) -> String {
        self.observed.join("\n")
    }

    fn is_running(&mut self) -> io::Result<bool> {
        self.child.try_wait().map(|status| status.is_none())
    }
}

impl Drop for WatchProcess {
    fn drop(&mut self) {
        // Still running, so end it; already gone or unknowable, so nothing to
        // do — a test fixture cannot report a failure from `drop` anyway.
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn write_log(path: &Path) -> io::Result<String> {
    Ok(format!("stdout: wrote: {}", path.canonicalize()?.display()))
}

fn error_log(path: &Path, diagnostic: &str) -> io::Result<String> {
    Ok(format!(
        "stderr: error: {}: {diagnostic}",
        path.canonicalize()?.display()
    ))
}

fn assert_one_write_and_running(watcher: &mut WatchProcess, context: &str) -> io::Result<()> {
    assert_eq!(watcher.writes().len(), 1, "output:\n{}", watcher.output());
    assert!(
        watcher.is_running()?,
        "watcher stopped {context}:\n{}",
        watcher.output()
    );
    Ok(())
}

fn assert_quiet_and_running(watcher: &mut WatchProcess, context: &str) -> io::Result<()> {
    assert!(watcher.writes().is_empty(), "output:\n{}", watcher.output());
    assert!(
        !watcher
            .observed
            .iter()
            .any(|line| line.starts_with("stderr: ")),
        "{context} produced an error:\n{}",
        watcher.output()
    );
    assert!(watcher.is_running()?, "watcher stopped after {context}");
    Ok(())
}

fn spawn_line_reader(
    stream_name: &'static str,
    stream: impl Read + Send + 'static,
    sender: Sender<String>,
) {
    drop(thread::spawn(move || {
        for result in BufReader::new(stream).lines() {
            let line = match result {
                Ok(line) => line,
                Err(error) => format!("could not read {stream_name}: {error}"),
            };
            if sender.send(format!("{stream_name}: {line}")).is_err() {
                break;
            }
        }
    }));
}

fn build_initial_region(source_path: &Path) -> io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_dmx"))
        .arg("build")
        .arg(source_path)
        .arg("--insert-regions")
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "initial build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

fn rename_model_field(source_path: &Path, previous: &str, next: &str) -> io::Result<()> {
    let source = fs::read_to_string(source_path)?;
    fs::write(source_path, renamed_model_field(&source, previous, next)?)
}

fn renamed_model_field(source: &str, previous: &str, next: &str) -> io::Result<String> {
    let previous_constructor = format!("  const User({{required this.{previous}}});");
    let next_constructor = format!("  const User({{required this.{next}}});");
    let previous_field = format!("  final String {previous};");
    let next_field = format!("  final String {next};");
    let constructor_edited = replace_required(
        source,
        &previous_constructor,
        &next_constructor,
        previous,
        "constructor",
    )?;
    replace_required(
        &constructor_edited,
        &previous_field,
        &next_field,
        previous,
        "field",
    )
}

fn invalid_model_source(source: &str) -> io::Result<String> {
    replace_required(
        source,
        "  final String id;",
        "  final String id",
        "id",
        "field",
    )
}

fn replace_required(
    source: &str,
    previous: &str,
    next: &str,
    field: &str,
    declaration: &str,
) -> io::Result<String> {
    let edited = source.replacen(previous, next, 1);
    if edited == source {
        return Err(io::Error::other(format!(
            "could not find `{field}` {declaration} declaration to edit"
        )));
    }
    Ok(edited)
}

fn wait_for_generated_field(source_path: &Path, field: &str, timeout: Duration) -> io::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let source = fs::read_to_string(source_path)?;
        if generated_region(&source)?.contains(&format!("'{field}': {field}")) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("generated members never reflected field `{field}`:\n{source}"),
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_regeneration(
    watcher: &mut WatchProcess,
    source_path: &Path,
    field: &str,
) -> io::Result<()> {
    wait_for_generated_field(source_path, field, REGENERATION_TIMEOUT)?;
    watcher.wait_for_write(source_path)
}

fn generated_region(source: &str) -> io::Result<&str> {
    let start = marker_offset(source, "//#region")?;
    let end = marker_offset(source, "//#endregion")?;
    Ok(&source[start..end])
}

fn marker_offset(source: &str, marker: &str) -> io::Result<usize> {
    source
        .find(marker)
        .ok_or_else(|| io::Error::other(format!("generated region marker `{marker}` is missing")))
}

fn assert_region_presence(
    source_path: &Path,
    needle: &str,
    expected: bool,
    context: &str,
) -> io::Result<()> {
    let source = fs::read_to_string(source_path)?;
    assert_eq!(
        generated_region(&source)?.contains(needle),
        expected,
        "{context}:\n{source}"
    );
    Ok(())
}

/// Reproduces a region gutted by hand: everything from the divider down to the
/// middle of the generated `toJson` is deleted, leaving a dangling map entry and
/// a `};` that closes the class body early. That is what makes the file
/// unparseable, and why regenerating it needs [emission.inline-backend.region-recovery].
fn gut_generated_region(source: &str) -> io::Result<String> {
    let start = marker_offset(source, "//#region")? + "//#region".len();
    let dangling = source
        .find("        'id': id,")
        .ok_or_else(|| io::Error::other("generated toJson entry is missing"))?;
    Ok(format!("{}\n  \n{}", &source[..start], &source[dangling..]))
}

/// Empties the region without breaking the file: still valid Dart, so this
/// exercises the ordinary path rather than recovery.
fn empty_generated_region(source: &str) -> io::Result<String> {
    let start = marker_offset(source, "//#region")? + "//#region".len();
    let end = marker_offset(source, "//#endregion")?;
    Ok(format!("{}\n  {}", &source[..start], &source[end..]))
}

/// Waits for the file to come back byte-for-byte, failing loudly if it does not.
///
/// Deliberately not `wait_for_generated_field`: a gutted region still contains
/// `'id': id` in its dangling fragment, so a field probe would pass without a
/// single member having been restored.
fn wait_for_source(source_path: &Path, expected: &str, timeout: Duration) -> io::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let actual = fs::read_to_string(source_path)?;
        if actual == expected {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("the watcher never regenerated the file; it is still:\n{actual}"),
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn run_to_exit(mut command: Command, timeout: Duration) -> io::Result<Output> {
    let _ = command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait()? {
            Some(_) => return child.wait_with_output(),
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            None => {
                child.kill()?;
                let output = child.wait_with_output()?;
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!(
                        "command did not exit\nstdout:\n{}\nstderr:\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    ),
                ));
            }
        }
    }
}

fn watch_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dmx"));
    let _ = command.arg("watch");
    command
}

fn run_watch_target_to_exit(path: &Path) -> io::Result<Output> {
    let mut command = watch_command();
    let _ = command.arg(path);
    run_to_exit(command, READY_TIMEOUT)
}

fn failed_stderr(output: &Output, target: &Path, accepted_message: &str) -> String {
    assert_eq!(output.status.code(), Some(1), "{accepted_message}");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        stderr.contains(&target.display().to_string()),
        "stderr omitted rejected target:\n{stderr}"
    );
    stderr
}

/// Verifies debounced watch mode [execution.modes] through the explicit-path CLI [cli].
#[test]
fn watch_regenerates_the_same_dart_file_after_each_edit() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    rename_model_field(&source_path, "id", "name")?;
    wait_for_regeneration(&mut fixture.watcher, &source_path, "name")?;

    // A second edit proves the watcher survives its own atomic replacement of
    // the inline Dart file instead of losing an inode-specific file watch.
    rename_model_field(&source_path, "name", "email")?;
    wait_for_regeneration(&mut fixture.watcher, &source_path, "email")?;

    assert!(
        fixture.watcher.is_running()?,
        "watcher stopped after regenerating"
    );
    assert_eq!(
        fixture.watcher.writes().len(),
        2,
        "output:\n{}",
        fixture.watcher.output()
    );
    Ok(())
}

/// Verifies recursive source discovery and exclusions [surface.zero-config] under watch mode [execution.modes].
#[test]
fn watch_recurses_but_ignores_hidden_and_generated_dart_files() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-watch-cli")?;
    let nested = directory.path.join("lib").join("models");
    let hidden = nested.join(".private");
    fs::create_dir_all(&hidden)?;

    let watched_path = nested.join("user.dart");
    fs::write(&watched_path, INITIAL_SOURCE)?;
    build_initial_region(&watched_path)?;
    let generated_source = fs::read_to_string(&watched_path)?;

    let generated_path = nested.join("user.g.dart");
    let hidden_path = hidden.join("user.dart");
    fs::write(&generated_path, &generated_source)?;
    fs::write(&hidden_path, &generated_source)?;

    let mut watcher = WatchProcess::spawn_ready(&directory.path)?;

    rename_model_field(&watched_path, "id", "name")?;
    rename_model_field(&generated_path, "id", "name")?;
    rename_model_field(&hidden_path, "id", "name")?;

    wait_for_regeneration(&mut watcher, &watched_path, "name")?;
    watcher.observe_for(QUIET_PERIOD);

    for excluded in [&generated_path, &hidden_path] {
        let context = format!("excluded file regenerated: {}", excluded.display());
        assert_region_presence(excluded, "'id': id", true, &context)?;
        assert_region_presence(excluded, "'name': name", false, &context)?;
    }
    assert_one_write_and_running(&mut watcher, "after exclusions")
}

/// Verifies recursive source filtering [surface.zero-config] ignores directories ending in `.dart`.
#[test]
fn watch_ignores_a_directory_whose_name_ends_in_dart() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-watch-cli")?;
    let mut watcher = WatchProcess::spawn_ready(&directory.path)?;

    let fake_source = directory.path.join("fake.dart");
    fs::create_dir(&fake_source)?;
    let child = fake_source.join("child.txt");
    fs::write(&child, "not Dart\n")?;
    watcher.observe_for(QUIET_PERIOD);

    assert!(fake_source.is_dir(), "fake Dart directory disappeared");
    assert!(child.is_file(), "child in fake Dart directory disappeared");
    assert_quiet_and_running(&mut watcher, "fake Dart directory")
}

/// Verifies recursive watch boundaries [surface.zero-config] do not traverse directory symlinks.
#[cfg(unix)]
#[test]
fn watch_ignores_a_symlinked_source_directory_at_startup_and_during_events() -> io::Result<()> {
    use std::os::unix::fs::symlink;

    let external = GeneratedFixture::create()?;
    let external_source = external.source_path();
    rename_model_field(&external_source, "id", "name")?;
    let watched_directory = TempDirectory::create("dmx-watch-cli")?;
    let linked_directory = watched_directory.path.join("linked_models");
    symlink(&external.directory.path, &linked_directory)?;

    let mut watcher = WatchProcess::spawn_ready(&watched_directory.path)?;
    assert_region_presence(
        &external_source,
        "'id': id",
        true,
        "startup traversed symlinked source directory",
    )?;

    rename_model_field(&external_source, "name", "email")?;
    watcher.observe_for(QUIET_PERIOD);

    assert_region_presence(
        &external_source,
        "'id': id",
        true,
        "event watching traversed symlinked source directory",
    )?;
    assert!(
        fs::symlink_metadata(&linked_directory)?
            .file_type()
            .is_symlink(),
        "source directory symlink disappeared"
    );
    assert_quiet_and_running(&mut watcher, "symlinked source directory")
}

/// Verifies recursive watch mode [execution.modes] discovers Dart sources created after startup [surface.zero-config].
#[test]
fn watch_regenerates_a_stale_dart_file_created_after_readiness() -> io::Result<()> {
    let source_fixture = GeneratedFixture::create()?;
    let generated = fs::read_to_string(source_fixture.source_path())?;
    let stale = renamed_model_field(&generated, "id", "name")?;
    let watched_directory = TempDirectory::create("dmx-watch-cli")?;

    let mut watcher = WatchProcess::spawn_ready(&watched_directory.path)?;
    assert_eq!(
        watcher.output(),
        "stdout: dmx: watching 1 path(s)",
        "empty startup unexpectedly generated output"
    );

    let nested = watched_directory.path.join("lib").join("models");
    fs::create_dir_all(&nested)?;
    let introduced = nested.join("user.dart");
    fs::write(&introduced, stale)?;

    wait_for_regeneration(&mut watcher, &introduced, "name")?;
    watcher.observe_for(QUIET_PERIOD);

    assert_region_presence(
        &introduced,
        "'id': id",
        false,
        "new source retained stale generated members",
    )?;
    assert_one_write_and_running(&mut watcher, "after new source")
}

/// Verifies watch mode performs its initial incremental run [execution.modes] before CLI readiness [cli].
#[test]
fn watch_repairs_existing_drift_before_reporting_readiness() -> io::Result<()> {
    let fixture = GeneratedFixture::create()?;
    let source_path = fixture.source_path();
    rename_model_field(&source_path, "id", "name")?;

    let watcher = WatchProcess::spawn_ready(&source_path)?;

    wait_for_generated_field(&source_path, "name", Duration::ZERO)?;
    assert_eq!(watcher.writes().len(), 1, "output:\n{}", watcher.output());
    assert!(
        watcher.observed[0].starts_with("stdout: wrote: "),
        "readiness preceded startup regeneration:\n{}",
        watcher.output()
    );
    assert_eq!(
        watcher.observed[1],
        "stdout: dmx: watching 1 path(s)",
        "unexpected readiness sequence:\n{}",
        watcher.output()
    );
    Ok(())
}

/// [emission.inline-backend.region-location]: legacy qualified markers regenerate under [execution.modes].
#[test]
fn watch_regenerates_a_file_with_legacy_qualified_region_markers() -> io::Result<()> {
    let fixture = GeneratedFixture::create()?;
    let source_path = fixture.source_path();
    let generated = fs::read_to_string(&source_path)?;
    let Some((prefix, after_start)) = generated.split_once("  //#region\n") else {
        return Err(io::Error::other("bare generated region start is missing"));
    };
    let Some((body, suffix)) = after_start.split_once("  //#endregion") else {
        return Err(io::Error::other("bare generated region end is missing"));
    };
    let hashed_body = body.strip_suffix('\n').unwrap_or(body);
    let hash = &blake3::hash(hashed_body.as_bytes()).to_hex()[..16];
    let legacy_header =
        format!("//#region dmx:generated builtin/model@0.1.0 b3:{hash} — DO NOT EDIT");
    let legacy_source =
        format!("{prefix}  {legacy_header}\n{body}  //#endregion dmx:generated{suffix}");
    assert!(
        legacy_source.contains(&legacy_header),
        "legacy qualified marker was not installed"
    );
    fs::write(&source_path, legacy_source)?;

    let mut watcher = WatchProcess::spawn_ready(&source_path)?;
    rename_model_field(&source_path, "id", "name")?;
    watcher.wait_for_write(&source_path)?;
    wait_for_generated_field(&source_path, "name", Duration::ZERO)
}

/// Verifies validation failures [validation] are recoverable events in watch mode [execution.modes].
#[test]
fn watch_survives_invalid_dart_and_regenerates_after_the_next_valid_save() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    let invalid_source = invalid_model_source(&fixture.initial_source)?;
    fs::write(&source_path, &invalid_source)?;
    fixture.watcher.wait_for_error(&source_path, "DMX4001")?;
    assert_eq!(
        fs::read_to_string(&source_path)?,
        invalid_source,
        "watcher overwrote invalid user input"
    );
    assert!(
        fixture.watcher.is_running()?,
        "watcher died on invalid Dart"
    );

    fs::write(
        &source_path,
        renamed_model_field(&fixture.initial_source, "id", "name")?,
    )?;
    wait_for_regeneration(&mut fixture.watcher, &source_path, "name")?;
    assert_one_write_and_running(&mut fixture.watcher, "after recovering")
}

/// Verifies one validation failure [validation] cannot block another watched target [execution.modes].
#[test]
fn watch_isolates_an_invalid_file_from_a_valid_file_in_the_same_batch() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-watch-cli")?;
    let invalid_path = directory.path.join("a_invalid.dart");
    fs::write(&invalid_path, INITIAL_SOURCE)?;
    build_initial_region(&invalid_path)?;
    let generated = fs::read_to_string(&invalid_path)?;
    let valid_path = directory.path.join("z_valid.dart");
    fs::write(&valid_path, &generated)?;

    let mut watcher = WatchProcess::spawn_ready(&directory.path)?;

    let invalid = invalid_model_source(&generated)?;
    fs::write(&invalid_path, &invalid)?;
    fs::write(&valid_path, renamed_model_field(&generated, "id", "name")?)?;

    watcher.wait_for_error_and_write(&invalid_path, "DMX4001", &valid_path)?;
    wait_for_generated_field(&valid_path, "name", Duration::ZERO)?;
    assert_eq!(
        fs::read_to_string(&invalid_path)?,
        invalid,
        "watcher overwrote the invalid file"
    );
    watcher.observe_for(QUIET_PERIOD);

    let invalid_error = error_log(&invalid_path, "DMX4001")?;
    let error_count = watcher
        .observed
        .iter()
        .filter(|line| line.starts_with(&invalid_error))
        .count();
    assert_eq!(error_count, 1, "output:\n{}", watcher.output());
    assert_one_write_and_running(&mut watcher, "after one file failed")
}

/// Verifies trailing debounce [execution.modes] and no-op feedback-loop prevention [emission.inline-backend.no-op-writes].
#[test]
fn watch_coalesces_a_save_burst_and_does_not_rewrite_its_own_output() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    for field in ["first", "second", "finalValue"] {
        fs::write(
            &source_path,
            renamed_model_field(&fixture.initial_source, "id", field)?,
        )?;
    }
    wait_for_regeneration(&mut fixture.watcher, &source_path, "finalValue")?;

    let settled_content = fs::read_to_string(&source_path)?;
    let settled_mtime = fs::metadata(&source_path)?.modified()?;
    fixture.watcher.observe_for(QUIET_PERIOD);

    assert_eq!(
        fs::read_to_string(&source_path)?,
        settled_content,
        "watch feedback changed file content"
    );
    assert_eq!(
        fs::metadata(&source_path)?.modified()?,
        settled_mtime,
        "watch feedback changed file mtime"
    );
    assert_one_write_and_running(&mut fixture.watcher, "after save burst")
}

/// Verifies ignored event churn cannot starve the watch debounce deadline [execution.modes].
#[test]
fn watch_regenerates_dart_while_an_ignored_file_keeps_churning() -> io::Result<()> {
    let fixture = GeneratedFixture::create()?;
    let source_path = fixture.source_path();
    let noise_path = fixture.directory.path.join("noise.txt");
    fs::write(&noise_path, "initial\n")?;
    let mut watcher = WatchProcess::spawn_ready(&fixture.directory.path)?;

    let (started_sender, started_receiver) = mpsc::channel();
    let churn_path = noise_path.clone();
    let churn = thread::spawn(move || -> io::Result<()> {
        for tick in 0..700 {
            fs::write(&churn_path, tick.to_string())?;
            if tick == 0 {
                started_sender
                    .send(())
                    .map_err(|_| io::Error::other("churn start receiver disconnected"))?;
            }
            thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    });
    started_receiver
        .recv_timeout(READY_TIMEOUT)
        .map_err(|_| io::Error::other("ignored-file churn did not start"))?;

    let edit_started = Instant::now();
    rename_model_field(&source_path, "id", "name")?;
    wait_for_regeneration(&mut watcher, &source_path, "name")?;
    let regeneration_elapsed = edit_started.elapsed();

    assert!(
        !churn.is_finished(),
        "Dart regeneration waited for ignored-file churn to finish"
    );
    assert!(
        regeneration_elapsed < Duration::from_secs(1),
        "Dart regeneration took {regeneration_elapsed:?} during ignored churn"
    );
    churn
        .join()
        .map_err(|_| io::Error::other("ignored-file churn thread panicked"))??;
    watcher.observe_for(QUIET_PERIOD);
    assert_one_write_and_running(&mut watcher, "during ignored-file churn")
}

/// Verifies the watch invocation surface rejects build-only flags [cli].
#[test]
fn watch_rejects_every_build_only_flag() -> io::Result<()> {
    for flag in ["--insert-regions", "--force", "--check"] {
        let mut command = watch_command();
        let _ = command.arg(flag);
        let output = run_to_exit(command, READY_TIMEOUT)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "flag `{flag}` was accepted");
        assert!(stderr.contains(flag), "stderr omitted `{flag}`:\n{stderr}");
        assert!(stderr.contains("usage:"), "stderr omitted usage:\n{stderr}");
        assert!(
            stderr.contains("dmx watch"),
            "stderr omitted watch usage:\n{stderr}"
        );
    }
    Ok(())
}

/// Verifies a missing watch root is a hard invocation failure [cli].
#[test]
fn watch_rejects_a_missing_root() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-watch-cli")?;
    let missing = directory.path.join("missing");
    let output = run_watch_target_to_exit(&missing)?;
    let stderr = failed_stderr(&output, &missing, "missing root was accepted");

    assert!(stderr.starts_with("error:"), "unexpected stderr:\n{stderr}");
    Ok(())
}

/// Verifies explicit watch targets obey source inclusion [surface.zero-config] and [cli].
#[test]
fn watch_rejects_an_explicit_non_dart_file_without_reporting_readiness() -> io::Result<()> {
    let directory = TempDirectory::create("dmx-watch-cli")?;
    let non_dart = directory.path.join("notes.txt");
    fs::write(&non_dart, "not Dart\n")?;
    let output = run_watch_target_to_exit(&non_dart)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = failed_stderr(&output, &non_dart, "non-Dart target was accepted");

    assert!(stdout.is_empty(), "unexpected stdout:\n{stdout}");
    assert!(
        stderr.starts_with("error: DMX1002 [cli]: watch target is not a Dart source:"),
        "unexpected stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("dmx: watching"),
        "readiness reported for rejected target:\n{stderr}"
    );
    Ok(())
}

/// Verifies the watcher regenerates a region a human gutted
/// [emission.inline-backend.region-recovery], [execution.modes].
///
/// This is the case that used to fail silently in an editor: deleting generated
/// members leaves the file unparseable, validation rejected the whole input, and
/// the members stayed deleted no matter how often the file was saved.
#[test]
fn watch_regenerates_a_region_gutted_by_hand() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    let gutted = gut_generated_region(&fixture.initial_source)?;
    assert!(
        !gutted.contains("Map<String, dynamic> toJson()"),
        "the fixture removed no generated members, so this proves nothing:\n{gutted}"
    );
    fs::write(&source_path, &gutted)?;

    // Byte equality with the healthy source: every member must return, not just
    // enough of one to satisfy a substring probe.
    wait_for_source(&source_path, &fixture.initial_source, REGENERATION_TIMEOUT)?;
    fixture.watcher.wait_for_write(&source_path)?;
    assert_one_write_and_running(&mut fixture.watcher, "after repairing a gutted region")
}

/// Verifies an emptied — but still parseable — region is refilled [execution.modes].
///
/// The control for the test above: this path never needed recovery, so it must
/// keep working exactly as before.
#[test]
fn watch_refills_a_region_emptied_without_breaking_the_file() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    let emptied = empty_generated_region(&fixture.initial_source)?;
    assert!(
        !emptied.contains("Map<String, dynamic> toJson()"),
        "the fixture removed no generated members:\n{emptied}"
    );
    fs::write(&source_path, &emptied)?;

    wait_for_source(&source_path, &fixture.initial_source, REGENERATION_TIMEOUT)?;
    fixture.watcher.wait_for_write(&source_path)?;
    assert_one_write_and_running(&mut fixture.watcher, "after refilling an emptied region")
}

/// Verifies repair is repeatable, not a one-shot [emission.inline-backend.region-recovery].
///
/// A watcher that repairs once and then goes deaf is the same bug wearing a
/// different hat, so the damage is inflicted twice against the same process.
#[test]
fn watch_regenerates_a_region_gutted_twice_in_a_row() -> io::Result<()> {
    let mut fixture = WatchedGeneratedFixture::create()?;
    let source_path = fixture.source_path();

    for round in 1..=2 {
        fs::write(&source_path, gut_generated_region(&fixture.initial_source)?)?;
        wait_for_source(&source_path, &fixture.initial_source, REGENERATION_TIMEOUT)
            .map_err(|e| io::Error::new(e.kind(), format!("round {round}: {e}")))?;
        fixture.watcher.wait_for_write(&source_path)?;
    }
    assert!(
        fixture.watcher.is_running()?,
        "watcher stopped after repeated repairs:\n{}",
        fixture.watcher.output()
    );
    Ok(())
}
