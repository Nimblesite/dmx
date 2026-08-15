//! `dmx` CLI [cli], currently supporting build and watch.

use anyhow::{Result, bail};
use std::path::PathBuf;
use std::process::ExitCode;

use dmx::{Options, Outcome, process_file, watch};

/// What `dmx` prints when it cannot tell what was asked of it.
const USAGE: &str = "usage:\n  dmx build [PATHS...] [--insert-regions] [--check]\n  \
     dmx watch [PATHS...]\n  dmx --version\n  dmx --help";

/// The version this build reports [release.version].
///
/// The tag is the version. `Cargo.toml` carries the placeholder `0.0.0` and is
/// never rewritten — cargo owns that file, and nothing in this repository may
/// edit a structured file by pattern. The release passes the version the tag
/// names in `DMX_VERSION` instead, so the number is a property of the build
/// rather than of a commit somebody had to remember to bump.
///
/// A build with nothing to inject reports the placeholder, which is the honest
/// answer: a local build is not a release.
const VERSION: &str = match option_env!("DMX_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

/// The subcommand this invocation is [cli].
#[derive(Clone, Copy)]
enum Command {
    /// Generate once over every path given.
    Build,
    /// Generate, then keep generating as the sources change.
    Watch,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(1)
        }
    }
}

/// Parses the argument list and runs the subcommand it names.
fn run() -> Result<ExitCode> {
    let mut args = std::env::args().skip(1);
    let command = match args.next().as_deref() {
        Some("build") => Command::Build,
        Some("watch") => Command::Watch,
        // A binary that ships inside a VSIX has to be able to say which one it
        // is: `dmx.path` points at a build of somebody's choosing, and the
        // first question any report about it raises is which build [cli].
        Some("--version" | "-V") => {
            println!("dmx {VERSION}");
            return Ok(ExitCode::SUCCESS);
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            return Ok(ExitCode::SUCCESS);
        }
        _ => bail!("[cli] {USAGE}"),
    };
    let mut opts = Options {
        insert_regions: false,
        check: false,
    };
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--insert-regions" => opts.insert_regions = true,
            "--check" => opts.check = true,
            _ if arg.starts_with('-') => bail!("[cli] unknown flag `{arg}`\n{USAGE}"),
            _ => paths.push(PathBuf::from(arg)),
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("lib"));
    }

    match command {
        Command::Build => build(&paths, opts),
        Command::Watch if opts.insert_regions || opts.check => {
            bail!("[cli] build-only flags are not accepted by `dmx watch`\n{USAGE}")
        }
        Command::Watch => {
            watch::run(&paths, &opts)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// One generation pass, reporting what it wrote and exiting non-zero under
/// `--check` when anything was out of date [execution].
fn build(paths: &[PathBuf], opts: Options) -> Result<ExitCode> {
    let files = watch::collect_dart_files(paths)?;
    let mut updated = 0usize;
    for file in &files {
        if let Outcome::Updated = process_file(file, &opts)? {
            updated = updated.saturating_add(1);
            println!(
                "{} {}",
                if opts.check { "drift:" } else { "wrote:" },
                file.display()
            );
        }
    }
    if opts.check && updated > 0 {
        return Ok(ExitCode::from(2)); // Drift exit status [cli].
    }
    println!("dmx: {updated} of {} file(s) updated", files.len());
    Ok(ExitCode::SUCCESS)
}
