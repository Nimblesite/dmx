//! `@dmx('union')` [catalogue.macros] — a tagged sum type that decodes.
//!
//! The macro that proves the front end is relational: `@dmx('union')` on the sealed
//! base reads its *sibling declarations* to find the variants — no type
//! resolver, no analyzer, no build graph. The file is the scope and a name is a
//! name [frontend.name-index].

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawAnnotation, RawDecl};
use crate::macros;
use crate::render;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/union.mustache");

/// The discriminator key when `@dmx('union')` does not name one.
const DEFAULT_DISCRIMINATOR: &str = "type";

#[derive(Content)]
/// One variant of the union, as the template names its parts.
pub struct VariantCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The discriminator key, as a Dart string literal. Repeated per variant
    /// because a mustache section is its own context.
    pub discriminator: String,
    /// This variant's tag, as a Dart string literal.
    pub tag: String,
    /// The same tag unquoted, for interpolation into a decode path.
    pub tagText: String,
    /// The `when`/`maybeWhen` parameter that handles this variant.
    pub param: String,
    /// The `isX` predicate this generates.
    pub isName: String,
    /// The `asX` narrowing accessor this generates.
    pub asName: String,
}

#[derive(Content)]
/// The whole context `union.mustache` renders against.
pub struct UnionCtx {
    /// The sealed base the variants share.
    pub unionName: String,
    /// The discriminator key, as a Dart string literal.
    pub discriminator: String,
    /// The same key unquoted, for interpolation into a path.
    pub discriminatorText: String,
    /// What the fall-through arm binds the unrecognised tag to.
    pub discriminatorBind: String,
    /// Every sibling that extends or implements the base, in source order.
    pub variants: Vec<VariantCtx>,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "union")?;
    render::render(TEMPLATE, &build(decl, file)?)
}

/// The discriminator key `@dmx('union')` dispatches on.
pub fn discriminator(union: &RawAnnotation) -> String {
    union
        .arg("discriminator")
        .map_or_else(|| DEFAULT_DISCRIMINATOR.to_owned(), casing::unquote)
}

/// The tag a variant carries, under the union's rename policy.
pub fn tag(union: &RawAnnotation, variant: &str) -> String {
    match union.arg("fieldRename").map(casing::unquote) {
        Some(policy) => casing::rename(&policy, variant),
        None => casing::camel(variant),
    }
}

/// Whether this declaration is one of `base`'s variants.
fn is_variant_of(decl: &RawDecl, base: &str) -> bool {
    decl.extends.as_deref() == Some(base) || decl.interfaces.iter().any(|name| name == base)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<UnionCtx> {
    let union = decl
        .annotation("union")
        .context("DMX2000: internal error — reached the union builder without @dmx('union')")?;
    let key = discriminator(union);

    let variants: Vec<VariantCtx> = file
        .iter()
        .filter(|candidate| is_variant_of(candidate, &decl.name))
        .map(|variant| variant_context(union, &key, &variant.name))
        .collect();
    if variants.is_empty() {
        // An empty `switch (this)` does not compile, and "no variants" is the
        // author's own mistake to hear about in their terms [diagnostics].
        bail!(
            "DMX2005: `@dmx('union')` on `{}` found no variants in this file; a variant \
             is a declaration that `extends {0}` or `implements {0}` \
             [frontend.name-index]",
            decl.name
        );
    }

    Ok(UnionCtx {
        unionName: decl.name.clone(),
        discriminator: casing::dart_string(&key),
        // The tag binding never collides: a Dart pattern variable and a class
        // name live in different namespaces, and the arm uses nothing else.
        discriminatorBind: casing::camel(&key),
        discriminatorText: key,
        variants,
    })
}

/// Everything the template names about one variant.
fn variant_context(union: &RawAnnotation, key: &str, name: &str) -> VariantCtx {
    let tag = tag(union, name);
    VariantCtx {
        discriminator: casing::dart_string(key),
        param: casing::camel(name),
        isName: format!("is{}", casing::pascal(name)),
        asName: format!("as{}", casing::pascal(name)),
        tag: casing::dart_string(&tag),
        tagText: tag,
        name: name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, refusal, rendered};

    const ORDERS: &str = "@dmx('union', {'discriminator': 'type', 'fieldRename': 'snake'}) \
                          sealed class OrderState {}\n\
                          final class Draft extends OrderState {}\n\
                          final class AwaitingPayment extends OrderState {}";

    /// [frontend.name-index]: the variants are the siblings that extend the base.
    #[test]
    fn variants_are_found_by_name_in_the_file() {
        emits(
            &rendered(expand, ORDERS),
            &[
                "{ 'type': 'draft' } => Draft.fromJson(json, '$path(draft)'),",
                "{ 'type': 'awaiting_payment' } => \
                 AwaitingPayment.fromJson(json, '$path(awaiting_payment)'),",
            ],
        );
    }

    /// The tag follows the rename policy; the handler stays a Dart identifier.
    #[test]
    fn a_handler_is_named_for_the_variant_not_for_its_tag() {
        emits(
            &rendered(expand, ORDERS),
            &[
                "required T Function(AwaitingPayment value) awaitingPayment,",
                "bool get isAwaitingPayment => this is AwaitingPayment;",
                "AwaitingPayment? get asAwaitingPayment",
            ],
        );
    }

    /// An unrecognised tag fails as data, naming where it was found.
    #[test]
    fn an_unrecognised_tag_fails_at_the_discriminator() {
        emits(
            &rendered(expand, ORDERS),
            &["Err(DecodeError('$path.type', 'OrderState', type)),"],
        );
    }

    /// `implements` is as good as `extends` for finding a variant.
    #[test]
    fn an_implementing_sibling_is_a_variant() {
        emits(
            &rendered(
                expand,
                "@dmx('union') sealed class S {}\nfinal class A implements S {}",
            ),
            &["{ 'type': 'a' } => A.fromJson(json, '$path(a)'),"],
        );
    }

    /// [diagnostics]: an empty `switch (this)` would not compile, so the
    /// author hears about it in their own terms instead.
    #[test]
    fn a_union_with_no_variants_is_refused() {
        let err = refusal(expand, "@dmx('union') sealed class Alone {}");
        assert!(err.contains("DMX2005"), "{err}");
    }
}
