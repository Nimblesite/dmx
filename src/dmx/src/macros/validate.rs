//! `@dmx('validate')` [catalogue.macros] — rules next to the field they constrain.
//!
//! Validation accumulates on purpose: a form that reports one problem, gets
//! fixed, and then reports the next one is how you lose someone at checkout.
//! Every `@dmx('check.…')` compiles to a condition and a message here, so the template
//! lays out a list and evaluates nothing [authoring.intelligence].

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawAnnotation, RawDecl};
use crate::macros::{self, Field, model};
use crate::render;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/validate.mustache");

/// The annotation prefix every constraint constructor shares.
const CHECK: &str = "check.";

#[derive(Content)]
/// One constraint, resolved to the test it performs.
pub struct RuleCtx {
    /// The Dart condition under which this rule is violated.
    pub condition: String,
    /// The field the violation is reported against, as a Dart string literal.
    pub field: String,
    /// What to tell a person, as a Dart string literal.
    pub message: String,
}

#[derive(Content)]
/// The whole context `validate.mustache` renders against.
pub struct ValidateCtx {
    /// The class the members are generated into.
    pub className: String,
    /// Every rule on every field, in field then annotation order.
    pub rules: Vec<RuleCtx>,
}

/// One constraint, already resolved to the test it performs.
enum Constraint {
    /// A string, list, map, or set that must not be empty.
    NotEmpty,
    /// A string no shorter than this.
    MinLength(String),
    /// A string no longer than this.
    MaxLength(String),
    /// A number no smaller than this.
    AtLeast(String),
    /// A number no larger than this.
    AtMost(String),
    /// A boolean that must be set.
    IsTrue,
    /// A string matching this regular expression.
    Matches(String),
}

impl Constraint {
    /// The Dart condition that holds when `subject` violates this constraint.
    fn condition(&self, subject: &str) -> String {
        match self {
            Self::NotEmpty => format!("{subject}.isEmpty"),
            Self::MinLength(min) => format!("{subject}.length < {min}"),
            Self::MaxLength(max) => format!("{subject}.length > {max}"),
            Self::AtLeast(min) => format!("{subject} < {min}"),
            Self::AtMost(max) => format!("{subject} > {max}"),
            Self::IsTrue => format!("!{subject}"),
            Self::Matches(pattern) => format!("!RegExp({pattern}).hasMatch({subject})"),
        }
    }

    /// What to say when the author did not say it themselves.
    fn message(&self) -> String {
        match self {
            Self::NotEmpty => "must not be empty".to_owned(),
            Self::MinLength(min) => format!("must be at least {min} characters"),
            Self::MaxLength(max) => format!("must be at most {max} characters"),
            Self::AtLeast(min) => format!("must be at least {min}"),
            Self::AtMost(max) => format!("must be at most {max}"),
            Self::IsTrue => "must be accepted".to_owned(),
            Self::Matches(_) => "has the wrong format".to_owned(),
        }
    }
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, _file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "validate")?;
    render::render(TEMPLATE, &build(decl)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl) -> Result<ValidateCtx> {
    // A violation names the field the way the field travels, so a server that
    // rejects the same form can be reconciled with the client that built it.
    let policy = decl
        .annotation("model")
        .and_then(|model| model.arg("fieldRename"))
        .map(casing::unquote);

    let mut rules = Vec::new();
    for field in macros::typed_fields(decl)? {
        let key = model::json_key(&field, policy.as_deref());
        for check in field
            .raw
            .annotations
            .iter()
            .filter(|a| a.name.starts_with(CHECK))
        {
            for constraint in constraints(check)? {
                rules.push(rule(&field, &key, check, &constraint));
            }
        }
    }

    Ok(ValidateCtx {
        className: decl.name.clone(),
        rules,
    })
}

/// One `@dmx('check.…')` becomes one rule per bound it sets: `length(min:, max:)` is two
/// separate things a person can get wrong, and they read as two.
fn constraints(check: &RawAnnotation) -> Result<Vec<Constraint>> {
    let bounds = |low: fn(String) -> Constraint, high: fn(String) -> Constraint| {
        check
            .arg("min")
            .map(|min| low(min.to_owned()))
            .into_iter()
            .chain(check.arg("max").map(|max| high(max.to_owned())))
            .collect::<Vec<_>>()
    };

    let kind = check.name.trim_start_matches(CHECK);
    let found = match kind {
        "notEmpty" => vec![Constraint::NotEmpty],
        "isTrue" => vec![Constraint::IsTrue],
        "length" => bounds(Constraint::MinLength, Constraint::MaxLength),
        "range" => bounds(Constraint::AtLeast, Constraint::AtMost),
        "maxLength" => check
            .arg("limit")
            .map(|limit| vec![Constraint::MaxLength(limit.trim().to_owned())])
            .unwrap_or_default(),
        "matches" | "pattern" => check
            .arg("expression")
            .map(|pattern| vec![Constraint::Matches(pattern.trim().to_owned())])
            .unwrap_or_default(),
        _ => bail!(
            "DMX2006: `@dmx('check.{kind}')` is not a constraint; the vocabulary is \
             notEmpty, length, maxLength, range, matches, pattern, isTrue"
        ),
    };
    if found.is_empty() {
        bail!("DMX2007: `@dmx('check.{kind}')` sets no bound, so it can never be violated");
    }
    Ok(found)
}

/// The author's message, else the constraint's own.
fn message(check: &RawAnnotation, constraint: &Constraint) -> String {
    check
        .arg("message")
        .map_or_else(|| constraint.message(), casing::unquote)
}

/// A nullable field is only checked when it is present: absent is not invalid.
fn rule(field: &Field<'_>, key: &str, check: &RawAnnotation, constraint: &Constraint) -> RuleCtx {
    let name = field.name();
    let condition = if field.ty.nullable {
        format!(
            "{name} case final {} value when {}",
            field.ty.non_null().source,
            constraint.condition("value")
        )
    } else {
        constraint.condition(name)
    };
    RuleCtx {
        message: casing::dart_string(&message(check, constraint)),
        field: key.to_owned(),
        condition,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, refusal, rendered};

    /// Every constructor in the vocabulary compiles to the test it names.
    #[test]
    fn each_constraint_compiles_to_its_condition() {
        emits(
            &rendered(
                expand,
                "@dmx('validate') class F { \
                 @dmx('check.notEmpty') final String a; \
                 @dmx('check.length', {'min': 4, 'max': 10}) final String b; \
                 @dmx('check.range', {'min': 1, 'max': 99}) final int c; \
                 @dmx('check.isTrue', {'message': 'the terms must be accepted'}) final bool d; }",
            ),
            &[
                "if (a.isEmpty)",
                "if (b.length < 4)",
                "if (b.length > 10)",
                "if (c < 1)",
                "if (c > 99)",
                "if (!d)",
                "const Violation('d', 'the terms must be accepted'),",
            ],
        );
    }

    /// One `@dmx('check.…')` with two bounds reads as two things a person can fix.
    #[test]
    fn a_bounded_check_produces_one_rule_per_bound() {
        emits(
            &rendered(
                expand,
                "@dmx('validate') class F { @dmx('check.length', {'min': 4, 'max': 10}) final String b; }",
            ),
            &[
                "'must be at least 4 characters'",
                "'must be at most 10 characters'",
            ],
        );
    }

    /// [catalogue.macros]: absent is not invalid.
    #[test]
    fn a_nullable_field_is_only_checked_when_present() {
        emits(
            &rendered(
                expand,
                "@dmx('validate') class F { @dmx('check.maxLength', {'limit': 200}) final String? note; }",
            ),
            &["if (note case final String value when value.length > 200)"],
        );
    }

    /// The pattern reaches the output verbatim, raw-string prefix included.
    #[test]
    fn a_pattern_keeps_the_literal_the_author_wrote() {
        emits(
            &rendered(
                expand,
                "@dmx('validate') class F { @dmx('check.matches', {'expression': r'^[A-Z]{2}$', 'message': 'must be a country code'}) \
                 final String country; }",
            ),
            &[
                "if (!RegExp(r'^[A-Z]{2}$').hasMatch(country))",
                "'must be a country code'",
            ],
        );
    }

    /// A violation names the field the way the field travels.
    #[test]
    fn violations_are_keyed_by_the_json_name() {
        emits(
            &rendered(
                expand,
                "@dmx('model', {'fieldRename': 'snake'}) @dmx('validate') class F { \
                 @dmx('check.isTrue') final bool acceptsTerms; }",
            ),
            &["const Violation('accepts_terms', 'must be accepted'),"],
        );
    }

    /// [diagnostics]: an unknown constructor is refused in the author's terms.
    #[test]
    fn an_unknown_constraint_is_refused() {
        let err = refusal(
            expand,
            "@dmx('validate') class F { @dmx('check.mystery') final String a; }",
        );
        assert!(err.contains("DMX2006"), "{err}");
    }

    /// A bound-less `length` can never fire, which is never what was meant.
    #[test]
    fn a_check_that_sets_no_bound_is_refused() {
        let err = refusal(
            expand,
            "@dmx('validate') class F { @dmx('check.length') final String a; }",
        );
        assert!(err.contains("DMX2007"), "{err}");
    }
}
