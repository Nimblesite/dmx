//! The typeDiagram syntax tree [typediagram.model].
//!
//! One immutable value per production in the published grammar, in the order
//! the author wrote them. Nothing here resolves a name — a `TypeRef` is a
//! spelling until [`super::model`] says what it refers to — so the parser can
//! be read against the grammar without knowing anything about resolution.

/// Where a node came from, inside the definition it was parsed from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    /// One-based line within the definition.
    pub line: usize,
    /// One-based column within the line.
    pub col: usize,
    /// How many characters the node spans on its opening line.
    pub length: usize,
}

/// A whole definition: every declaration, in source order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagram {
    /// The declarations, in the order they were written.
    pub decls: Vec<Decl>,
}

/// The four things a typeDiagram definition can declare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decl {
    /// `type Name<..> { field: Type … }`.
    Record(Record),
    /// `union Name<..> { Variant … }`, optionally `untagged`.
    Union(Union),
    /// `alias Name<..> = Type`.
    Alias(Alias),
    /// `function name<..>(..) -> Type`, or an overload block.
    Function(Function),
}

impl Decl {
    /// The declared name, whichever form this is.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Record(d) => &d.name,
            Self::Union(d) => &d.name,
            Self::Alias(d) => &d.name,
            Self::Function(d) => &d.name,
        }
    }

    /// The generic parameters it introduces, in declaration order.
    #[must_use]
    pub fn generics(&self) -> &[String] {
        match self {
            Self::Record(d) => &d.generics,
            Self::Union(d) => &d.generics,
            Self::Alias(d) => &d.generics,
            Self::Function(d) => &d.generics,
        }
    }

    /// Where the declaration begins.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Record(d) => d.span,
            Self::Union(d) => d.span,
            Self::Alias(d) => d.span,
            Self::Function(d) => d.span,
        }
    }

    /// The `@targets` / `@skipTargets` filter written above it, if any.
    #[must_use]
    pub fn targeting(&self) -> Option<&Targeting> {
        match self {
            Self::Record(d) => d.targeting.as_ref(),
            Self::Union(d) => d.targeting.as_ref(),
            Self::Alias(d) => d.targeting.as_ref(),
            Self::Function(d) => d.targeting.as_ref(),
        }
    }
}

/// Which generation targets a declaration is meant for.
///
/// Each list is absent rather than empty when the author did not write the
/// annotation, because upstream's model JSON distinguishes the two and the
/// differential corpus compares against it [typediagram.delivery.baseline].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Targeting {
    /// `@targets(a, b)` — an allow list.
    pub targets: Option<Vec<String>>,
    /// `@skipTargets(a, b)` — a deny list.
    pub skip_targets: Option<Vec<String>>,
}

impl Targeting {
    /// Whether a declaration carrying this filter is visible to `target`.
    ///
    /// An allow list that was written but left empty filters nothing, which is
    /// upstream's rule and the only reading that keeps `@targets()` harmless.
    #[must_use]
    pub fn admits(&self, target: &str) -> bool {
        let allowed = self
            .targets
            .as_ref()
            .is_none_or(|names| names.is_empty() || names.iter().any(|name| name == target));
        allowed
            && !self
                .skip_targets
                .as_ref()
                .is_some_and(|names| names.iter().any(|name| name == target))
    }
}

/// `type Name<..> { … }`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// The declared name.
    pub name: String,
    /// Its generic parameters, in declaration order.
    pub generics: Vec<String>,
    /// Its fields, in source order.
    pub fields: Vec<Field>,
    /// The target filter written above it.
    pub targeting: Option<Targeting>,
    /// Where the declaration begins.
    pub span: Span,
}

/// `union Name<..> { … }`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Union {
    /// The declared name.
    pub name: String,
    /// Its generic parameters, in declaration order.
    pub generics: Vec<String>,
    /// Whether it was declared `untagged`.
    pub untagged: bool,
    /// Its variants, in source order.
    pub variants: Vec<Variant>,
    /// The target filter written above it.
    pub targeting: Option<Targeting>,
    /// Where the declaration begins.
    pub span: Span,
}

/// `alias Name<..> = Target`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alias {
    /// The declared name.
    pub name: String,
    /// Its generic parameters, in declaration order.
    pub generics: Vec<String>,
    /// What the name stands for.
    pub target: TypeRef,
    /// The target filter written above it.
    pub targeting: Option<Targeting>,
    /// Where the declaration begins.
    pub span: Span,
}

/// `function name<..> …` — one or more overload signatures under one name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    /// The declared name.
    pub name: String,
    /// Its generic parameters, in declaration order.
    pub generics: Vec<String>,
    /// Its signatures, in source order; a bare form declares exactly one.
    pub signatures: Vec<Signature>,
    /// The target filter written above it.
    pub targeting: Option<Targeting>,
    /// Where the declaration begins.
    pub span: Span,
}

/// One `(params) -> Return` signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    /// The parameters, in source order.
    pub params: Vec<Field>,
    /// What the signature returns.
    pub returns: TypeRef,
    /// Whether the signature itself was written `async`.
    pub is_async: bool,
    /// Where the signature begins.
    pub span: Span,
}

/// A named, typed member: a record field, a variant field, or a parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    /// The member's name. A tuple variant's positional members are `_0`, `_1`,
    /// … exactly as upstream names them.
    pub name: String,
    /// Its declared type.
    pub ty: TypeRef,
    /// Where the member begins.
    pub span: Span,
}

/// One arm of a union.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variant {
    /// The variant's name.
    pub name: String,
    /// The pinned wire value, as written, when the author gave one.
    pub discriminant: Option<String>,
    /// Its payload, empty for a bare variant.
    pub fields: Vec<Field>,
    /// Where the variant begins.
    pub span: Span,
}

impl Variant {
    /// Whether the payload was written in tuple form — which upstream records
    /// by naming the members `_0`, `_1`, … and nothing else.
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        !self.fields.is_empty()
            && self
                .fields
                .iter()
                .enumerate()
                .all(|(index, field)| field.name == format!("_{index}"))
    }
}

/// A type as written: a name and its type arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeRef {
    /// The name as the author spelled it.
    pub name: String,
    /// Its type arguments, in source order.
    pub args: Vec<TypeRef>,
    /// Where the reference begins.
    pub span: Span,
}

impl TypeRef {
    /// The canonical typeDiagram spelling, arguments included
    /// [typediagram.model].
    #[must_use]
    pub fn canonical(&self) -> String {
        if self.args.is_empty() {
            return self.name.clone();
        }
        format!(
            "{}<{}>",
            self.name,
            self.args
                .iter()
                .map(Self::canonical)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, Span, Targeting, TypeRef, Variant};

    /// A span used where the test is not about positions.
    const SPAN: Span = Span {
        line: 1,
        col: 1,
        length: 1,
    };

    /// A reference with no arguments, for the tests below.
    fn plain(name: &str) -> TypeRef {
        TypeRef {
            name: name.to_owned(),
            args: Vec::new(),
            span: SPAN,
        }
    }

    /// [typediagram.model]: the canonical spelling round-trips nesting.
    #[test]
    fn canonical_spelling_keeps_nested_arguments() {
        let nested = TypeRef {
            name: "Map".to_owned(),
            args: vec![
                plain("String"),
                TypeRef {
                    name: "List".to_owned(),
                    args: vec![plain("Product")],
                    span: SPAN,
                },
            ],
            span: SPAN,
        };
        assert_eq!(nested.canonical(), "Map<String, List<Product>>");
        assert_eq!(plain("Int").canonical(), "Int");
    }

    /// [typediagram.model]: tuple form is exactly the `_0`, `_1`, … naming.
    #[test]
    fn tuple_variants_are_recognised_by_their_member_names() {
        let field = |name: &str| Field {
            name: name.to_owned(),
            ty: plain("Int"),
            span: SPAN,
        };
        let variant = |fields: Vec<Field>| Variant {
            name: "V".to_owned(),
            discriminant: None,
            fields,
            span: SPAN,
        };
        assert!(variant(vec![field("_0"), field("_1")]).is_tuple());
        assert!(!variant(vec![field("_0"), field("width")]).is_tuple());
        assert!(!variant(Vec::new()).is_tuple());
    }

    /// [typediagram.model]: an allow list excludes everything not on it; a
    /// deny list excludes only what is.
    #[test]
    fn targeting_filters_by_allow_then_deny() {
        let allow = Targeting {
            targets: Some(vec!["dart".to_owned()]),
            skip_targets: None,
        };
        assert!(allow.admits("dart"));
        assert!(!allow.admits("rust"));

        let deny = Targeting {
            targets: None,
            skip_targets: Some(vec!["dart".to_owned()]),
        };
        assert!(!deny.admits("dart"));
        assert!(deny.admits("rust"));
        assert!(Targeting::default().admits("anything"));

        let empty = Targeting {
            targets: Some(Vec::new()),
            skip_targets: None,
        };
        assert!(
            empty.admits("anything"),
            "an empty allow list filters nothing"
        );
    }
}
