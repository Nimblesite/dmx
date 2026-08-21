//! `@dmx('diff')` [catalogue.macros] — field-level structural diff, as data.
//!
//! Nothing here is reflective: the field list is fixed at generation time, so
//! adding a field adds a line on the next build and cannot be forgotten. The
//! "changed" test is the exact negation of `==` [model.equality], borrowed
//! from the model builder rather than restated, so the two cannot disagree.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::Result;
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, model};
use crate::render;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/diff.mustache");

#[derive(Content)]
/// One field, as the template names its parts.
pub struct FieldCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The Dart condition under which this field contributes a change.
    pub differs: String,
    /// The name the change reports, as a Dart string literal.
    pub key: String,
}

#[derive(Content)]
/// The whole context `diff.mustache` renders against.
pub struct DiffCtx {
    /// The class the members are generated into.
    pub className: String,
    /// The parameter the comparison is against.
    pub otherParam: String,
    /// Every field the macro generates for, in source order.
    pub fields: Vec<FieldCtx>,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, _file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "diff")?;
    render::render(TEMPLATE, &build(decl)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl) -> Result<DiffCtx> {
    // A change is reported under the name it travels by. Where the declaration
    // also carries `@dmx('model')`, that is the JSON key — a diff sent to an audit log
    // beside the encoded value must name the same field the value arrived as.
    let policy = decl
        .annotation("model")
        .and_then(|model| model.arg("fieldRename"))
        .map(casing::unquote);

    let names: Vec<String> = decl.fields.iter().map(|f| f.name.clone()).collect();
    let other = macros::fresh_name(&["other", "that", "operand"], &names).to_owned();

    let fields = macros::typed_fields(decl)?
        .iter()
        .map(|field| FieldCtx {
            differs: model::comparison(
                &field.ty,
                &other,
                field.name(),
                false,
                crate::types::Runtime::IN_CLASS,
            ),
            key: model::json_key(field, policy.as_deref()),
            name: field.name().to_owned(),
        })
        .collect();

    Ok(DiffCtx {
        className: decl.name.clone(),
        otherParam: other,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{Case, each, emits, omits, rendered};

    const CASES: &[Case<'_>] = &[
        (
            "[model.equality]: a collection changed when its *contents* changed",
            "@dmx('diff') class S { final int a; final Map<String, int> m; }",
            &["if (other.a != a)", "if (!dmxDeepEquals(other.m, m))"],
        ),
        (
            "the change names the field the way the field travels on the wire",
            "@dmx('model', {'fieldRename': 'snake'}) @dmx('diff') class S { final int onHand; }",
            &["DmxChange('on_hand', onHand, other.onHand),"],
        ),
        (
            "without `@dmx('model')` there is no rename policy, so the field's own name it is",
            "@dmx('diff') class S { final int onHand; }",
            &["DmxChange('onHand', onHand, other.onHand),"],
        ),
        (
            "a field named `other` would shadow the parameter, so the parameter moves",
            "@dmx('diff') class S { final int other; }",
            &["List<DmxChange> diff(S that)", "if (that.other != other)"],
        ),
    ];

    #[test]
    fn every_case_emits_what_it_names() {
        each(expand, CASES);
    }

    /// [surface.annotations]: an ignored field is not part of the value, so it
    /// cannot be part of what changed about it. Stated apart from the table
    /// because what matters is the absence, not the presence.
    #[test]
    fn ignored_fields_never_produce_a_change() {
        let out = rendered(
            expand,
            "@dmx('diff') class S { final int a; @dmx('key', {'ignore': true}) final int b; }",
        );
        emits(&out, &["'a'"]);
        omits(&out, &["'b'"]);
    }
}
