//! One scratch workspace, entered by one test at a time.
//!
//! The working directory is process-wide state, and both front ends generate
//! against it: an output path is workspace-relative, and the root it resolves
//! under is found by walking up from the source. Tests that need a real tree
//! therefore have to enter one, and two of them entering different trees at
//! once is a race that shows up as a file another test already deleted.
//!
//! The lock lives here rather than in either front end's tests because a lock
//! per module locks nothing: the thing being shared is the process.

use std::fs;
use std::path::{Path, PathBuf};

/// The one lock every test that enters a workspace holds.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Runs `body` in a fresh workspace holding `files`, with the process working
/// directory pointed at it.
///
/// Each entry is a workspace-relative path and its contents; parent
/// directories are created. The workspace is removed afterwards whatever
/// `body` did.
///
/// # Panics
///
/// Panics when the workspace cannot be created, written, entered, or left. A
/// test that cannot get a directory has not failed at what it was testing, and
/// unwinding here names the problem better than any `Result` plumbing would.
pub fn in_workspace<T>(files: &[(&str, &str)], body: impl FnOnce(&Path) -> T) -> T {
    let guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let directory = directory();
    for (name, content) in files {
        let path = directory.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory");
        }
        fs::write(&path, content).expect("fixture");
    }
    let previous = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(&directory).expect("enter workspace");
    let outcome = body(&directory);
    std::env::set_current_dir(previous).expect("leave workspace");
    drop(fs::remove_dir_all(&directory));
    drop(guard);
    outcome
}

/// A directory nobody else holds.
fn directory() -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!("dmx-td-{}-{unique}", std::process::id()));
    fs::create_dir_all(&path).expect("scratch directory");
    path
}
