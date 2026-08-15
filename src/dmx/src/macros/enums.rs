//! `@dmx('enum')` [catalogue.macros] — an enum that survives the wire.
//!
//! Dart gives an enum `.name` and `.values` and stops. Crossing a network
//! boundary needs a wire name that is not the Dart identifier, a human label
//! that is neither, a decode that fails as data rather than throwing, and —
//! for enums somebody else owns — an `unknown:` fallback so a constant added
//! upstream after this build shipped is data, not an outage.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl, RawValue};
use crate::macros;
use crate::render;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/enum.mustache");

#[derive(Content)]
/// One enum constant, as the template names its parts.
pub struct ValueCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The wire name, as a Dart string literal.
    pub wire: String,
    /// The human label, as a Dart string literal.
    pub label: String,
    /// The `isX` predicate this generates.
    pub isName: String,
}

#[derive(Content)]
/// The whole context `enum.mustache` renders against.
pub struct EnumCtx {
    /// The enum the members are generated into.
    pub enumName: String,
    /// The enum constants, in source order.
    pub values: Vec<ValueCtx>,
    /// `unknown:` was given, so an unrecognised wire name decodes rather than
    /// failing.
    pub hasFallback: bool,
    /// The constant an unrecognised wire name decodes to.
    pub fallback: String,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, _file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Enum, "enum")?;
    render::render(TEMPLATE, &build(decl)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl) -> Result<EnumCtx> {
    let annotation = decl
        .annotation("enum")
        .context("DMX2000: internal error — reached the enum builder without @dmx('enum')")?;
    let policy = annotation.arg("fieldRename").map(casing::unquote);
    // An expression, not a string: the author writes `unknown: Reason.other`,
    // which is the constant itself and reaches the template verbatim.
    let fallback = annotation.arg("unknown").map(str::to_owned);

    Ok(EnumCtx {
        enumName: decl.name.clone(),
        values: decl
            .values
            .iter()
            .map(|value| value_context(value, policy.as_deref()))
            .collect(),
        hasFallback: fallback.is_some(),
        fallback: fallback.unwrap_or_default(),
    })
}

/// One constant: its wire name, its label, and its predicate.
fn value_context(value: &RawValue, policy: Option<&str>) -> ValueCtx {
    let annotation = value.annotations.iter().find(|a| a.name == "value");
    let wire = match annotation.and_then(|v| v.arg("wire")) {
        // `@dmx('value', {'wire': })` pins the one constant the provider named differently.
        Some(pinned) => casing::unquote(pinned),
        None => match policy {
            Some(policy) => casing::rename(policy, &value.name),
            None => value.name.clone(),
        },
    };
    let label = match annotation.and_then(|v| v.arg("label")) {
        Some(explicit) => casing::unquote(explicit),
        None => casing::label(&value.name),
    };

    ValueCtx {
        isName: format!("is{}", casing::pascal(&value.name)),
        wire: casing::dart_string(&wire),
        label: casing::dart_string(&label),
        name: value.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{Case, each, emits, omits, refusal, rendered};

    const CASES: &[Case<'_>] = &[
        (
            "[surface.annotations]: the policy renames every constant at once, and \
             an explicit `@dmx('value', {'wire': })` still wins over it",
            "@dmx('enum', {'fieldRename': 'snake'}) enum M { card, applePay, \
             @dmx('value', {'wire': 'ach_transfer'}) bankTransfer; }",
            &[
                "M.applePay => 'apple_pay',",
                "M.bankTransfer => 'ach_transfer',",
                "'ach_transfer' => M.bankTransfer,",
            ],
        ),
        (
            "`screaming_snake` is a policy someone will write; it must be one",
            "@dmx('enum', {'fieldRename': 'screaming_snake'}) enum M { itemNotReceived; }",
            &["M.itemNotReceived => 'ITEM_NOT_RECEIVED',"],
        ),
        (
            "a label is for a person to read, and an explicit one is left alone",
            "@dmx('enum') enum M { applePay, @dmx('value', {'label': 'Gift card'}) giftCard; }",
            &["M.applePay => 'Apple pay',", "M.giftCard => 'Gift card',"],
        ),
        (
            "without a fallback, an unrecognised value is a decode failure \
             carrying the value",
            "@dmx('enum') enum M { a, b; }",
            &[
                "null => Err(DecodeError(path, 'M', value)),",
                "bool get isA => this == M.a;",
            ],
        ),
    ];

    #[test]
    fn every_case_emits_what_it_names() {
        each(expand, CASES);
    }

    /// [catalogue.macros]: `unknown:` makes the decode total — which is as much
    /// about the failure arm no longer being emitted as about the fallback.
    #[test]
    fn an_unknown_fallback_makes_the_decode_total() {
        let out = rendered(
            expand,
            "@dmx('enum', {'unknown': M.other}) enum M { a, other; }",
        );
        emits(&out, &["Ok(tryParse(value) ?? M.other)"]);
        omits(&out, &["null => Err(DecodeError(path, 'M', value)),"]);
    }

    /// [emission.inline-backend.insertion]: an enum body only admits members
    /// after a `;`, and saying so beats a compile error in generated code.
    #[test]
    fn an_unterminated_enum_is_refused_with_the_fix() {
        let err = refusal(expand, "@dmx('enum') enum M { a, b }");
        assert!(err.contains("DMX2004"), "{err}");
    }
}
