//! The macro registry [catalogue].
//!
//! A macro is an annotation name, a context builder, and a template. Nothing
//! else — no lifecycle, no plugin API, no registration ceremony. Adding one is
//! adding a row to [`REGISTRY`] and a file to `src/dmx/templates/`.
//!
//! Every macro receives the declaration it is attached to *and the whole file*,
//! because the interesting ones are relational: `@dmx('union')` reads its variants,
//! `@dmx('fake')` reads the interface it implements [frontend.name-index]. Resolution
//! is by name, never by type inference [frontend.no-type-inference].
//!
//! Several macros may sit on one declaration. Each contributes a fragment, and
//! the fragments emit in the order the author wrote the annotations
//! [rendering], into the single region that declaration owns.

mod cli;
mod diff;
mod enums;
mod fake;
mod lerp;
mod model;
mod rest;
mod route;
mod table;
#[cfg(test)]
mod testing;
mod union;
mod validate;

use anyhow::{Context as _, Result, bail};

use crate::frontend::{Annotated, DeclKind, RawDecl, RawField};
use crate::types::DartType;

/// What every macro is: a trigger and a way to render.
struct MacroDef {
    /// The annotation that triggers it, without the `@`.
    annotation: &'static str,
    /// Builds its context and renders its template.
    expand: fn(&RawDecl, &[RawDecl]) -> Result<String>,
}

/// Order here is documentation only — a declaration's fragments follow the
/// order its *annotations* were written, so the author controls the output.
const REGISTRY: &[MacroDef] = &[
    MacroDef {
        annotation: "model",
        expand: model::expand,
    },
    MacroDef {
        annotation: "union",
        expand: union::expand,
    },
    MacroDef {
        annotation: "enum",
        expand: enums::expand,
    },
    MacroDef {
        annotation: "diff",
        expand: diff::expand,
    },
    MacroDef {
        annotation: "lerp",
        expand: lerp::expand,
    },
    MacroDef {
        annotation: "validate",
        expand: validate::expand,
    },
    MacroDef {
        annotation: "table",
        expand: table::expand,
    },
    MacroDef {
        annotation: "route",
        expand: route::expand,
    },
    MacroDef {
        annotation: "cli",
        expand: cli::expand,
    },
    MacroDef {
        annotation: "fake",
        expand: fake::expand,
    },
    MacroDef {
        annotation: "restClient",
        expand: rest::expand,
    },
];

/// Everything one declaration's macros produced: the joined region body, and
/// any whole sibling files a user macro authored [dartmacros.files].
#[derive(Debug)]
pub struct Expanded {
    /// The region body, fragments separated by a blank line.
    pub text: String,
    /// Macro-authored sibling files, in the order the macros returned them.
    pub files: Vec<crate::emit::GeneratedFile>,
}

/// The generated body for one declaration, or `None` when no macro applies.
///
/// Fragments are separated by a blank line, which [`crate::render::normalize`]
/// has already guaranteed does not appear inside one. `origin` is the file the
/// declaration came from, which is how the Dart worker serving it is found
/// [dartmacros.discovery]; `None` where the caller generated from a string.
///
/// # Errors
///
/// Fails when a macro refuses the declaration, carrying the macro's own
/// diagnostic [diagnostics].
pub fn expand(
    decl: &RawDecl,
    file: &[RawDecl],
    origin: Option<&std::path::Path>,
) -> Result<Option<Expanded>> {
    let mut fragments = Vec::new();
    // Only the native Dart macro worker can author whole files; the wasm
    // playground has no worker, so the binding never mutates there
    // [dartmacros.files].
    #[cfg(not(target_arch = "wasm32"))]
    let mut files = Vec::new();
    #[cfg(target_arch = "wasm32")]
    let files = Vec::new();
    // No worker on wasm, so nothing there needs to know where the source came
    // from [playground.wasm].
    #[cfg(target_arch = "wasm32")]
    let _ = origin;
    for annotation in &decl.annotations {
        if !annotation.dmx {
            continue;
        }
        if annotation.name == "dmx" {
            bail!(
                "DMX2005: `@dmx` on `{}` needs its macro as a string literal, \
                 e.g. `@dmx('model')`",
                decl.name
            );
        }
        if let Some(def) = REGISTRY.iter().find(|m| m.annotation == annotation.name) {
            fragments.push((def.expand)(decl, file).with_context(|| {
                format!("DMX2100: `@dmx('{}')` on `{}`", def.annotation, decl.name)
            })?);
            continue;
        }
        // Not a built-in: offer it to the project's Dart macro worker, which
        // stays inert when absent or when it does not serve the name
        // [dartmacros.resolution].
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(expansion) = crate::dartmacros::expand(annotation, decl, origin)? {
            fragments.push(expansion.text);
            files.extend(expansion.files);
        }
    }
    if fragments.is_empty() {
        return Ok(None);
    }
    Ok(Some(Expanded {
        text: fragments.join("\n\n"),
        files,
    }))
}

/// Whether `name` is a built-in macro, which a user macro may never shadow
/// [dartmacros.resolution].
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    REGISTRY.iter().any(|m| m.annotation == name)
}

/// Whether any macro — built-in or potentially user-defined — triggers on
/// this declaration. Any class-level `@dmx` qualifies: an unregistered name
/// may be served by the project's Dart worker [dartmacros.discovery], and one
/// nothing serves expands to no fragments and leaves the file untouched.
#[must_use]
pub fn applies_to(decl: &RawDecl) -> bool {
    decl.annotations.iter().any(|a| a.dmx)
}

/// Counts registered macro annotations across a parsed file
/// [playground.wasm].
///
/// Unlike [`applies_to`], this counts two registered annotations on one
/// declaration as two applications. A caller-supplied template is ambiguous in
/// that case because each macro exposes a different context.
pub(crate) fn application_count(declarations: &[RawDecl]) -> usize {
    declarations
        .iter()
        .flat_map(|declaration| &declaration.annotations)
        .filter(|annotation| {
            annotation.dmx
                && REGISTRY
                    .iter()
                    .any(|definition| definition.annotation == annotation.name)
        })
        .count()
}

/// A field with its type already parsed — the starting point of nearly every
/// context builder, so none of them repeats the parse or its diagnostic.
#[derive(Debug)]
pub struct Field<'a> {
    /// The declaration as the front end read it.
    pub raw: &'a RawField,
    /// Its type, already parsed.
    pub ty: DartType,
}

impl Field<'_> {
    /// The field's own name, as the author wrote it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.raw.name
    }
}

/// Every field of `decl`, minus the ones `@dmx('key', {'ignore': true})` excludes.
///
/// # Errors
///
/// Fails when a field has no explicit type, or a type no codec covers.
pub fn typed_fields(decl: &RawDecl) -> Result<Vec<Field<'_>>> {
    decl.fields
        .iter()
        .filter(|f| {
            !f.annotation("key")
                .and_then(|k| k.flag("ignore"))
                .unwrap_or(false)
        })
        .map(|raw| {
            let text = raw
                .type_text
                .as_ref()
                .with_context(|| format!("DMX2001: field `{}` needs an explicit type", raw.name))?;
            let ty = DartType::parse(text)
                .with_context(|| format!("DMX2102: field `{}` in `{}`", raw.name, decl.name))?;
            Ok(Field { raw, ty })
        })
        .collect()
}

/// Refuses a declaration a macro cannot mean anything for, in the author's
/// terms rather than as a compile error in generated code [diagnostics].
///
/// # Errors
///
/// Fails when the declaration is the wrong kind for the macro, or is an enum
/// whose constants are not terminated by a `;`.
pub fn require(decl: &RawDecl, kind: DeclKind, macro_name: &str) -> Result<()> {
    if decl.kind != kind {
        bail!(
            "DMX2003: `@dmx('{macro_name}')` applies to a {}, but `{}` is {}",
            match kind {
                DeclKind::Class => "class",
                DeclKind::Enum => "enum",
            },
            decl.name,
            match decl.kind {
                DeclKind::Class => "a class",
                DeclKind::Enum => "an enum",
            }
        );
    }
    if decl.kind == DeclKind::Enum && !decl.values_terminated {
        bail!(
            "DMX2004: enum `{}` needs a `;` after its constants before members \
             can be generated:\n\n  enum {0} {{ a, b; }}",
            decl.name
        );
    }
    Ok(())
}

/// First candidate that no field shadows. Generated parameters are readable
/// names, never hash-mangled; only a real collision forces the next candidate.
#[must_use]
pub fn fresh_name<'a>(candidates: &[&'a str], taken: &[String]) -> &'a str {
    candidates
        .iter()
        .find(|c| !taken.iter().any(|t| t == *c))
        .or_else(|| candidates.last())
        .copied()
        // An empty candidate list is a caller's bug, not a name collision, and
        // an obviously wrong identifier reports it better than a panic.
        .unwrap_or("value")
}

/// `when` opens a guard clause, so a pattern can never bind it; `_` is the
/// wildcard. Everything else keeps the author's name.
#[must_use]
pub fn binding_name(name: &str) -> String {
    match name {
        "when" | "_" => format!("{name}Value"),
        _ => name.to_owned(),
    }
}

/// A value as the `String?` a query map takes [catalogue.macros].
///
/// Shared by `@dmx('route')` and `@dmx('restClient')`, because a query parameter is a query
/// parameter whether the URL is being built for a screen or for a server.
#[must_use]
pub fn query_string(ty: &DartType, name: &str) -> String {
    match ty.non_null().name.as_str() {
        "String" => name.to_owned(),
        _ if ty.is_declared() && ty.nullable => format!("{name}?.toJson()"),
        _ if ty.is_declared() => format!("{name}.toJson()"),
        _ if ty.nullable => format!("{name}?.toString()"),
        // Interpolation, not `toString()`: a segment ends at the quote, so the
        // simple form cannot run into what follows it.
        _ => format!("'${name}'"),
    }
}

/// The record patterns that select each failing decode of `arity` in turn:
/// `(Err(error: final e), _)`, then `(_, Err(error: final e))`.
///
/// A one-element record needs its trailing comma or it is a parenthesised
/// expression instead — which is the kind of thing a template must never be
/// asked to know [context.discipline].
#[must_use]
pub fn error_patterns(arity: usize) -> Vec<String> {
    slot_patterns(arity, "Err(error: final e)")
}

/// The record patterns that put `marker` in each slot of `arity` in turn, and
/// a wildcard in the rest.
#[must_use]
pub fn slot_patterns(arity: usize, marker: &str) -> Vec<String> {
    (0..arity)
        .map(|slot| {
            let slots: Vec<&str> = (0..arity)
                .map(|i| if i == slot { marker } else { "_" })
                .collect();
            format!(
                "({}{})",
                slots.join(", "),
                if arity == 1 { "," } else { "" }
            )
        })
        .collect()
}

/// The sibling `decl` extends or implements that carries `annotation`
/// [frontend.name-index].
///
/// Resolution is by name, in this file, and nowhere else: a base in another
/// library is not one of these, because nothing here can see what it declares.
#[must_use]
pub fn base_with<'a>(decl: &RawDecl, file: &'a [RawDecl], annotation: &str) -> Option<&'a RawDecl> {
    file.iter().find(|base| {
        base.annotation(annotation).is_some()
            && (decl.extends.as_deref() == Some(base.name.as_str())
                || decl.interfaces.contains(&base.name))
    })
}

/// Marks the last element, so a template can lay out separators without
/// arithmetic [context.discipline].
pub fn mark_last<T>(items: &mut [T], set: impl Fn(&mut T)) {
    if let Some(last) = items.last_mut() {
        set(last);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::Frontend;

    fn decls(src: &str) -> Vec<RawDecl> {
        Frontend::new().unwrap().declarations(src).unwrap()
    }

    /// Generated parameters are readable, and only collide-then-rename.
    #[test]
    fn parameter_names_are_readable() {
        assert_eq!(fresh_name(&["other", "that"], &["id".into()]), "other");
        assert_eq!(fresh_name(&["other", "that"], &["other".into()]), "that");
    }

    /// An unannotated declaration is not dmx's business at all.
    #[test]
    fn only_annotated_declarations_expand() {
        let file = decls("class Plain { final int a; }\n@dmx('model') class M { final int a; }");
        assert!(!applies_to(&file[0]));
        assert!(applies_to(&file[1]));
        assert!(expand(&file[0], &file, None).unwrap().is_none());
    }

    /// [rendering]: fragments follow the order the annotations were written.
    #[test]
    fn several_macros_compose_in_source_order() {
        let file = decls("@dmx('model') @dmx('diff') class M { final int a; }");
        let out = expand(&file[0], &file, None).unwrap().unwrap().text;
        let model = out.find("copyWith").expect("model fragment");
        let diff = out.find("diff(").expect("diff fragment");
        assert!(model < diff, "annotation order decides fragment order");

        let file = decls("@dmx('diff') @dmx('model') class M { final int a; }");
        let out = expand(&file[0], &file, None).unwrap().unwrap().text;
        assert!(out.find("diff(") < out.find("copyWith"));
    }

    /// [diagnostics]: the wrong target is refused in the author's terms.
    #[test]
    fn macros_refuse_the_wrong_declaration_kind() {
        let file = decls("@dmx('enum') class NotAnEnum {}");
        let err = expand(&file[0], &file, None).unwrap_err().to_string();
        assert!(err.contains("DMX2100"), "{err}");
    }

    /// An enum body only admits members after a `;` [emission.inline-backend.insertion].
    #[test]
    fn enum_without_a_terminator_is_refused_with_the_fix() {
        let file = decls("@dmx('enum') enum E { a, b }");
        let err = format!("{:#}", expand(&file[0], &file, None).unwrap_err());
        assert!(
            err.contains("DMX2004") && err.contains("enum E { a, b; }"),
            "{err}"
        );
    }
}
