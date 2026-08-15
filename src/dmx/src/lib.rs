//! `dmx` — out-of-band Dart metaprogramming (spec v0.3).
//!
//! Pipeline (spec §3): parse → context → render → validate → emit inline.

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

pub mod casing;
#[cfg(not(target_arch = "wasm32"))]
pub mod dartmacros;
pub mod emit;
#[cfg(not(target_arch = "wasm32"))]
pub mod engine;
pub mod frontend;
pub mod jsoncontent;
pub mod macros;
pub mod render;
pub mod types;
#[cfg(not(target_arch = "wasm32"))]
pub mod watch;

use anyhow::{Context as _, Result, bail};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

pub use emit::{GeneratedFile, Options};

/// What one file's pass through the pipeline came to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Outcome {
    /// No annotated declarations, or output already up to date (spec §11.3.6).
    Unchanged,
    /// The file was (or, under `check`, would be) rewritten.
    Updated,
}

/// Everything one source's pass through the string pipeline produced.
#[derive(Debug)]
pub struct Processed {
    /// The file's new content, when it differs from the input.
    pub output: Option<String>,
    /// Whole sibling files the source's macros authored, validated but not
    /// yet written — emission is [`process_file`]'s business
    /// [dartmacros.files].
    pub files: Vec<GeneratedFile>,
    /// Whether a macro actually ran. Only then does this source own sibling
    /// files, and only then may a pass collect the ones it no longer produces
    /// [dartmacros.files].
    ///
    /// An annotation alone is not enough. A worker that is missing, uninstalled
    /// or mid-crash expands nothing, and reading that as "the source of truth
    /// dropped every table" would delete a generated tree over a broken
    /// toolchain.
    pub expanded: bool,
}

/// Makes a file with a hand-gutted region processable again
/// [emission.inline-backend.region-recovery].
///
/// Returns `Some(repaired)` when the input did not parse but does once the
/// machine-owned regions are emptied — proof that everything broken was inside
/// the divider, where regeneration is precisely the repair. Returns `None` when
/// the input already parses. Propagates the *original* parse error when the
/// file is still broken with the regions gone: that damage is the author's own,
/// and rewriting their code is never dmx's business.
fn repair_gutted_regions(frontend: &mut frontend::Frontend, src: &str) -> Result<Option<String>> {
    let Err(original) = frontend.validate(src, "input") else {
        return Ok(None);
    };
    let stripped = emit::strip_region_bodies(src);
    match frontend.validate(&stripped, "input") {
        Ok(()) => Ok(Some(stripped)),
        Err(_) => Err(original),
    }
}

/// Runs the whole pipeline over one Dart source string.
/// Returns the new file content when it differs from the input, together
/// with any whole files the source's macros authored [dartmacros.files].
///
/// # Errors
///
/// Fails when the input does not parse, when a macro refuses a declaration, or
/// when the generated output would not parse or would disturb the author's
/// own bytes.
pub fn process_source(src: &str, opts: &Options) -> Result<Processed> {
    process_source_inner(src, *opts, None, None)
}

/// Runs the production pipeline with a caller-supplied Mustache template
/// replacing the inferred macro's built-in template [playground.wasm].
///
/// Exactly one registered macro annotation must appear in `src`, so the
/// template has one unambiguous Rust-built context. The rendered Dart still
/// passes through the normal validation, byte-exactness, and inline emission
/// stages.
///
/// # Errors
///
/// Fails when the source does not contain exactly one registered macro, the
/// input or generated Dart does not parse, the macro refuses the declaration,
/// the user template does not compile, or byte-exactness is violated.
pub fn process_source_with_template(
    src: &str,
    template: &str,
    opts: &Options,
) -> Result<Processed> {
    process_source_inner(src, *opts, Some(template), None)
}

/// The one pipeline both entry points run, with `user_template` set only where
/// a caller supplied one [playground.wasm] and `origin` only where the source
/// came from a file [dartmacros.discovery].
fn process_source_inner(
    src: &str,
    opts: Options,
    user_template: Option<&str>,
    origin: Option<&std::path::Path>,
) -> Result<Processed> {
    let mut frontend = frontend::Frontend::new()?;
    // Refusing to touch unparseable Dart is right for the author's code, but
    // applied to the whole file it makes the one case that most needs
    // regenerating — a region someone just deleted from — the one case that can
    // never recover. Reduce that case to the healthy one first.
    let repaired = repair_gutted_regions(&mut frontend, src)?;
    let input = repaired.as_deref().unwrap_or(src);

    let mut out = input.to_owned();
    // The whole file reaches every macro, because the interesting ones read
    // their siblings [frontend.name-index]. Only the annotated declarations are
    // generated *into*.
    let file = frontend.declarations(input)?;
    if user_template.is_some() {
        let applications = macros::application_count(&file);
        if applications != 1 {
            bail!(
                "DMX4002 [playground.wasm]: a user template requires exactly one registered \
                 macro annotation; found {applications}"
            );
        }
    }
    let mut targets: Vec<&frontend::RawDecl> =
        file.iter().filter(|d| macros::applies_to(d)).collect();
    if targets.is_empty() {
        return Ok(Processed {
            output: None,
            files: Vec::new(),
            expanded: false,
        });
    }
    // Later declarations first, so earlier byte offsets stay valid while
    // splicing. Offsets come from `input`, so each declaration is spliced
    // against a tail that still matches — declarations never nest inside each
    // other's regions.
    targets.sort_by_key(|d| std::cmp::Reverse(d.close_brace));
    let mut files: Vec<GeneratedFile> = Vec::new();
    let mut expanded_any = false;
    for decl in targets {
        // A malformed fold is only an error once a macro needs the region it
        // failed to locate [emission.inline-backend.region-location].
        if let Some(error) = &decl.region_error {
            bail!("{error}");
        }
        let expansion = match user_template {
            Some(template) => {
                render::with_template(template, || macros::expand(decl, &file, origin))?
            }
            None => macros::expand(decl, &file, origin)?,
        };
        let Some(expanded) = expansion else {
            continue;
        };
        expanded_any = true;
        for authored in expanded.files {
            // One pass, one owner per name — two macros (or two declarations)
            // claiming the same file cannot both be right [dartmacros.files].
            if files.iter().any(|f| f.name == authored.name) {
                bail!(
                    "DMX7008: two macro expansions both author `{}` [dartmacros.files]",
                    authored.name
                );
            }
            // Spec §10 applies to whole authored files too: never write bad
            // Dart, and fail before anything lands on disk.
            frontend
                .validate(&authored.text, "macro-authored file")
                .with_context(|| format!("DMX7007: in macro-authored file `{}`", authored.name))?;
            files.push(authored);
        }
        out = emit::splice(&out, decl, &expanded.text, &opts)?;
    }
    // Compared against the original, not the stripped input: a recovered file
    // always differs, and an already-correct one must still report Unchanged.
    if out == *src {
        return Ok(Processed {
            output: None,
            files,
            expanded: expanded_any,
        });
    }
    // Spec §10: re-parse the complete candidate file; never write bad Dart.
    frontend.validate(&out, "generated output")?;
    emit::verify_byte_exactness(src, &out)?;
    Ok(Processed {
        output: Some(out),
        files,
        expanded: expanded_any,
    })
}

#[cfg(not(target_arch = "wasm32"))]
/// Runs the pipeline over one file, writing it — and every file its macros
/// authored [dartmacros.files] — only when something changed.
///
/// # Errors
///
/// Fails when the file cannot be read or written, when generation fails, or
/// when a macro-authored file would overwrite a hand-written one.
pub fn process_file(path: &Path, opts: &Options) -> Result<Outcome> {
    let src = fs::read_to_string(path)
        .with_context(|| format!("DMX1002: cannot read {}", path.display()))?;
    let processed = process_source_inner(&src, *opts, None, Some(path))?;
    // Only a pass where a macro ran may write or collect siblings: a file whose
    // macros never expanded owns nothing, and scanning its directory would be
    // pure cost at best and a deletion at worst [dartmacros.files].
    let siblings = processed.expanded && emit::emit_macro_files(path, &processed.files, opts)?;
    let seed = match processed.output {
        None => false,
        Some(out) => {
            if !opts.check {
                emit::write_atomic(path, &out)
                    .with_context(|| format!("DMX1003: cannot write {}", path.display()))?;
            }
            true
        }
    };
    Ok(if seed || siblings {
        Outcome::Updated
    } else {
        Outcome::Unchanged
    })
}

#[cfg(target_arch = "wasm32")]
const PLAYGROUND_OPTIONS: Options = Options {
    insert_regions: true,
    check: false,
};

#[cfg(target_arch = "wasm32")]
fn playground_result(source: &str, result: Result<Processed>) -> js_sys::Array {
    // `files` is structurally empty here: only the Dart macro worker authors
    // files [dartmacros.files], and the worker does not exist on wasm.
    let (ok, value) = match result.map(|processed| processed.output) {
        Ok(Some(generated)) => (true, generated),
        Ok(None) => (true, source.to_owned()),
        Err(error) => (false, format!("{error:#}")),
    };
    js_sys::Array::of2(
        &wasm_bindgen::JsValue::from_bool(ok),
        &wasm_bindgen::JsValue::from_str(&value),
    )
}

/// Runs the production generator in a browser without converting failures into
/// JavaScript exceptions [playground.wasm].
///
/// The two-element result is `[ok, source_or_diagnostic]`: on success the
/// second value is the generated Dart (or the unchanged input), and on failure
/// it is the complete diagnostic chain.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn generate(source: &str) -> js_sys::Array {
    playground_result(source, process_source(source, &PLAYGROUND_OPTIONS))
}

/// Runs the production generator against an exact caller-supplied Mustache
/// template without converting failures into JavaScript exceptions
/// [playground.wasm].
///
/// The source must contain exactly one registered macro annotation. Its normal
/// Rust context builder supplies the template variables. The two-element result
/// has the same `[ok, source_or_diagnostic]` contract as [`generate`].
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn generate_with_template(source: &str, template: &str) -> js_sys::Array {
    playground_result(
        source,
        process_source_with_template(source, template, &PLAYGROUND_OPTIONS),
    )
}

#[cfg(test)]
mod tests {
    use super::{Options, process_source, process_source_with_template};

    const SOURCE: &str = include_str!("../tests/golden/plain.dart");
    const OPTIONS: Options = Options {
        insert_regions: true,
        check: false,
    };

    /// [playground.wasm]: user Dart and Mustache use the real model context and
    /// inline validation pipeline.
    #[test]
    fn user_template_replaces_the_inferred_macro_template() {
        let template = "{{#fields}}\n  String get {{name}}WireName => {{{jsonKey}}};\n{{/fields}}";
        let output = process_source_with_template(SOURCE, template, &OPTIONS)
            .expect("custom template pipeline")
            .output
            .expect("custom template output");

        assert!(output.contains("String get idWireName => 'id';"));
        assert!(output.contains("String get countWireName => 'count';"));
        assert!(!output.contains("static Result<Plain, DecodeError> fromJson"));
        assert!(output.contains("//#region"));
    }

    /// [playground.wasm]: a user template has exactly one real macro context.
    #[test]
    fn user_template_refuses_an_ambiguous_context() {
        let source = SOURCE.replacen("@dmx('model')", "@dmx('model')\n@dmx('diff')", 1);
        let error = process_source_with_template(&source, "{{className}}", &OPTIONS)
            .expect_err("two registered annotations must be ambiguous");

        assert!(format!("{error:#}").contains("DMX4002"));
        assert!(format!("{error:#}").contains("found 2"));
    }

    /// [playground.wasm]: template parse failures are typed diagnostics and do
    /// not leak the override into ordinary generation.
    #[test]
    fn bad_user_template_is_diagnostic_and_restores_the_builtin() {
        let error = process_source_with_template(SOURCE, "{{> unavailable}}", &OPTIONS)
            .expect_err("unsupported user partial must fail");
        assert!(format!("{error:#}").contains("bad user template"));

        let built_in = process_source(SOURCE, &OPTIONS)
            .expect("built-in pipeline after custom error")
            .output
            .expect("built-in output after custom error");
        assert!(built_in.contains("static Result<Plain, DecodeError> fromJson"));
    }
}
