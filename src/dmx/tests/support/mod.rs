//! A scratch directory that cleans itself up, shared by the test binaries that
//! need real files on a real filesystem — which is all of them, because the
//! thing under test is a program that reads and writes Dart sources.

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

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempDirectory {
    pub path: PathBuf,
}

impl TempDirectory {
    /// A directory nobody else holds. The suffix loop covers two tests that
    /// start inside the same nanosecond, which parallel test binaries do.
    pub fn create(prefix: &str) -> io::Result<Self> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        for suffix in 0..100 {
            let path = std::env::temp_dir()
                .join(format!("{prefix}-{}-{unique}-{suffix}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::other(
            "could not allocate a unique test directory",
        ))
    }

    pub fn write(&self, name: &str, contents: &str) -> io::Result<PathBuf> {
        let path = self.path.join(name);
        fs::write(&path, contents)?;
        Ok(path)
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
