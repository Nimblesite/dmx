//! Generation targets [typediagram.model].
//!
//! A target is the *only* place a language leaks into this feature: it maps a
//! resolved typeDiagram reference onto that language's type text, says what
//! extension its files carry, and validates a finished file. Everything else
//! in `typediagram` — the lexer, the parser, the model, the binder, the
//! context builder, the emitter — is language-neutral.
//!
//! One target ships today, because one language does. The registry is a table
//! rather than a trait object for the same reason the macro catalogue is
//! [catalogue]: adding a language is adding a row, not a plugin lifecycle.

use anyhow::{Result, bail};

use super::ast::{Decl, TypeRef};
use super::model::{Model, Resolution};

/// Everything the pipeline needs to know about one output language.
pub struct Target {
    /// The name `dmx.target` selects it by.
    pub name: &'static str,
    /// The extension every output it generates must carry, without the dot.
    pub extension: &'static str,
    /// The directory this language keeps its sources in, relative to the
    /// project root — where a standalone template's output lands when the
    /// template names no path of its own [typediagram.standalone].
    pub source_root: &'static str,
    /// The file that marks a project root in this language, which is what an
    /// output path is resolved against [typediagram.output].
    pub project_marker: &'static str,
    /// This language's text for one resolved reference.
    pub type_text: fn(&TypeRef, &Model) -> Result<String>,
    /// This language's text for a reference the JSON codec table has to work
    /// in, or a refusal when the reference has no codec [typediagram.canonical].
    pub codec_text: fn(&TypeRef, &Model) -> Result<String>,
    /// Refuses a finished file that does not parse, or that generated code is
    /// not allowed to contain [hygiene].
    pub validate: fn(&str, &str) -> Result<()>,
    /// The canonical model template for this language: what a definition
    /// renders through when no template beside it says otherwise
    /// [typediagram.canonical].
    pub canonical: &'static str,
}

/// A target is mostly function pointers, which carry nothing a diagnostic
/// could act on. Its name and extension are the whole of what identifies it.
impl std::fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Target")
            .field("name", &self.name)
            .field("extension", &self.extension)
            .finish_non_exhaustive()
    }
}

/// Every target this build can generate [typediagram.model].
const TARGETS: &[Target] = &[Target {
    name: "dart",
    extension: "dart",
    source_root: "lib",
    project_marker: "pubspec.yaml",
    type_text: dart_type,
    codec_text: dart_codec_type,
    validate: validate_dart,
    canonical: include_str!("../../templates/diagram_model.mustache"),
}];

/// Every file that marks a project root, for any target this build carries
/// [typediagram.output].
///
/// One document resolves its outputs against one root, so the search is over
/// all of them rather than over the target a particular template named: a
/// document that generated Dart into one package and something else into
/// another would have two identities and no single ownership marker.
pub fn project_markers() -> impl Iterator<Item = &'static str> {
    TARGETS.iter().map(|target| target.project_marker)
}

/// Every extension a generated output can carry, for any target this build
/// carries [typediagram.output].
///
/// This is what a pass sweeps for when it collects the outputs a removed
/// template used to produce: a stale file is found by its ownership marker,
/// and this is the set of files worth opening to look for one.
pub fn extensions() -> impl Iterator<Item = &'static str> {
    TARGETS.iter().map(|target| target.extension)
}

/// The target `name` selects.
///
/// # Errors
///
/// Fails when no target carries that name, listing the ones that do.
pub fn find(name: &str) -> Result<&'static Target> {
    TARGETS
        .iter()
        .find(|target| target.name == name)
        .map_or_else(
            || {
                bail!(
                    "DMX8007 [typediagram.model]: `{name}` is not a generation target dmx knows; \
                 available: {}",
                    TARGETS
                        .iter()
                        .map(|target| target.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            Ok,
        )
}

/// The Dart text for one resolved reference [typediagram.model].
///
/// This is the whole of dmx's typeDiagram-to-Dart mapping, and it matches the
/// published table: `Option<T>` is Dart's own `T?`, the semantic scalars land
/// on Dart's native types where Dart has one, and a declared name keeps its
/// arguments.
///
/// # Errors
///
/// Fails when a container built-in was given the wrong number of arguments,
/// which the model deliberately does not check — a diagram renders `List` with
/// no argument, and Dart cannot.
fn dart_type(reference: &TypeRef, model: &Model) -> Result<String> {
    let args = reference
        .args
        .iter()
        .map(|arg| dart_type(arg, model))
        .collect::<Result<Vec<_>>>()?;
    match model.resolution(reference) {
        // A generic parameter is its own name, and a declared type is its name
        // plus whatever it was applied to.
        Resolution::TypeParam => Ok(reference.name.clone()),
        Resolution::Declared(name) => Ok(applied(name, &args)),
        Resolution::Primitive => Ok(primitive(&reference.name).to_owned()),
        Resolution::External => container(reference, &args),
    }
}

/// The Dart text the JSON codec table works in [typediagram.canonical].
///
/// The same mapping as [`dart_type`] with two differences, both of them about
/// what a codec can actually be built for.
///
/// An alias is followed to what it stands for. `alias Email = String` makes
/// `Email` a Dart typedef, and a typedef is not a name the codec table can look
/// up — it has to see the `String` underneath. A record or a union keeps its
/// own name, because that name is exactly what its codec is filed under.
///
/// Everything else is refused rather than guessed. A type parameter has no
/// codec because the diagram never says what it will be; a generic declaration
/// has none for the same reason; an untagged union has none because nothing in
/// the payload says which case it is; and `Unit` is Dart's `void`, which is not
/// a value at all. A refusal here costs the declaration its JSON extension and
/// nothing else — the class, its value semantics, and `copyWith` are unaffected.
///
/// # Errors
///
/// Fails when the reference has no JSON codec, naming what it was.
fn dart_codec_type(reference: &TypeRef, model: &Model) -> Result<String> {
    let args = reference
        .args
        .iter()
        .map(|arg| dart_codec_type(arg, model))
        .collect::<Result<Vec<_>>>()?;
    let no_codec = |what: &str| {
        bail!(
            "DMX8009 [typediagram.canonical]: `{}` has no JSON codec: {what}",
            reference.canonical()
        )
    };
    match model.resolution(reference) {
        Resolution::TypeParam => no_codec("a type parameter is not known until it is applied"),
        Resolution::Declared(name) => match model.declaration(name) {
            Some(Decl::Alias(alias)) if alias.generics.is_empty() => {
                dart_codec_type(&alias.target, model)
            }
            Some(Decl::Record(record)) if record.generics.is_empty() => Ok(applied(name, &args)),
            Some(Decl::Union(union)) if union.generics.is_empty() && !union.untagged => {
                Ok(applied(name, &args))
            }
            Some(Decl::Union(union)) if union.untagged => {
                let _ = union;
                no_codec("an untagged union carries nothing that says which case a payload is")
            }
            Some(Decl::Function(_)) => no_codec("a function is not data"),
            // A generic alias, record, or union, or a name this model does not
            // declare at all — which resolution already proved it does.
            _ => no_codec("a generic declaration has no codec until it is applied"),
        },
        Resolution::Primitive => match primitive(&reference.name) {
            "void" => no_codec("`Unit` is Dart's `void`, which is not a value"),
            text => Ok(text.to_owned()),
        },
        Resolution::External => container(reference, &args),
    }
}

/// Dart's name for one typeDiagram scalar.
///
/// `Uuid` and `Decimal` have no native Dart type, so they carry the string
/// their wire form already is; `Unit` is Dart's `void`.
fn primitive(name: &str) -> &'static str {
    match name {
        "Bool" => "bool",
        "Int" => "int",
        "Float" => "double",
        "Bytes" => "List<int>",
        "Unit" => "void",
        "DateTime" => "DateTime",
        // `String`, `Uuid`, and `Decimal` are all Dart strings. The list is
        // exhaustive over PRIMITIVES, and a name that somehow reached here
        // without being one keeps its spelling rather than becoming a lie.
        _ => "String",
    }
}

/// Dart's form for one of the container built-ins.
fn container(reference: &TypeRef, args: &[String]) -> Result<String> {
    let arity = |wanted: usize| {
        if args.len() == wanted {
            return Ok(());
        }
        bail!(
            "DMX8004 [typediagram.model]: `{}` takes {wanted} type argument(s), got {} \
             (line {}, column {})",
            reference.name,
            args.len(),
            reference.span.line,
            reference.span.col
        )
    };
    match reference.name.as_str() {
        "Option" => {
            arity(1)?;
            Ok(nullable(args.first().map_or("Object", String::as_str)))
        }
        "List" => {
            arity(1)?;
            Ok(applied("List", args))
        }
        "Map" => {
            arity(2)?;
            Ok(applied("Map", args))
        }
        // `Any` is Dart's `Object`, matching the published mapping table.
        "Any" => {
            arity(0)?;
            Ok("Object".to_owned())
        }
        // Generation validation has already refused every other external name
        // [typediagram.model], so nothing else can reach here.
        other => bail!("DMX8004 [typediagram.model]: `{other}` has no Dart type"),
    }
}

/// `Name<a, b>`, or just `Name` when there are no arguments.
fn applied(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        return name.to_owned();
    }
    format!("{name}<{}>", args.join(", "))
}

/// The nullable form of a Dart type.
///
/// Dart has no `T??` and no `void?`, so an already-optional type and `void`
/// are their own nullable forms — which is exactly what `Option<Option<T>>`
/// means anyway.
fn nullable(text: &str) -> String {
    if text.ends_with('?') || text == "void" {
        return text.to_owned();
    }
    format!("{text}?")
}

/// Refuses generated Dart that does not parse, or that breaks [hygiene].
fn validate_dart(source: &str, origin: &str) -> Result<()> {
    crate::frontend::Frontend::new()?.validate(source, origin)?;
    crate::hygiene::check(source, origin)
}

#[cfg(test)]
mod tests {
    use super::super::model::Model;
    use super::super::parser::parse;
    use super::{find, nullable};

    /// The Dart types of the first declaration's fields, in order.
    fn dart_fields(source: &str) -> Vec<String> {
        let model = Model::resolve(parse(source).expect("parse")).expect("resolve");
        let target = find("dart").expect("dart target");
        let super::super::ast::Decl::Record(record) = &model.decls()[0] else {
            panic!("expected a record");
        };
        record
            .fields
            .iter()
            .map(|field| (target.type_text)(&field.ty, &model).expect("dart type"))
            .collect()
    }

    /// [typediagram.model]: the published mapping table, scalar by scalar.
    #[test]
    fn scalars_map_to_dart() {
        assert_eq!(
            dart_fields(
                "type A { a: Bool, b: Int, c: Float, d: String, e: Bytes, f: DateTime, g: Uuid, h: Decimal }"
            ),
            [
                "bool",
                "int",
                "double",
                "String",
                "List<int>",
                "DateTime",
                "String",
                "String"
            ]
        );
    }

    /// [typediagram.model]: containers, nesting, declared names, and generic
    /// parameters all come out as Dart writes them.
    #[test]
    fn containers_and_declared_names_map_to_dart() {
        assert_eq!(
            dart_fields(
                "type A<T> { a: List<String>, b: Map<String, List<B>>, c: Option<Int>, d: Any, e: T, f: B, g: C<Int> }\ntype B { x: Int }\ntype C<T> { y: T }"
            ),
            [
                "List<String>",
                "Map<String, List<B>>",
                "int?",
                "Object",
                "T",
                "B",
                "C<int>",
            ]
        );
    }

    /// [typediagram.model]: Dart has no `T??`, so a doubled option is one
    /// option — and `Option<Unit>` is still `void`.
    #[test]
    fn nullability_never_doubles() {
        assert_eq!(
            dart_fields("type A { a: Option<Option<String>>, b: Option<Unit>, c: Option<Any> }"),
            ["String?", "void", "Object?"]
        );
        assert_eq!(nullable("int"), "int?");
    }

    /// [typediagram.model]: a container given the wrong arity renders as a
    /// diagram and cannot become Dart, so generation refuses it.
    #[test]
    fn a_container_with_the_wrong_arity_is_refused() {
        for source in [
            "type A { a: List }",
            "type A { a: Map<String> }",
            "type A { a: Option<Int, Int> }",
            "type A { a: Any<Int> }",
        ] {
            let model = Model::resolve(parse(source).expect("parse")).expect("resolve");
            let target = find("dart").expect("dart target");
            let super::super::ast::Decl::Record(record) = &model.decls()[0] else {
                panic!("expected a record");
            };
            let error = (target.type_text)(&record.fields[0].ty, &model).expect_err(source);
            assert!(
                format!("{error:#}").contains("DMX8004"),
                "{source}: {error:#}"
            );
        }
    }

    /// [typediagram.model]: an unknown target is named, with the ones that
    /// exist.
    #[test]
    fn an_unknown_target_is_refused_with_the_alternatives() {
        let error = format!("{:#}", find("kotlin").expect_err("no kotlin target"));
        assert!(error.contains("DMX8007"), "{error}");
        assert!(error.contains("available: dart"), "{error}");
        let dart = find("dart").expect("dart target");
        assert_eq!(dart.extension, "dart");
        assert_eq!(dart.project_marker, "pubspec.yaml");
        assert!(super::project_markers().any(|marker| marker == "pubspec.yaml"));
        assert!(super::extensions().any(|extension| extension == "dart"));
    }

    /// [hygiene]: the Dart target refuses a file that does not parse and one
    /// that breaks the generated-code rules.
    #[test]
    fn the_dart_target_validates_what_it_emits() {
        let target = find("dart").expect("dart target");
        (target.validate)("final class A {}\n", "test").expect("valid Dart");
        assert!((target.validate)("final class A {", "test").is_err());
        assert!((target.validate)("int f(Object o) => throw 'no';\n", "test").is_err());
    }
}
