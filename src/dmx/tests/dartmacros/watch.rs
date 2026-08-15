//! E2E: `dmx watch` keeps macro-authored files current [execution.modes],
//! [dartmacros.files].
//!
//! The seed is not the only file a macro owns, so it cannot be the only file
//! the watcher keeps current. Everything here is driven the way a person
//! drives it — save a file, delete a file, start the watcher from wherever
//! your editor happens to be — and asserted where a person sees it: on disk.

use std::fs;
use std::io::{BufRead as _, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use super::support::TempDirectory;
use super::{SEED_MARKER, build_and_read, dmx, files_project};

/// How long a watcher gets to answer a save before the test calls it broken.
const ANSWER: Duration = Duration::from_secs(10);

/// A worker authoring one file, whose name and content it chooses.
const ONE_FILE: &str = "List<Map<String, String>> files(Map<String, Object?> invocation) =>
   [{'name': 'customer_row.dart', 'text': 'final class CustomerRow {\\n  const CustomerRow();\\n}\\n'}];";

/// A `dmx watch` process, killed when the test that started it ends.
struct Watching {
    /// The watcher itself.
    child: Child,
    /// Its stdout, drained by a reader thread so a chatty pass cannot block
    /// the process on a full pipe.
    lines: Receiver<String>,
}

impl Watching {
    /// Starts `dmx watch <target>` in `directory` and waits until it says it
    /// is watching — an edit made before that is an edit nobody heard.
    fn started_in(directory: &Path, target: &str) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_dmx"))
            .args(["watch", target])
            .current_dir(directory)
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn dmx watch");
        let stdout = child.stdout.take().expect("watch stdout");
        let (sender, lines) = mpsc::channel();
        let _ = thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if sender.send(line).is_err() {
                    return;
                }
            }
        });
        let watching = Self { child, lines };
        watching.wait_until_ready();
        watching
    }

    /// The same, for the ordinary case of watching `lib` from the project.
    fn started(directory: &TempDirectory) -> Self {
        Self::started_in(&directory.path, "lib")
    }

    /// Blocks until the readiness line lands, so every later save is heard.
    fn wait_until_ready(&self) {
        let deadline = Instant::now() + ANSWER;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(remaining) {
                Ok(line) if line.starts_with("dmx: watching") => return,
                Ok(_) => {}
                Err(error) => panic!("the watcher never reported readiness: {error}"),
            }
        }
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Polls until `settled` holds, failing with what is actually on disk.
fn wait_for(description: &str, path: &Path, settled: impl Fn(&str) -> bool) {
    let deadline = Instant::now() + ANSWER;
    loop {
        let actual = fs::read_to_string(path).unwrap_or_default();
        if settled(&actual) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}; `{}` is:\n{actual}",
            path.display()
        );
        thread::sleep(Duration::from_millis(25));
    }
}

/// A generated project, plus the exact bytes its macro authored.
fn generated() -> (TempDirectory, String) {
    let dir = files_project(ONE_FILE);
    let _seed = build_and_read(&dir, "schema.dart");
    let authored = fs::read_to_string(dir.path.join("lib/customer_row.dart"))
        .expect("the first build must author the sibling");
    (dir, authored)
}

/// [dartmacros.files] + [execution.modes]: editing a file the macro wrote puts
/// it straight back. Deleting a generated member is how anyone tests a
/// generator, and the answer has to be that it grows back.
#[test]
fn an_edit_to_a_generated_file_is_undone() {
    let (dir, authored) = generated();
    let sibling = dir.path.join("lib/customer_row.dart");
    let _watching = Watching::started(&dir);

    fs::write(
        &sibling,
        format!("{SEED_MARKER}\n\nfinal class CustomerRow {{}}\n"),
    )
    .expect("tamper");

    wait_for("the generated file to be restored", &sibling, |actual| {
        actual == authored
    });
}

/// [dartmacros.files] + [execution.modes]: a generated file someone deletes is
/// written again. It is the macro's file, not theirs, and the source of truth
/// still says it exists.
#[test]
fn a_deleted_generated_file_is_written_again() {
    let (dir, authored) = generated();
    let sibling = dir.path.join("lib/customer_row.dart");
    let _watching = Watching::started(&dir);

    fs::remove_file(&sibling).expect("delete the generated file");

    wait_for("the generated file to come back", &sibling, |actual| {
        actual == authored
    });
}

/// [dartmacros.discovery] + [execution.modes]: the worker is found from the
/// source being generated, not from wherever the watcher was launched. An
/// editor starts it at the workspace root, which in any repo holding more than
/// one package is not the package root.
#[test]
fn the_watcher_regenerates_from_any_working_directory() {
    let (dir, authored) = generated();
    let sibling = dir.path.join("lib/customer_row.dart");
    let elsewhere = dir.path.parent().expect("temp dir has a parent").to_owned();
    let project = dir
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("project directory name")
        .to_owned();

    let _watching = Watching::started_in(&elsewhere, &format!("{project}/lib"));
    fs::write(
        &sibling,
        format!("{SEED_MARKER}\n\nfinal class CustomerRow {{}}\n"),
    )
    .expect("tamper");

    wait_for(
        "a watcher started outside the package to regenerate",
        &sibling,
        |actual| actual == authored,
    );
}

/// [dartmacros.files]: a pass where no macro ran keeps its hands off files
/// some macro wrote. Losing a worker — an uninstalled `dart`, a crash, a
/// checkout without `tool/` — must never be read as "every table was dropped".
#[test]
fn a_pass_that_ran_no_macro_collects_nothing() {
    let (dir, authored) = generated();
    fs::remove_file(dir.path.join("tool/dmx/macros.dart")).expect("remove the worker");

    let output = dmx(&dir, &["build", "lib"]);

    assert!(
        output.status.success(),
        "a project whose worker is gone still builds:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(dir.path.join("lib/customer_row.dart")).unwrap_or_default(),
        authored,
        "a pass that ran no macro must not collect what a macro wrote"
    );
}
