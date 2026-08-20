//! Name resolution and validation for a parsed definition [typediagram.model].
//!
//! Resolution answers one question per type reference — generic parameter,
//! declared type, primitive, or external — in the order upstream answers it,
//! because a declared name deliberately shadows a built-in so that
//! `alias Uuid = String` keeps meaning what it meant before the semantic
//! scalars existed [typediagram.delivery.baseline].
//!
//! Validation is deliberately in two halves. Structural validation is what
//! *any* consumer of the model needs — duplicate declarations, generic arity —
//! and matches upstream exactly. Generation validation is stricter: a name
//! that resolves to nothing renders as a diagram but cannot become code, so
//! [`Model::validate_for_target`] refuses it before a template ever runs.

use std::collections::{BTreeMap, BTreeSet};

use super::ast::{Decl, Diagram, Field, Span, TypeRef};
use super::diagnostic::{Diagnostic, Diagnostics};

/// The scalar names typeDiagram always understands, semantic scalars included.
pub const PRIMITIVES: &[&str] = &[
    "Bool", "Int", "Float", "String", "Bytes", "Unit", "DateTime", "Uuid", "Decimal",
];

/// The generic names every converter understands without a declaration.
pub const BUILTIN_GENERICS: &[&str] = &["List", "Map", "Option", "Any"];

/// What one type reference turned out to name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    /// A generic parameter of the declaration it was written in; the
    /// reference's own name is the parameter.
    TypeParam,
    /// Another declaration in the same definition.
    Declared(String),
    /// One of [`PRIMITIVES`].
    Primitive,
    /// A name this definition does not declare.
    External,
}

/// A definition with every reference resolved [typediagram.model].
///
/// The declarations are the parsed ones, unchanged and in source order: the
/// resolution table is keyed by position rather than folded into the tree, so
/// nothing here can reorder or drop what the author wrote.
#[derive(Clone, Debug)]
pub struct Model {
    /// The declarations, in source order.
    decls: Vec<Decl>,
    /// What each reference resolves to, keyed by its position in the source.
    resolutions: BTreeMap<(usize, usize), Resolution>,
}

impl Model {
    /// Resolves and structurally validates `diagram`.
    ///
    /// # Errors
    ///
    /// Fails on a duplicate declaration or a generic-arity mismatch, reporting
    /// every one it found.
    pub fn resolve(diagram: Diagram) -> Result<Self, Diagnostics> {
        let mut found = Diagnostics::default();
        let mut arity: BTreeMap<&str, usize> = BTreeMap::new();
        for decl in &diagram.decls {
            if arity.insert(decl.name(), decl.generics().len()).is_some() {
                found.0.push(diagnostic(
                    format!("duplicate declaration '{}'", decl.name()),
                    decl.span(),
                ));
            }
        }

        let mut resolutions = BTreeMap::new();
        for decl in &diagram.decls {
            let generics: BTreeSet<&str> = decl.generics().iter().map(String::as_str).collect();
            for reference in references(decl) {
                resolve_reference(reference, &generics, &arity, &mut resolutions, &mut found);
            }
        }

        if found.is_empty() {
            return Ok(Self {
                decls: diagram.decls,
                resolutions,
            });
        }
        Err(found)
    }

    /// The declarations, exactly once each and in source order.
    #[must_use]
    pub fn decls(&self) -> &[Decl] {
        &self.decls
    }

    /// The declarations `target` generates from — everything, minus what a
    /// `@targets` / `@skipTargets` annotation excludes.
    pub fn visible(&self, target: &str) -> impl Iterator<Item = &Decl> {
        self.decls
            .iter()
            .filter(move |decl| decl.targeting().is_none_or(|t| t.admits(target)))
    }

    /// What `reference` names. An unrecorded position cannot occur for a
    /// reference taken from this model's own declarations, and reading it as
    /// external is the answer that changes nothing.
    #[must_use]
    pub fn resolution(&self, reference: &TypeRef) -> &Resolution {
        self.resolutions
            .get(&(reference.span.line, reference.span.col))
            .unwrap_or(&Resolution::External)
    }

    /// Refuses references `target` cannot turn into code [typediagram.model].
    ///
    /// A diagram renders an unknown name as inline text. Generation cannot:
    /// the name would reach the output verbatim and produce source that does
    /// not compile, which is the worst thing this pipeline can emit.
    ///
    /// # Errors
    ///
    /// Fails naming every unresolvable reference, once each.
    pub fn validate_for_target(&self, target: &str) -> Result<(), Diagnostics> {
        let mut found = Diagnostics::default();
        let mut reported: BTreeSet<&str> = BTreeSet::new();
        for decl in self.visible(target) {
            for reference in references(decl) {
                let unknown = matches!(self.resolution(reference), Resolution::External)
                    && !BUILTIN_GENERICS.contains(&reference.name.as_str())
                    && reported.insert(reference.name.as_str());
                if unknown {
                    found.0.push(diagnostic(
                        format!(
                            "unknown type '{}': not a primitive, a built-in, or a declared type",
                            reference.name
                        ),
                        reference.span,
                    ));
                }
            }
        }
        if found.is_empty() {
            return Ok(());
        }
        Err(found)
    }
}

/// Records what one reference names, and complains about a wrong arity on a
/// declared one exactly where upstream does.
fn resolve_reference(
    reference: &TypeRef,
    generics: &BTreeSet<&str>,
    arity: &BTreeMap<&str, usize>,
    resolutions: &mut BTreeMap<(usize, usize), Resolution>,
    found: &mut Diagnostics,
) {
    let name = reference.name.as_str();
    let resolution = match (generics.contains(name), arity.get(name)) {
        (true, _) => Resolution::TypeParam,
        (false, Some(&expected)) => {
            if reference.args.len() != expected {
                found.0.push(diagnostic(
                    format!(
                        "type '{name}' takes {expected} type argument(s), got {}",
                        reference.args.len()
                    ),
                    reference.span,
                ));
            }
            Resolution::Declared(name.to_owned())
        }
        (false, None) if PRIMITIVES.contains(&name) => Resolution::Primitive,
        (false, None) => Resolution::External,
    };
    let _ = resolutions.insert((reference.span.line, reference.span.col), resolution);
}

/// Every type reference a declaration contains, nested arguments included, in
/// source order.
///
/// One walk serves resolution, generation validation, and the context builder,
/// so no consumer can accidentally look at a different set of references than
/// another one did.
#[must_use]
pub fn references(decl: &Decl) -> Vec<&TypeRef> {
    let mut out = Vec::new();
    match decl {
        Decl::Record(record) => push_fields(&record.fields, &mut out),
        Decl::Union(union) => {
            for variant in &union.variants {
                push_fields(&variant.fields, &mut out);
            }
        }
        Decl::Alias(alias) => push_ref(&alias.target, &mut out),
        Decl::Function(function) => {
            for signature in &function.signatures {
                push_fields(&signature.params, &mut out);
                push_ref(&signature.returns, &mut out);
            }
        }
    }
    out
}

/// Adds every reference in `fields`, in order.
fn push_fields<'a>(fields: &'a [Field], out: &mut Vec<&'a TypeRef>) {
    for field in fields {
        push_ref(&field.ty, out);
    }
}

/// Adds `reference` and everything nested inside it, outermost first.
fn push_ref<'a>(reference: &'a TypeRef, out: &mut Vec<&'a TypeRef>) {
    out.push(reference);
    for arg in &reference.args {
        push_ref(arg, out);
    }
}

/// One diagnostic anchored at `span`.
fn diagnostic(message: String, span: Span) -> Diagnostic {
    Diagnostic::at(message, span.line, span.col, span.length)
}

#[cfg(test)]
mod tests {
    use super::super::ast::Decl;
    use super::super::parser::parse;
    use super::{Model, Resolution, references};

    /// The model `source` resolves to.
    fn resolved(source: &str) -> Model {
        Model::resolve(parse(source).expect("parse")).expect("resolve")
    }

    /// The resolutions of every reference in the first declaration.
    fn first_resolutions(source: &str) -> Vec<Resolution> {
        let model = resolved(source);
        references(&model.decls()[0])
            .into_iter()
            .map(|reference| model.resolution(reference).clone())
            .collect()
    }

    /// [typediagram.model]: generic parameter, then declared, then primitive,
    /// then external — the order that lets a declaration shadow a built-in.
    #[test]
    fn resolution_follows_the_upstream_precedence() {
        assert_eq!(
            first_resolutions("type A<T> { a: T\n b: B\n c: Int\n d: Mystery }\ntype B { x: Int }"),
            [
                Resolution::TypeParam,
                Resolution::Declared("B".to_owned()),
                Resolution::Primitive,
                Resolution::External,
            ]
        );
    }

    /// [typediagram.model]: a declared name wins over a built-in scalar, so a
    /// pre-scalar diagram keeps its meaning.
    #[test]
    fn a_declaration_shadows_a_primitive() {
        assert_eq!(
            first_resolutions("type A { id: Uuid }\nalias Uuid = String"),
            [Resolution::Declared("Uuid".to_owned())]
        );
        assert_eq!(
            first_resolutions("type A { id: Uuid }"),
            [Resolution::Primitive]
        );
    }

    /// [typediagram.model]: nested arguments resolve individually.
    #[test]
    fn nested_arguments_each_resolve() {
        assert_eq!(
            first_resolutions("type A { m: Map<String, List<B>> }\ntype B { x: Int }"),
            [
                Resolution::External,
                Resolution::Primitive,
                Resolution::External,
                Resolution::Declared("B".to_owned()),
            ]
        );
    }

    /// [typediagram.model]: duplicates and arity mismatches are refused, all
    /// of them at once.
    #[test]
    fn structural_faults_are_reported_together() {
        let error = Model::resolve(
            parse("type A { }\ntype A { }\ntype C<T> { x: T }\ntype D { c: C }").expect("parse"),
        )
        .expect_err("two structural faults");
        let text = error.to_string();
        assert!(text.contains("duplicate declaration 'A'"), "{text}");
        assert!(
            text.contains("type 'C' takes 1 type argument(s), got 0"),
            "{text}"
        );
    }

    /// [typediagram.model]: an unknown name fails before rendering, once, with
    /// its position — and the container built-ins are not unknown.
    #[test]
    fn generation_refuses_unresolvable_names() {
        let model = resolved("type A { a: Timestamp\n b: Timestamp\n c: List<Instant>\n d: Any }");
        let error = model
            .validate_for_target("dart")
            .expect_err("two unknown names");
        assert_eq!(error.0.len(), 2, "{error}");
        assert!(error.0[0].message.contains("unknown type 'Timestamp'"));
        assert_eq!((error.0[0].line, error.0[0].col), (1, 13));
        assert!(error.0[1].message.contains("unknown type 'Instant'"));

        resolved("type A { a: List<String>\n b: Map<String, Any>\n c: Option<Int> }")
            .validate_for_target("dart")
            .expect("container built-ins are known");
    }

    /// [typediagram.model]: a declaration another target owns is neither
    /// generated nor validated for this one.
    #[test]
    fn targeting_hides_a_declaration_from_generation() {
        let model = resolved("@skipTargets(dart)\ntype A { a: Timestamp }\ntype B { b: Int }");
        model
            .validate_for_target("dart")
            .expect("the skipped declaration is not this target's problem");
        assert_eq!(
            model.visible("dart").map(Decl::name).collect::<Vec<_>>(),
            ["B"]
        );
        assert_eq!(
            model.decls().len(),
            2,
            "nothing is discarded from the model"
        );
        assert!(model.visible("go").any(|decl| decl.name() == "A"));
    }
}
