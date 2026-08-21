//! What generated code calls each union case [typediagram.canonical.names].
//!
//! typeDiagram's own emitters name a case's class by the case's own name —
//! `final class Circle extends Shape` — and dmx names it the same way, because
//! a diagram is a shared source of truth and two tools generating from it must
//! agree on what the types are called.
//!
//! A case name is only unique inside its union, though, and a Dart library has
//! one namespace. So a case whose name is already taken — by another
//! declaration in the same definition, by a case of another union, or by a Dart
//! name generated code writes itself — takes its union's name as a prefix and
//! becomes `<Union><Case>`. That is the [PROPER NAMES] rule: the name a case
//! was given, qualified only on a real collision.
//!
//! When both are taken the definition is refused (`DMX8010`) rather than
//! guessed at: two classes with one name is Dart that does not compile, and a
//! numbered suffix would be a name nobody chose.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};

use super::ast::Decl;
use super::model::Model;

/// The Dart names generated code writes itself, which a declaration therefore
/// cannot take without changing what those words mean in the file.
///
/// This is the target's mapping table read back: every name
/// [`super::target::dart_type`] can produce, plus the ones the canonical
/// template spells out — `Object` for a JSON value and for `Object.hash`,
/// `String` for `toString`, `Function` for a signature typedef.
const DART_NAMES: &[&str] = &[
    "bool", "double", "int", "void", "DateTime", "Function", "List", "Map", "Object", "String",
];

/// The class name every union case in one model generates under
/// [typediagram.canonical.names].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Names {
    /// `(union, case)` to the class name that case generates.
    chosen: BTreeMap<(String, String), String>,
}

impl Names {
    /// Decides a class name for every case of every union `target` generates
    /// from.
    ///
    /// # Errors
    ///
    /// Fails (`DMX8010`) when a case can be called neither by its own name nor
    /// by its qualified one, naming both and what holds them.
    pub fn of(model: &Model, target: &str) -> Result<Self> {
        let mut taken: BTreeSet<String> = DART_NAMES.iter().map(|&name| name.to_owned()).collect();
        for decl in model.visible(target) {
            let _ = taken.insert(decl.name().to_owned());
        }
        let cases = cases(model, target);
        let mut shared: BTreeMap<&str, usize> = BTreeMap::new();
        for &(_, variant) in &cases {
            let seen = shared.entry(variant).or_default();
            *seen = seen.saturating_add(1);
        }

        let mut chosen = BTreeMap::new();
        for &(union, variant) in &cases {
            let qualified = format!("{union}{variant}");
            let bare_is_free =
                !taken.contains(variant) && shared.get(variant).is_none_or(|count| *count == 1);
            let name = if bare_is_free {
                variant.to_owned()
            } else if taken.contains(&qualified) {
                bail!(
                    "DMX8010 [typediagram.canonical.names]: the `{variant}` case of `{union}` \
                     has no name left to generate under — `{variant}` is already taken, and so \
                     is `{qualified}`"
                )
            } else {
                qualified
            };
            let _ = taken.insert(name.clone());
            let _ = chosen.insert((union.to_owned(), variant.to_owned()), name);
        }
        Ok(Self { chosen })
    }

    /// What the `variant` case of `union` is called.
    ///
    /// A case this model never declared falls back to its qualified name, which
    /// is the answer that collides with nothing.
    #[must_use]
    pub fn case(&self, union: &str, variant: &str) -> String {
        self.chosen
            .get(&(union.to_owned(), variant.to_owned()))
            .cloned()
            .unwrap_or_else(|| format!("{union}{variant}"))
    }
}

/// Every `(union, case)` pair `target` generates, in declaration order.
fn cases<'a>(model: &'a Model, target: &'a str) -> Vec<(&'a str, &'a str)> {
    model
        .visible(target)
        .flat_map(|decl| match decl {
            Decl::Union(union) => union
                .variants
                .iter()
                .map(|variant| (union.name.as_str(), variant.name.as_str()))
                .collect(),
            Decl::Record(_) | Decl::Alias(_) | Decl::Function(_) => Vec::new(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::model::Model;
    use super::super::parser::parse;
    use super::Names;

    /// The names one definition's cases generate under, for the Dart target.
    fn names(source: &str) -> Names {
        let model = Model::resolve(parse(source).expect("parse")).expect("resolve");
        Names::of(&model, "dart").expect("names")
    }

    /// [typediagram.canonical.names]: a case nothing else claims keeps its own
    /// name, exactly as typeDiagram's emitters write it.
    #[test]
    fn a_case_keeps_the_name_the_diagram_gave_it() {
        let chosen = names("union Shape { Circle { r: Float } Square { s: Float } }");
        assert_eq!(chosen.case("Shape", "Circle"), "Circle");
        assert_eq!(chosen.case("Shape", "Square"), "Square");
    }

    /// [typediagram.canonical.names]: two unions with a case of the same name
    /// both qualify, so neither is renamed by the accident of coming second.
    #[test]
    fn a_name_two_unions_share_qualifies_on_both_sides() {
        let chosen = names("union Result { Ok { v: Int } }\nunion Outcome { Ok { v: Int } }");
        assert_eq!(chosen.case("Result", "Ok"), "ResultOk");
        assert_eq!(chosen.case("Outcome", "Ok"), "OutcomeOk");
    }

    /// [typediagram.canonical.names]: a case that would shadow a record, or a
    /// Dart name the file writes itself, qualifies; its siblings do not.
    #[test]
    fn a_taken_name_qualifies_and_leaves_its_siblings_alone() {
        let chosen = names(
            "type Circle { r: Float }\nunion Shape { Circle { r: Float } Square { s: Float } String { s: String } }",
        );
        assert_eq!(chosen.case("Shape", "Circle"), "ShapeCircle");
        assert_eq!(chosen.case("Shape", "String"), "ShapeString");
        assert_eq!(chosen.case("Shape", "Square"), "Square");
    }

    /// [typediagram.canonical.names]: a declaration excluded from this target
    /// takes no name with it.
    #[test]
    fn a_declaration_another_target_owns_claims_nothing() {
        let chosen =
            names("@targets(rust)\ntype Circle { r: Float }\nunion Shape { Circle { r: Float } }");
        assert_eq!(chosen.case("Shape", "Circle"), "Circle");
    }

    /// [typediagram.canonical.names]: when both names are taken the definition
    /// is refused rather than generated as Dart that will not compile.
    #[test]
    fn a_case_with_no_name_left_is_refused() {
        let model = Model::resolve(
            parse("type Circle { r: Float }\ntype ShapeCircle { r: Float }\nunion Shape { Circle { r: Float } }")
                .expect("parse"),
        )
        .expect("resolve");
        let error = format!("{:#}", Names::of(&model, "dart").expect_err("no name left"));
        assert!(error.contains("DMX8010"), "{error}");
        assert!(error.contains("`ShapeCircle`"), "{error}");
    }

    /// A case nobody declared answers with the name that collides with
    /// nothing.
    #[test]
    fn an_undeclared_case_falls_back_to_its_qualified_name() {
        assert_eq!(Names::default().case("Shape", "Circle"), "ShapeCircle");
    }
}
