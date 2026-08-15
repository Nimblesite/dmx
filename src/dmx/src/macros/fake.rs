//! `@dmx('fake')` [catalogue.macros] — deterministic fixtures, no mock library.
//!
//! There is no randomness here on purpose: every value comes from the seed by
//! arithmetic, so a failing test fails identically on the next run, on CI, and
//! on the machine of whoever picks it up. Nullable fields default to null — a
//! fixture is the *simplest* valid value, not the fullest.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field};
use crate::render;
use crate::types::DartType;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/fake.mustache");
/// The template an enum's fixture renders.
const ENUM_TEMPLATE: &str = include_str!("../../templates/fake_enum.mustache");

#[derive(Content)]
/// One field, as the template names its parts.
pub struct FieldCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The override parameter, always optional so a test states only what the
    /// test is about.
    pub param: String,
    /// The override, or the value the seed produces.
    pub value: String,
}

/// An enum needs no field list: its fixture is one of its own constants.
#[derive(Content)]
pub struct FakeEnumCtx {
    /// The enum's name.
    pub enumName: String,
    /// Where the walk over the constants starts.
    pub seed: String,
}

#[derive(Content)]
/// The whole context `fake.mustache` renders against.
pub struct FakeCtx {
    /// The class the members are generated into.
    pub className: String,
    /// Where the deterministic value sequence starts.
    pub seed: String,
    /// Every field the macro generates for, in source order.
    pub fields: Vec<FieldCtx>,
    /// The class also encodes, so a fixture can be handed to a decoder.
    pub wantsJson: bool,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    // An enum is fakeable too, and has to be: a fixture reaches types from
    // other files, where `.fake(seed:)` is the only thing it can rely on
    // [frontend.name-index].
    match decl.kind {
        DeclKind::Enum => {
            macros::require(decl, DeclKind::Enum, "fake")?;
            render::render(ENUM_TEMPLATE, &enum_context(decl)?)
        }
        DeclKind::Class => render::render(TEMPLATE, &build(decl, file)?),
    }
}

/// An enum's fixture, which needs no field list at all.
fn enum_context(decl: &RawDecl) -> Result<FakeEnumCtx> {
    let annotation = decl
        .annotation("fake")
        .context("DMX2000: internal error — reached the fake builder without @dmx('fake')")?;
    if decl.values.is_empty() {
        bail!(
            "DMX2023: `@dmx('fake')` on `{}` found no constants, so there is nothing \
             to pick a fixture from",
            decl.name
        );
    }
    Ok(FakeEnumCtx {
        enumName: decl.name.clone(),
        seed: annotation.arg("seed").unwrap_or("0").to_owned(),
    })
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<FakeCtx> {
    let annotation = decl
        .annotation("fake")
        .context("DMX2000: internal error — reached the fake builder without @dmx('fake')")?;

    let mut fields = Vec::new();
    for (offset, field) in macros::typed_fields(decl)?.iter().enumerate() {
        let name = field.name().to_owned();
        // Every override is optional, so its type is nullable whatever the
        // field's is; a nullable field then simply has no value to fall back on.
        let param = format!("{}? {name}", field.ty.non_null().source);
        let value = if field.ty.nullable {
            name.clone()
        } else {
            format!("{name} ?? {}", value(field, offset, file)?)
        };
        fields.push(FieldCtx { name, param, value });
    }

    Ok(FakeCtx {
        className: decl.name.clone(),
        seed: annotation.arg("seed").unwrap_or("0").to_owned(),
        // A fixture that can also be handed to a decoder is worth twice as
        // much, but only where there is a decoder to hand it to.
        wantsJson: decl
            .annotation("model")
            .is_some_and(|model| model.flag("json").unwrap_or(true)),
        fields,
    })
}

/// The value the seed produces for one field.
///
/// `offset` keeps two fields of the same type from colliding: every field
/// reads a different point of the same arithmetic sequence.
fn value(field: &Field<'_>, offset: usize, file: &[RawDecl]) -> Result<String> {
    let ty = &field.ty;
    let step = format!("seed + {offset}");
    if let Some(element) = ty.args.first() {
        let inner = seeded(element, &step, field.name(), file)?;
        return Ok(match ty.name.as_str() {
            "List" | "Iterable" => format!("<{}>[{inner}]", element.source),
            "Set" => format!("<{}>{{{inner}}}", element.source),
            "Map" => match ty.args.as_slice() {
                [_, values] => format!(
                    "<String, {}>{{'key-${{{step}}}': {}}}",
                    values.source,
                    seeded(values, &step, field.name(), file)?
                ),
                _ => bail!("DMX2102: `{}` needs exactly two type arguments", ty.source),
            },
            _ => bail!("DMX2018: no fixture shape for `{}`", ty.source),
        });
    }
    seeded(ty, &step, field.name(), file)
}

/// One scalar value derived from `step`.
fn seeded(ty: &DartType, step: &str, name: &str, file: &[RawDecl]) -> Result<String> {
    let bare = ty.non_null();
    Ok(match bare.name.as_str() {
        // An address that is not an address fails a validator somewhere, and
        // the failure reads as the generator's bug rather than the test's.
        "String" if name.to_lowercase().contains("email") => {
            format!("'{name}-${{{step}}}@example.test'")
        }
        "String" => format!("'{name}-${{{step}}}'"),
        "int" | "num" => format!("({step})"),
        "double" => format!("({step}).toDouble()"),
        "bool" => format!("({step}).isEven"),
        // A fixed epoch plus a walk, so a fixture is a date and still stable.
        "DateTime" => format!("DateTime.utc(2024).add(Duration(days: ({step}) % 365))"),
        "Duration" => format!("Duration(seconds: {step})"),
        "Uri" => format!("Uri.parse('https://example.test/${{{step}}}')"),
        other => match sibling(other, file) {
            Some(decl) if decl.kind == DeclKind::Enum => {
                format!("{other}.values[({step}) % {other}.values.length]")
            }
            // A sibling that says nothing about fixtures has none, and calling
            // one it does not have is worse than saying so [diagnostics].
            Some(decl) if decl.annotation("fake").is_none() => bail!(
                "DMX2018: `{name}` is a `{other}`, which has no fixture; give \
                 `{other}` its own `@dmx('fake')` so this one can compose with it \
                 [frontend.name-index]"
            ),
            // From another file, `.fake(seed:)` is the whole contract: a type
            // that carries `@dmx('fake')` has one, whatever kind of thing it is.
            _ => format!("{other}.fake(seed: {step})"),
        },
    })
}

/// The declaration of this name in this file, if there is one.
fn sibling<'a>(name: &str, file: &'a [RawDecl]) -> Option<&'a RawDecl> {
    file.iter().find(|decl| decl.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{Case, each, emits, omits, refusal, rendered};

    const CASES: &[Case<'_>] = &[
        (
            "every value comes from the seed by arithmetic, and no two fields \
             share a point in the sequence",
            "@dmx('fake', {'seed': 42}) class C { final String id; final int points; \
             final bool isVip; final DateTime joinedAt; }",
            &[
                "int seed = 42,",
                "id: id ?? 'id-${seed + 0}',",
                "points: points ?? (seed + 1),",
                "isVip: isVip ?? (seed + 2).isEven,",
                "joinedAt: joinedAt ?? DateTime.utc(2024).add(Duration(days: (seed + 3) % 365)),",
            ],
        ),
        (
            "a fixture is the simplest valid value, so a nullable field is null",
            "@dmx('fake') class C { final String? referredBy; }",
            &["String? referredBy,", "referredBy: referredBy,"],
        ),
        (
            "[frontend.name-index]: fixtures compose through siblings",
            "@dmx('fake') class C { final Address home; final Method paid; }\n\
             @dmx('fake') class Address { final String street; }\n\
             enum Method { card, cash }",
            &[
                "home: home ?? Address.fake(seed: seed + 0),",
                "paid: paid ?? Method.values[(seed + 1) % Method.values.length],",
            ],
        ),
        (
            "a collection is one element deep — enough to exercise the codec",
            "@dmx('fake') class C { final List<String> tags; }",
            &["tags: tags ?? <String>['tags-${seed + 0}'],"],
        ),
        (
            "an encodable fixture can be handed straight to a decoder",
            "@dmx('model') @dmx('fake') class C { final String id; }",
            &["static Map<String, dynamic> fakeJson({int seed = 0})"],
        ),
        (
            "a type from another file is reached by the one thing `@dmx('fake')` \
             promises: nothing here can see its declaration, but every fixture \
             has this shape",
            "@dmx('fake') class C { final PaymentMethod paid; }",
            &["paid: paid ?? PaymentMethod.fake(seed: seed + 0),"],
        ),
        (
            "an enum's own fixture walks its constants, so it wraps rather than fails",
            "@dmx('fake', {'seed': 3}) enum Method { card, cash; }",
            &[
                "static Method fake({int seed = 3}) =>",
                "Method.values[seed % Method.values.length];",
            ],
        ),
    ];

    #[test]
    fn every_case_emits_what_it_names() {
        each(expand, CASES);
    }

    /// Without `@dmx('model')` there is nothing to encode a fixture into, so no
    /// `fakeJson` is emitted at all.
    #[test]
    fn a_fixture_without_a_model_is_not_encoded() {
        omits(
            &rendered(expand, "@dmx('fake') class C { final String id; }"),
            &["fakeJson"],
        );
        emits(
            &rendered(expand, "@dmx('fake') class C { final String id; }"),
            &["static C fake("],
        );
    }

    /// [diagnostics]: a sibling that says nothing about fixtures has none, and
    /// calling one it does not have is worse than saying so.
    #[test]
    fn a_sibling_without_a_fixture_is_refused() {
        let err = refusal(
            expand,
            "@dmx('fake') class C { final Plain m; }\nclass Plain { final int x; }",
        );
        assert!(err.contains("DMX2018"), "{err}");
    }
}
