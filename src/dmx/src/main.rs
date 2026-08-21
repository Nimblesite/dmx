//! `dmx` CLI [cli], currently supporting build and watch.

use anyhow::{Result, bail};
use std::path::PathBuf;
use std::process::ExitCode;

use dmx::{Options, Outcome, process_path, sources, typediagram, watch};

/// What `dmx` prints when it cannot tell what was asked of it.
const USAGE: &str = "usage:\n  dmx build [PATHS...] [--insert-regions] [--check]\n  \
     dmx watch [PATHS...]\n  dmx explain FILE\n  dmx --version\n  dmx --help";

/// The subcommand this invocation is [cli].
#[derive(Clone, Copy)]
enum Command {
    /// Generate once over every path given.
    Build,
    /// Generate, then keep generating as the sources change.
    Watch,
    /// Print what a source produces, without producing it [typediagram.execution].
    Explain,
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
        Some("explain") => Command::Explain,
        // A binary that ships inside a VSIX has to be able to say which one it
        // is: `dmx.path` points at a build of somebody's choosing, and the
        // first question any report about it raises is which build [cli].
        Some("--version" | "-V") => {
            println!("dmx {}", dmx::VERSION);
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
    // `lib` is the zero-config source root [surface.zero-config], and it is the
    // default for the subcommands that sweep. `explain` names one file, so a
    // default there would only ever be the wrong file.
    if paths.is_empty() && !matches!(command, Command::Explain) {
        paths.push(PathBuf::from("lib"));
    }

    match command {
        Command::Build => build(&paths, opts),
        Command::Watch | Command::Explain if opts.insert_regions || opts.check => {
            bail!("[cli] build-only flags are not accepted by this subcommand\n{USAGE}")
        }
        Command::Watch => {
            watch::run(&paths, &opts)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Explain => explain(&paths),
    }
}

/// Prints the generation groups, dependencies, and exact context of one
/// typeDiagram source [typediagram.execution].
///
/// A definition file, a template beside one, or a Markdown document: three
/// spellings of the same question, so all three answer it.
fn explain(paths: &[PathBuf]) -> Result<ExitCode> {
    let [path] = paths else {
        bail!("[cli] `dmx explain` takes exactly one file\n{USAGE}");
    };
    let report = match path {
        _ if typediagram::is_markdown(path) => typediagram::document::explain(path)?,
        _ if typediagram::is_definition(path) => typediagram::standalone::explain(path)?,
        _ => match typediagram::definition_of(path) {
            Some(definition) => typediagram::standalone::explain(&definition)?,
            None => bail!(
                "[cli] `dmx explain` explains a typeDiagram definition (`.td`), a template bound \
                 to one, or a Markdown document; {} is none of those",
                path.display()
            ),
        },
    };
    print!("{report}");
    Ok(ExitCode::SUCCESS)
}

/// One generation pass, reporting what it wrote and exiting non-zero under
/// `--check` when anything was out of date [execution].
fn build(paths: &[PathBuf], opts: Options) -> Result<ExitCode> {
    let files = sources::collect_sources(paths)?;
    let mut updated = 0usize;
    for file in &files {
        if let Outcome::Updated = process_path(file, paths, &opts)? {
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
