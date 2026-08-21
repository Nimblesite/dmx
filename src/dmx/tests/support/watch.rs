//! `dmx watch` as a test drives it: the real binary, piped output, and the
//! lines it prints [execution.modes], [cli].
//!
//! Shared rather than private to one suite because more than one thing is
//! watched — Dart sources, Markdown documents, and standalone typeDiagram
//! definitions — and a second copy of a process harness is a second set of
//! timeouts to get wrong. It is included with `#[path]` by the suites that
//! spawn a watcher, so no other binary compiles it.

// [TEST-RULES] admits `expect` in a test, and every waiter here is only useful
// to some of its callers: a harness carries what a watcher can be asked, not
// what one suite happens to ask it.
#![cfg_attr(test, allow(dead_code, clippy::expect_used, clippy::panic))]

use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// How long the watcher has to announce the paths it is watching.
pub const READY_TIMEOUT: Duration = Duration::from_secs(5);

/// How long one save has to produce the line it should produce.
pub const REGENERATION_TIMEOUT: Duration = Duration::from_secs(5);

pub struct WatchProcess {
    /// The running `dmx watch`.
    child: Child,
    /// Lines the two reader threads have handed over.
    logs: Receiver<String>,
    /// Every line either stream has produced so far, in arrival order.
    pub observed: Vec<String>,
}

impl WatchProcess {
    pub fn spawn_ready(path: &Path) -> io::Result<Self> {
        let mut watcher = Self::spawn(path)?;
        watcher.wait_until_ready(1)?;
        Ok(watcher)
    }

    pub fn spawn(path: &Path) -> io::Result<Self> {
        Self::spawn_args(None, &[path.as_os_str()])
    }

    /// A watcher started *inside* `directory`, watching the relative paths
    /// `args` names.
    ///
    /// A Markdown document's outputs are workspace-relative
    /// [typediagram.output], so where the watcher runs is part of what it does
    /// — which is the one thing `spawn` cannot express.
    pub fn spawn_ready_in(directory: &Path, args: &[&str]) -> io::Result<Self> {
        let owned: Vec<&std::ffi::OsStr> = args.iter().map(std::ffi::OsStr::new).collect();
        let mut watcher = Self::spawn_args(Some(directory), &owned)?;
        watcher.wait_until_ready(args.len())?;
        Ok(watcher)
    }

    pub fn spawn_args(directory: Option<&Path>, args: &[&std::ffi::OsStr]) -> io::Result<Self> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dmx"));
        let _ = command.arg("watch").args(args);
        if let Some(directory) = directory {
            let _ = command.current_dir(directory);
        }
        let mut child = command
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

    pub fn wait_until_ready(&mut self, root_count: usize) -> io::Result<()> {
        let expected = format!("stdout: dmx: watching {root_count} path(s)");
        self.wait_for_log(READY_TIMEOUT, |line| line == expected, &expected)
    }

    /// Waits for a line on `stream` carrying `needle`.
    ///
    /// The exact-match waiters below spell out a whole line because a Dart
    /// source's write line is one path and nothing else. A document is named
    /// by both its write line and its diagnostics, so what identifies which
    /// one arrived is the stream it arrived on.
    pub fn wait_for_line_on(&mut self, stream: &str, needle: &str) -> io::Result<()> {
        let prefix = stream.to_owned();
        let expected = format!("{stream}…{needle}");
        self.wait_for_log(
            REGENERATION_TIMEOUT,
            move |line| line.starts_with(&prefix) && line.contains(needle),
            &expected,
        )
    }

    pub fn wait_for_write(&mut self, path: &Path) -> io::Result<()> {
        let expected = write_log(path)?;
        self.wait_for_log(REGENERATION_TIMEOUT, |line| line == expected, &expected)
    }

    pub fn wait_for_error(&mut self, path: &Path, diagnostic: &str) -> io::Result<()> {
        let expected = error_log(path, diagnostic)?;
        self.wait_for_log(
            REGENERATION_TIMEOUT,
            |line| line.starts_with(&expected),
            &expected,
        )
    }

    pub fn wait_for_log(
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

    pub fn wait_for_error_and_write(
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

    pub fn wait_for_observed(
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

    pub fn observe_for(&mut self, duration: Duration) {
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

    pub fn writes(&self) -> Vec<&str> {
        self.observed
            .iter()
            .map(String::as_str)
            .filter(|line| line.starts_with("stdout: wrote: "))
            .collect()
    }

    pub fn output(&self) -> String {
        self.observed.join("\n")
    }

    pub fn is_running(&mut self) -> io::Result<bool> {
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

/// The exact line the watcher prints when it writes `path`.
///
/// # Errors
///
/// Fails when `path` cannot be canonicalized, which means it is not there.
pub fn write_log(path: &Path) -> io::Result<String> {
    Ok(format!("stdout: wrote: {}", path.canonicalize()?.display()))
}

/// The exact prefix the watcher prints when `path` is refused.
///
/// # Errors
///
/// Fails when `path` cannot be canonicalized, which means it is not there.
pub fn error_log(path: &Path, diagnostic: &str) -> io::Result<String> {
    Ok(format!(
        "stderr: error: {}: {diagnostic}",
        path.canonicalize()?.display()
    ))
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
