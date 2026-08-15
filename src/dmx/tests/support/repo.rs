//! Reaching the repository from inside the crate [repo.layout].
//!
//! Shared by the test binaries that hold the repository's own configuration to
//! the tree it describes, and by nothing else — the suites that only need a
//! scratch directory take `support/mod.rs` instead, so neither binary compiles
//! what it does not use.

// [TEST-RULES] admits `expect` in a test: a fixture that cannot be built is a
// broken test, and unwinding at the point of failure names it better than any
// `Result` plumbing would. Production code is still held to `unwrap_used` and
// `expect_used` at deny — this relaxation is `cfg(test)`-scoped on purpose.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic))]

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root. The crate lives at `src/dmx`, so its manifest sits two
/// directories below the root that carries the Makefile, licence and editors.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the crate manifest is two directories below the repository root")
        .to_owned()
}

/// A repository-relative file, read whole. A missing one is a broken test
/// rather than a failed assertion: the suite is describing a tree that is not
/// there.
pub fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}
