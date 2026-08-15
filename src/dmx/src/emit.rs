//! Stage 8: emission, `inline` backend [emission.inline-backend].
//!
//! Splices rendered members between the dividers inside the class body, with
//! byte-exactness verification [emission.inline-backend.byte-exactness], no-op writes [emission.inline-backend.no-op-writes], and atomic
//! replace [validation].
//!
//! There is no hash in the divider and no tamper check. Generation is a pure
//! function of the source, so a region that differs from what dmx would emit is
//! simply out of date; `--check` [execution] reports that as drift and exits non-zero.

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use anyhow::{Result, bail};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead as _, BufReader, ErrorKind};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use crate::frontend::{REGION_END, REGION_START, RawDecl, is_region_end, region_opener};

/// One whole sibling Dart file a macro authored and named
/// [dartmacros.files].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedFile {
    /// A bare `*.dart` file name, validated on receipt from the worker.
    pub name: String,
    /// The file's complete Dart source, normalized like any fragment.
    pub text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// What one run of the pipeline was asked to do [cli].
pub struct Options {
    /// Add a divider to an annotated declaration that has none.
    pub insert_regions: bool,
    /// `check` mode: never write, exit non-zero on drift [execution].
    pub check: bool,
}

/// The region as it is written: the dividers with the body between them.
fn region_text(body: &str) -> String {
    format!("  {REGION_START}\n{body}\n  {REGION_END}")
}

/// Splices one class's rendered body into `src`, returning the new source.
///
/// # Errors
///
/// Fails when the declaration has no divider and `--insert-regions` was not
/// given, or when its divider could not be located unambiguously.
pub fn splice(src: &str, class: &RawDecl, rendered: &str, opts: &Options) -> Result<String> {
    let region = region_text(rendered);
    match (&class.region, &class.region_error) {
        (Some(existing), _) => Ok(format!(
            "{}{}{}",
            &src[..existing.start],
            region,
            &src[existing.end..]
        )),
        // The front end defers divider faults so that an unannotated declaration
        // with a malformed fold stays none of dmx's business. Here a macro does
        // need the region, so the fault becomes fatal — and fatal *before*
        // `--insert-regions`, because adding a divider to a class that already
        // has an ambiguous one compounds the ambiguity rather than resolving it.
        (None, Some(fault)) => bail!("{fault}"),
        (None, None) if opts.insert_regions => {
            // [emission.inline-backend.insertion]: insert immediately before the closing brace, separated
            // from the preceding member by exactly one blank line, preserving
            // the brace's own indentation.
            let brace_line = crate::frontend::line_start(src, class.close_brace);
            let indent = &src[brace_line..class.close_brace];
            let head = src[..brace_line].trim_end_matches(['\n', ' ', '\t']);
            Ok(format!(
                "{head}\n\n{region}\n{indent}{}",
                &src[class.close_brace..]
            ))
        }
        (None, None) => bail!(
            "DMX6002: class `{}` has no `{REGION_START}` divider; add one or re-run \
             with --insert-regions:\n\n  {REGION_START}\n  {REGION_END}",
            class.name
        ),
    }
}

/// [emission.inline-backend.byte-exactness]: user content outside the regions must survive unchanged.
///
/// Stated precisely: every non-blank line outside the dividers is preserved, in
/// order, byte for byte. Blank lines are excluded because inserting a region
/// legitimately adjusts the blank line that separates it from the preceding
/// member — no user *content* may move either way.
///
/// # Errors
///
/// Fails when the file cannot be written, or when the temporary file it is
/// written through cannot be created or moved into place.
pub fn verify_byte_exactness(before: &str, after: &str) -> Result<()> {
    if content_outside_regions(before) != content_outside_regions(after) {
        bail!("DMX6103: internal error — content outside the divider changed; aborting");
    }
    Ok(())
}

/// Where a line sits relative to the machine-owned regions.
#[derive(PartialEq, Eq)]
enum Zone {
    /// The author's own line.
    Outside,
    /// A `//#region` or `//#endregion` divider.
    Marker,
    /// Generated content, between the dividers.
    Body,
}

/// Classifies every line by zone.
///
/// Ownership is decided by the same predicates the structural locator uses
/// [emission.inline-backend.region-location], not by a second string test: a
/// divider dmx will *rewrite* must be one this scanner already counts as dmx's,
/// or migrating a legacy marker to the bare form would read as the author's
/// content changing and trip DMX6103.
fn zones(src: &str) -> impl Iterator<Item = (&str, Zone)> {
    let mut in_region = false;
    src.lines().map(move |line| {
        let comment = line.trim();
        match (in_region, region_opener(comment), is_region_end(comment)) {
            (false, Some(true), _) => {
                in_region = true;
                (line, Zone::Marker)
            }
            (true, _, true) => {
                in_region = false;
                (line, Zone::Marker)
            }
            (true, ..) => (line, Zone::Body),
            (false, ..) => (line, Zone::Outside),
        }
    })
}

/// Every non-blank line the author owns — the bytes byte-exactness protects.
fn content_outside_regions(src: &str) -> Vec<&str> {
    zones(src)
        .filter(|(line, zone)| *zone == Zone::Outside && !line.trim().is_empty())
        .map(|(line, _)| line)
        .collect()
}

/// Empties every machine-owned region, leaving its dividers in place
/// [emission.inline-backend.region-recovery].
///
/// Used to recover a file whose region a human has gutted. Deleting generated
/// members usually leaves the file unparseable — an orphaned `};` closes the
/// class early — and the parse errors then land *outside* the region, so no
/// containment test can attribute the damage. Blanking the bytes dmx owns and
/// re-parsing settles it directly: if the file parses now, everything broken
/// was ours to replace.
#[must_use]
pub fn strip_region_bodies(src: &str) -> String {
    let kept: Vec<&str> = zones(src)
        .filter(|(_, zone)| *zone != Zone::Body)
        .map(|(line, _)| line)
        .collect();
    let mut out = kept.join("\n");
    if src.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The prefix every macro-authored file's first line carries — the whole
/// ownership protocol [dartmacros.files]: a file that starts with it is
/// machine-owned outright, and one that does not is somebody's hand-written
/// Dart that dmx must never touch.
#[cfg(not(target_arch = "wasm32"))]
const FILE_MARKER_PREFIX: &str = "// dmx: generated from ";

/// What that first line ends with, so the seed's name can be read back out of
/// it [dartmacros.files].
#[cfg(not(target_arch = "wasm32"))]
const FILE_MARKER_SUFFIX: &str = " — do not edit.";

/// The exact marker line for files generated from `seed_file_name`.
#[cfg(not(target_arch = "wasm32"))]
fn file_marker(seed_file_name: &str) -> String {
    format!("{FILE_MARKER_PREFIX}{seed_file_name}{FILE_MARKER_SUFFIX}")
}

/// The seed a macro-authored file names on its first line, when that seed is
/// still beside it [dartmacros.files].
///
/// This is what makes a generated file editable in the ordinary sense: an
/// editor saves it, and the marker says which annotated file has to run again
/// to put it back. `None` for anything dmx does not own — a hand-written
/// source, a marker naming a seed that is gone, a file that is not there.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn seed_of(path: &Path) -> Option<PathBuf> {
    let file = fs::File::open(path).ok()?;
    let mut first = String::new();
    // The marker is line one or it is not a marker, so one line is the whole
    // read no matter how large the generated file is.
    let _ = BufReader::new(file).read_line(&mut first).ok()?;
    let name = first
        .trim_end_matches('\n')
        .strip_prefix(FILE_MARKER_PREFIX)?
        .strip_suffix(FILE_MARKER_SUFFIX)?;
    let seed = path.parent().unwrap_or_else(|| Path::new(".")).join(name);
    seed.is_file().then_some(seed)
}

/// Emits every macro-authored file beside `seed`, and collects the ones a
/// previous pass wrote from this seed that this pass no longer produces
/// [dartmacros.files]. Returns whether anything changed (or, under `check`,
/// would change).
///
/// # Errors
///
/// Fails when a target exists without a dmx marker (`DMX7008` — that is a
/// human's file), when a name collides with the seed's own, or on I/O.
#[cfg(not(target_arch = "wasm32"))]
pub fn emit_macro_files(seed: &Path, files: &[GeneratedFile], opts: &Options) -> Result<bool> {
    let dir = match seed.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    let seed_name = seed
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let marker = file_marker(&seed_name);
    let mut changed = false;
    for file in files {
        if file.name == seed_name {
            bail!(
                "DMX7008: macro file `{}` would overwrite the annotated file itself \
                 [dartmacros.files]",
                file.name
            );
        }
        let target = dir.join(&file.name);
        let content = format!("{marker}\n\n{}\n", file.text);
        match fs::read_to_string(&target) {
            Ok(existing) if existing == content => continue,
            Ok(existing) if !existing.starts_with(FILE_MARKER_PREFIX) => bail!(
                "DMX7008: `{}` already exists and carries no dmx marker — a hand-written \
                 file is never overwritten [dartmacros.files]",
                target.display()
            ),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("DMX1002: cannot read {}", target.display()));
            }
        }
        changed = true;
        if !opts.check {
            write_atomic(&target, &content)
                .with_context(|| format!("DMX1003: cannot write {}", target.display()))?;
        }
    }
    Ok(collect_stale_files(dir, files, &marker, opts.check)? || changed)
}

/// Deletes (or, under `check`, reports) every `.dart` file in `dir` whose
/// marker names this seed and which this pass did not produce — a dropped
/// table means a dropped file [dartmacros.files].
#[cfg(not(target_arch = "wasm32"))]
fn collect_stale_files(
    dir: &Path,
    files: &[GeneratedFile],
    marker: &str,
    check: bool,
) -> Result<bool> {
    let mut changed = false;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let Some(name) = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };
        let is_dart = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dart"));
        if !is_dart || files.iter().any(|file| file.name == name) {
            continue;
        }
        // A file that is not UTF-8 cannot carry the ASCII marker; skip it.
        let Ok(existing) = fs::read_to_string(&path) else {
            continue;
        };
        if existing.lines().next() == Some(marker) {
            changed = true;
            if !check {
                fs::remove_file(&path)
                    .with_context(|| format!("DMX1003: cannot remove {}", path.display()))?;
            }
        }
    }
    Ok(changed)
}

/// Atomic write: temp file in the same directory, then rename [validation].
///
/// # Errors
///
/// Fails when a line the author owns differs before and after generation.
#[cfg(not(target_arch = "wasm32"))]
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(
        ".{}.dmx.tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
