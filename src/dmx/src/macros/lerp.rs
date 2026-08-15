//! `@dmx('lerp')` [catalogue.macros] — interpolation over a design-token set.
//!
//! Every blend expression is finished here [authoring.intelligence]. The
//! interesting rule is compositional: a field whose type is a sibling carrying
//! `@dmx('lerp')` blends by calling that type's own `lerp`, so a theme animates all
//! the way down with no hand-written traversal [frontend.name-index].

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::Result;
use ramhorns::Content;

use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field};
use crate::render;
use crate::types::DartType;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/lerp.mustache");

#[derive(Content)]
/// One field, as the template names its parts.
pub struct FieldCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The finished Dart expression blending this field towards `other`.
    pub blend: String,
}

#[derive(Content)]
/// The whole context `lerp.mustache` renders against.
pub struct LerpCtx {
    /// The class the members are generated into.
    pub className: String,
    /// The parameter the comparison is against.
    pub otherParam: String,
    /// Every field the macro generates for, in source order.
    pub fields: Vec<FieldCtx>,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "lerp")?;
    render::render(TEMPLATE, &build(decl, file)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<LerpCtx> {
    let names: Vec<String> = decl.fields.iter().map(|f| f.name.clone()).collect();
    let other = macros::fresh_name(&["other", "that", "operand"], &names).to_owned();
    let fields = macros::typed_fields(decl)?
        .iter()
        .map(|field| FieldCtx {
            name: field.name().to_owned(),
            blend: blend(field, &other, file),
        })
        .collect();

    Ok(LerpCtx {
        className: decl.name.clone(),
        otherParam: other,
        fields,
    })
}

/// Whether `ty` names a declaration in this file that also carries `@dmx('lerp')`.
///
/// Resolution is by name, never by inference [frontend.no-type-inference]: a
/// type from another file is not treated as interpolable, because nothing here
/// can see whether it has a `lerp` to call.
fn lerps(ty: &DartType, file: &[RawDecl]) -> bool {
    ty.is_declared()
        && file
            .iter()
            .any(|decl| decl.name == ty.name && decl.annotation("lerp").is_some())
}

/// [catalogue.macros]: numeric fields interpolate, composite fields recurse,
/// and everything with no meaningful midpoint steps at `t = 0.5` rather than
/// pretending a halfway string exists.
fn blend(field: &Field<'_>, other: &str, file: &[RawDecl]) -> String {
    let (name, ty) = (field.name(), &field.ty);
    let pair = format!("{name}, {other}.{name}, t");
    match ty.name.as_str() {
        // A null has no midpoint, whatever its type would otherwise allow.
        _ if ty.nullable => format!("dmxLerpStep({pair})"),
        "double" => format!("dmxLerpDouble({pair})"),
        "int" => format!("dmxLerpInt({pair})"),
        "Duration" => format!("dmxLerpDuration({pair})"),
        _ if lerps(ty, file) => format!("{name}.lerp({other}.{name}, t)"),
        _ => format!("dmxLerpStep({pair})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{Case, each, omits, rendered};

    const CASES: &[Case<'_>] = &[
        (
            "[catalogue.macros]: each numeric type gets the blender that suits it, \
             and a type with no midpoint steps rather than inventing one",
            "@dmx('lerp') class T { final double a; final int b; final Duration c; \
             final String d; }",
            &[
                "a: dmxLerpDouble(a, other.a, t),",
                "b: dmxLerpInt(b, other.b, t),",
                "c: dmxLerpDuration(c, other.c, t),",
                "d: dmxLerpStep(d, other.d, t),",
            ],
        ),
        (
            "[frontend.name-index]: a sibling that is itself `@dmx('lerp')` blends by \
             recursion; a plain one has no `lerp` to call, so calling one would \
             not compile",
            "@dmx('lerp') class T { final Rgba palette; final Plain p; }\n\
             @dmx('lerp') class Rgba { final int red; }\n\
             class Plain { final int x; }",
            &[
                "palette: palette.lerp(other.palette, t),",
                "p: dmxLerpStep(p, other.p, t),",
            ],
        ),
        (
            "a null has no midpoint, so a nullable double steps like anything else",
            "@dmx('lerp') class T { final double? a; }",
            &["a: dmxLerpStep(a, other.a, t),"],
        ),
        (
            "a field named `other` would shadow the parameter, so the parameter moves",
            "@dmx('lerp') class T { final int other; }",
            &[
                "T lerp(T that, double t)",
                "other: dmxLerpInt(other, that.other, t),",
            ],
        ),
    ];

    #[test]
    fn every_case_emits_what_it_names() {
        each(expand, CASES);
    }

    /// [hygiene]: generated code never throws, casts, or asserts non-null.
    #[test]
    fn output_obeys_the_house_rules() {
        omits(
            &rendered(
                expand,
                "@dmx('lerp') class T { final double a; final Motion m; }",
            ),
            &["throw", " as ", "!", "_$"],
        );
    }
}
