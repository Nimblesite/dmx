//! A scratch package the real `dmx` binary is driven over [typediagram].
//!
//! Shared rather than private to one suite because more than one front end
//! generates into a package — a Markdown document and a standalone `.td`
//! definition — and both are driven the same way: write files into a scratch
//! directory, run `dmx build` from inside it, and read what came back. What
//! differs between them is the seed files and the arguments `build` takes,
//! which is what [`Workspace::create`] takes.
//!
//! It is included with `#[path]` by the suites that need one, so no other
//! binary compiles it.

// [TEST-RULES] admits `expect` in a test, and a fixture carries what a
// workspace can be asked, not what one suite happens to ask it.
#![cfg_attr(test, allow(dead_code, clippy::expect_used, clippy::panic))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::support::TempDirectory;

pub struct Workspace {
    /// The scratch directory, removed when the test ends.
    pub directory: TempDirectory,
    /// The arguments [`Workspace::build`] runs `dmx` with.
    build: Vec<String>,
}

impl Workspace {
    /// A package with `lib/` in it, whatever `files` names, and `build`
    /// remembered as what a build of it runs.
    pub fn create(prefix: &str, build: &[&str], files: &[(&str, &str)]) -> Self {
        let directory = TempDirectory::create(prefix).expect("scratch directory");
        fs::create_dir_all(directory.at("lib")).expect("lib directory");
        let workspace = Self {
            directory,
            build: build
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
        };
        for (name, contents) in files {
            workspace.write(name, contents);
        }
        workspace
    }

    /// The workspace root.
    pub fn root(&self) -> &Path {
        &self.directory.path
    }

    /// One path inside it.
    pub fn path(&self, relative: &str) -> PathBuf {
        self.directory.at(relative)
    }

    /// The contents of one file inside it.
    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative))
            .unwrap_or_else(|error| panic!("cannot read {relative}: {error}"))
    }

    /// Whether one path inside it exists.
    pub fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    /// Writes one file inside it, creating the directories it needs.
    pub fn write(&self, relative: &str, contents: &str) {
        let _ = self.directory.write(relative, contents).expect(relative);
    }

    /// Runs `dmx` from the workspace root, as a shell in it would.
    pub fn dmx(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dmx"))
            .args(args)
            .current_dir(self.root())
            .output()
            .expect("run dmx")
    }

    /// A build over this workspace, which must succeed. Its stdout is what a
    /// pass reports.
    pub fn build(&self) -> String {
        let output = self.run();
        assert!(
            output.status.success(),
            "dmx build failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// The refusal a build produced, which it must have produced.
    pub fn build_failure(&self) -> String {
        let output = self.run();
        assert!(
            !output.status.success(),
            "dmx build succeeded when it should have refused\nstdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// This workspace's build, however it ended.
    fn run(&self) -> Output {
        let arguments: Vec<&str> = self.build.iter().map(String::as_str).collect();
        self.dmx(&arguments)
    }
}
