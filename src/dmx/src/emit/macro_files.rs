//! Whole files authored by a Dart macro [dartmacros.files].

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use super::{
    GeneratedFile, Options, collect_stale, dart_files_in, dart_files_under, file_marker,
    package_root, refuse_symlink_escape, relative_name, resolved, seed_dir, sibling_marker,
    write_owned,
};

/// A resolved destination with the ownership marker and validated Dart source.
struct MacroTarget<'a> {
    /// The destination within the seed's sibling directory or package.
    path: PathBuf,
    /// The marker identifying the annotated seed that owns this output.
    marker: String,
    /// Complete source returned by the Dart macro.
    text: &'a str,
}

/// Emits every macro-authored sibling or package-relative file, then collects
/// files this seed no longer produces [dartmacros.files].
///
/// # Errors
///
/// Fails before writing on collisions, unsafe paths, or human-owned targets.
pub fn emit_macro_files(seed: &Path, files: &[GeneratedFile], opts: &Options) -> Result<bool> {
    let targets = macro_targets(seed, files)?;
    let written = targets.iter().try_fold(false, |changed, target| {
        Ok::<bool, anyhow::Error>(write_macro_target(target, opts.check)? || changed)
    })?;
    Ok(collect_macro_stale(seed, &targets, opts.check)? || written)
}

/// Resolves the whole output set and refuses aliases of the same destination.
fn macro_targets<'a>(seed: &Path, files: &'a [GeneratedFile]) -> Result<Vec<MacroTarget<'a>>> {
    let mut targets = Vec::new();
    for file in files {
        let target = macro_target(seed, file)?;
        refuse_duplicate(&targets, &target)?;
        targets.push(target);
    }
    Ok(targets)
}

/// Rejects two output names that resolve to the same file.
fn refuse_duplicate(prior: &[MacroTarget<'_>], target: &MacroTarget<'_>) -> Result<()> {
    if prior
        .iter()
        .any(|item| resolved(&item.path) == resolved(&target.path))
    {
        bail!(
            "DMX7008: two macro outputs resolve to `{}` [dartmacros.files]",
            target.path.display()
        );
    }
    Ok(())
}

/// Resolves one output, preserving bare-name sibling behavior.
fn macro_target<'a>(seed: &Path, file: &'a GeneratedFile) -> Result<MacroTarget<'a>> {
    let (path, marker) = if file.name.contains('/') {
        package_target(seed, &file.name)?
    } else {
        (seed_dir(seed).join(&file.name), sibling_marker(seed))
    };
    if resolved(&path) == resolved(seed) {
        bail!(
            "DMX7008: macro file `{}` would overwrite the annotated file itself [dartmacros.files]",
            file.name
        );
    }
    Ok(MacroTarget {
        path,
        marker,
        text: &file.text,
    })
}

/// Anchors a path at the nearest package and refuses symlink escapes.
fn package_target(seed: &Path, name: &str) -> Result<(PathBuf, String)> {
    let root = package_root(seed).ok_or_else(|| {
        anyhow::anyhow!(
            "DMX7007: package-relative macro output `{name}` needs a pubspec.yaml [dartmacros.files]"
        )
    })?;
    let target = root.join(name);
    refuse_symlink_escape(&root, &target).map_err(|detail| {
        anyhow::anyhow!("DMX7007: macro output `{name}` {detail} [dartmacros.files]")
    })?;
    Ok((target, file_marker(&relative_name(&root, seed))))
}

/// Writes or checks one owned output with the canonical marker framing.
fn write_macro_target(target: &MacroTarget<'_>, check: bool) -> Result<bool> {
    let content = format!("{}\n\n{}\n", target.marker, target.text);
    write_owned(
        &target.path,
        &content,
        check,
        "DMX7008",
        "[dartmacros.files]",
    )
}

/// Collects obsolete outputs owned by this seed in both supported scopes.
fn collect_macro_stale(seed: &Path, targets: &[MacroTarget<'_>], check: bool) -> Result<bool> {
    let sibling = sibling_marker(seed);
    let kept = kept_with_marker(targets, &sibling);
    let mut changed = collect_stale(&dart_files_in(seed_dir(seed))?, &sibling, &kept, check)?;
    if let Some(root) = package_root(seed) {
        changed |= collect_package_stale(seed, &root, targets, check)?;
    }
    Ok(changed)
}

/// Collects obsolete package-relative outputs, never another seed's files.
fn collect_package_stale(
    seed: &Path,
    root: &Path,
    targets: &[MacroTarget<'_>],
    check: bool,
) -> Result<bool> {
    let marker = file_marker(&relative_name(root, seed));
    let kept = kept_with_marker(targets, &marker);
    collect_stale(&dart_files_under(root)?, &marker, &kept, check)
}

/// The current destinations sharing one ownership marker.
fn kept_with_marker(targets: &[MacroTarget<'_>], marker: &str) -> Vec<PathBuf> {
    targets
        .iter()
        .filter(|target| target.marker == marker)
        .map(|target| target.path.clone())
        .collect()
}
