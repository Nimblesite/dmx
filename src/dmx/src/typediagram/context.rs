//! The Mustache context one generation group renders against
//! [typediagram.model].
//!
//! Everything a template could otherwise be tempted to compute is finished
//! here: casings, target type text, generic declarations, constructor
//! fragments, separators, and the first/last markers that let a template lay
//! out a list without arithmetic [context.discipline]. A template selects and
//! places prepared values; it never resolves a type and never decides what a
//! language spells something.
//!
//! Two names carry the same value on purpose. `targetType` is what a
//! language-neutral template asks for, and `dartType` is the name
//! [typediagram.model] pins for the Dart target. Templates written against
//! either keep working when a second target lands.

use anyhow::Result;
use serde_json::{Map, Value, json};

use super::ast::{Decl, Field, Signature, TypeRef, Variant};
use super::markdown::{BoundTemplate, Group};
use super::model::{Model, Resolution};
use super::target::Target;
use crate::casing;

/// The context schema version. A change to the shape below bumps it, and the
/// golden fixtures move in the same commit [typediagram.model].
pub const CONTEXT_VERSION: u64 = 1;

/// Everything one bound template renders against.
///
/// # Errors
///
/// Fails when a reference has no text in this target — a container built-in
/// given the wrong number of arguments is the only way that happens, because
/// every other unresolvable name was refused before this point.
pub fn build(
    document: &str,
    group: &Group,
    template: &BoundTemplate,
    model: &Model,
    target: &Target,
) -> Result<Value> {
    let declarations = model
        .visible(target.name)
        .map(|decl| declaration(decl, model, target))
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({
        "modelVersion": CONTEXT_VERSION,
        "target": target.name,
        "source": {
            "path": document,
            "group": group.ordinal,
            "definitionFence": group.definition.ordinal,
            "definitionLine": group.definition.line,
            "templateFence": template.fence.ordinal,
            "templateLine": template.fence.line,
            "output": template.output,
        },
        "declarations": positioned(declarations),
    }))
}

/// Adds one prepared value to a context object.
///
/// `Map::insert` returns whatever it displaced, which is never anything here
/// and which `unused_results` obliges every caller to discard. Written out, the
/// builders below would be `let _ =` noise wrapped around the one thing that
/// matters — the name and the value.
fn put(out: &mut Map<String, Value>, name: &str, value: impl Into<Value>) {
    drop(out.insert(name.to_owned(), value.into()));
}

/// One declaration, with the flags and members its kind carries.
fn declaration(decl: &Decl, model: &Model, target: &Target) -> Result<Map<String, Value>> {
    let mut out = named(decl.name());
    let generics = decl.generics();
    put(&mut out, "kind", kind_name(decl));
    put(
        &mut out,
        "generics",
        positioned(generics.iter().map(|name| named(name)).collect()),
    );
    put(&mut out, "hasGenerics", !generics.is_empty());
    put(&mut out, "genericDeclaration", generic_list(generics));
    // Mutually exclusive kind flags, so a template selects a shape without a
    // per-kind list duplicating the declaration [typediagram.model].
    for kind in ["record", "union", "alias", "function"] {
        put(
            &mut out,
            &format!("is{}", casing::pascal(kind)),
            kind_name(decl) == kind,
        );
    }
    match decl {
        Decl::Record(record) => {
            put(&mut out, "hasFields", !record.fields.is_empty());
            members(&mut out, "fields", &record.fields, model, target)?;
        }
        Decl::Union(union) => {
            put(&mut out, "untagged", union.untagged);
            put(&mut out, "hasVariants", !union.variants.is_empty());
            let owner = Owner {
                name: &union.name,
                generic_declaration: generic_list(generics),
            };
            let variants = union
                .variants
                .iter()
                .map(|variant| self::variant(variant, &owner, model, target))
                .collect::<Result<Vec<_>>>()?;
            put(&mut out, "variants", positioned(variants));
        }
        Decl::Alias(alias) => {
            let typed = type_ref(&alias.target, model, target)?;
            typed.place_into(&mut out);
            put(&mut out, "target", typed.value);
        }
        Decl::Function(function) => {
            let overloaded = function.signatures.len() > 1;
            let signatures = function
                .signatures
                .iter()
                .map(|signature| self::signature(signature, overloaded, model, target))
                .collect::<Result<Vec<_>>>()?;
            put(&mut out, "hasOverloads", overloaded);
            put(&mut out, "signatures", positioned(signatures));
        }
    }
    Ok(out)
}

/// The union a variant belongs to, which its own `name` would otherwise hide.
///
/// A Mustache section pushes the variant onto the context stack, so `{{name}}`
/// inside `{{#variants}}` is the *variant's* name and the union's is
/// unreachable. Generating `final class Circle extends Shape` needs both, and
/// asking a template to carry one down by hand is exactly the logic
/// [context.discipline] keeps out of templates.
struct Owner<'a> {
    /// The union's declared name.
    name: &'a str,
    /// Its `<A, B>`, or the empty string.
    generic_declaration: String,
}

/// One variant of a union, with its payload shape already decided.
fn variant(
    variant: &Variant,
    owner: &Owner<'_>,
    model: &Model,
    target: &Target,
) -> Result<Map<String, Value>> {
    let mut out = named(&variant.name);
    put(&mut out, "owner", owner.name);
    put(
        &mut out,
        "ownerGenericDeclaration",
        owner.generic_declaration.clone(),
    );
    put(&mut out, "hasFields", !variant.fields.is_empty());
    put(&mut out, "isBare", variant.fields.is_empty());
    put(&mut out, "isTuple", variant.is_tuple());
    put(&mut out, "hasDiscriminant", variant.discriminant.is_some());
    put(
        &mut out,
        "discriminant",
        variant.discriminant.clone().unwrap_or_default(),
    );
    members(&mut out, "fields", &variant.fields, model, target)?;
    Ok(out)
}

/// One overload signature.
///
/// `overloaded` is repeated here from the declaration on purpose. A Mustache
/// section entered on a name the *declaration* carries pushes that value with
/// the declaration beneath it, so `{{#hasOverloads}}{{index}}{{/hasOverloads}}`
/// inside `{{#signatures}}` reads the declaration's ordinal, not the
/// signature's. A flag the signature carries itself keeps the signature under
/// the section, which is the difference between `Read0`/`Read1` and two
/// typedefs called `Read0` [typediagram.model].
fn signature(
    signature: &Signature,
    overloaded: bool,
    model: &Model,
    target: &Target,
) -> Result<Map<String, Value>> {
    let mut out = Map::new();
    let params = fields(&signature.params, model, target)?;
    let returns = type_ref(&signature.returns, model, target)?;
    put(&mut out, "isOverload", overloaded);
    put(&mut out, "isAsync", signature.is_async);
    put(&mut out, "hasParams", !signature.params.is_empty());
    put(&mut out, "parameterList", parameter_list(&params));
    put(&mut out, "returnType", returns.text);
    put(&mut out, "returns", returns.value);
    put(&mut out, "params", positioned(params));
    Ok(out)
}

/// Adds a member list under `name`, together with the constructor fragment it
/// adds up to — the two things a record and a variant both need, in one place.
fn members(
    out: &mut Map<String, Value>,
    name: &str,
    source: &[Field],
    model: &Model,
    target: &Target,
) -> Result<()> {
    let members = fields(source, model, target)?;
    put(
        out,
        "constructorParameters",
        constructor_parameters(&members),
    );
    put(out, name, positioned(members));
    Ok(())
}

/// The name generated code uses for one member [context.discipline].
///
/// typeDiagram spells a tuple variant's positional members `_0`, `_1`, … and
/// the model keeps that spelling, because upstream does and the parity corpus
/// holds it there. Generated code cannot keep it: a leading underscore makes
/// the member private in Dart, which is illegal as a named constructor
/// parameter and dead as a field. Positional members are therefore `value1`,
/// `value2`, … — a proper name [context.discipline], one-based the way every
/// language spells the first element of a tuple. Every other member keeps the
/// name its author wrote.
fn member_name(raw: &str) -> String {
    match raw.strip_prefix('_').map(str::parse::<usize>) {
        Some(Ok(position)) => format!("value{}", position.saturating_add(1)),
        Some(Err(_)) | None => raw.to_owned(),
    }
}

/// A field list — a record's, a variant's payload, or a signature's parameters.
fn fields(fields: &[Field], model: &Model, target: &Target) -> Result<Vec<Map<String, Value>>> {
    fields
        .iter()
        .map(|field| {
            let name = member_name(&field.name);
            let mut out = named(&name);
            let typed = type_ref(&field.ty, model, target)?;
            typed.place_into(&mut out);
            put(&mut out, "typeDiagram", field.ty.canonical());
            put(&mut out, "isOptional", typed.optional);
            put(&mut out, "isRequired", !typed.optional);
            put(&mut out, "parameter", parameter(&name, typed.optional));
            put(&mut out, "type", typed.value);
            Ok(out)
        })
        .collect()
}

/// One type reference, resolved for the target.
///
/// The prepared text and the optional flag come back beside the context object
/// rather than being read back out of it: a member that needs them should not
/// have to index into JSON to find what this function already computed.
struct Typed {
    /// The target's text for the reference.
    text: String,
    /// Whether the reference is an `Option`.
    optional: bool,
    /// The reference as a template sees it.
    value: Map<String, Value>,
}

impl Typed {
    /// Adds this type's text to whatever names it, under both the neutral name
    /// and the Dart one [typediagram.model].
    fn place_into(&self, out: &mut Map<String, Value>) {
        put(out, "targetType", self.text.clone());
        put(out, "dartType", self.text.clone());
    }
}

/// One type reference, resolved and rendered in the target's terms.
fn type_ref(reference: &TypeRef, model: &Model, target: &Target) -> Result<Typed> {
    let text = (target.type_text)(reference, model)?;
    let optional = reference.name == "Option";
    let resolution = model.resolution(reference);
    let arguments = reference
        .args
        .iter()
        .map(|arg| type_ref(arg, model, target).map(|typed| typed.value))
        .collect::<Result<Vec<_>>>()?;
    let mut out = Map::new();
    put(&mut out, "name", reference.name.clone());
    put(&mut out, "typeDiagram", reference.canonical());
    put(&mut out, "targetType", text.clone());
    put(&mut out, "dartType", text.clone());
    put(
        &mut out,
        "isPrimitive",
        matches!(resolution, Resolution::Primitive),
    );
    put(
        &mut out,
        "isDeclared",
        matches!(resolution, Resolution::Declared(_)),
    );
    put(
        &mut out,
        "isTypeParam",
        matches!(resolution, Resolution::TypeParam),
    );
    put(&mut out, "isOptional", optional);
    put(&mut out, "isList", reference.name == "List");
    put(&mut out, "isMap", reference.name == "Map");
    put(&mut out, "isAny", reference.name == "Any");
    put(&mut out, "hasArguments", !arguments.is_empty());
    put(&mut out, "arguments", positioned(arguments));
    Ok(Typed {
        text,
        optional,
        value: out,
    })
}

/// The `kind` a declaration reports.
fn kind_name(decl: &Decl) -> &'static str {
    match decl {
        Decl::Record(_) => "record",
        Decl::Union(_) => "union",
        Decl::Alias(_) => "alias",
        Decl::Function(_) => "function",
    }
}

/// A name in every casing a template might place it in
/// [context.helpers].
fn named(name: &str) -> Map<String, Value> {
    let mut out = Map::new();
    put(&mut out, "name", name);
    put(&mut out, "camelName", casing::camel(name));
    put(&mut out, "pascalName", casing::pascal(name));
    put(&mut out, "snakeName", casing::snake(name));
    put(
        &mut out,
        "screamingSnakeName",
        casing::screaming_snake(name),
    );
    put(&mut out, "label", casing::label(name));
    out
}

/// `<A, B>`, or the empty string when there are no parameters.
fn generic_list(generics: &[String]) -> String {
    if generics.is_empty() {
        return String::new();
    }
    format!("<{}>", generics.join(", "))
}

/// The named-parameter list a constructor takes, braces included, or the empty
/// string when there is nothing to take.
fn constructor_parameters(fields: &[Map<String, Value>]) -> String {
    let parts: Vec<&str> = fields
        .iter()
        .filter_map(|field| field.get("parameter").and_then(Value::as_str))
        .collect();
    if parts.is_empty() {
        return String::new();
    }
    format!("{{{}}}", parts.join(", "))
}

/// One constructor parameter. An optional member has a default of `null`
/// already, so requiring it would only make callers write it.
fn parameter(name: &str, optional: bool) -> String {
    if optional {
        return format!("this.{name}");
    }
    format!("required this.{name}")
}

/// The positional parameter list a free function takes.
fn parameter_list(params: &[Map<String, Value>]) -> String {
    params
        .iter()
        .filter_map(|param| {
            Some(format!(
                "{} {}",
                param.get("targetType")?.as_str()?,
                param.get("name")?.as_str()?
            ))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Stamps `first`, `last`, and `comma` onto every member of a list, so a
/// template lays out separators without counting [context.discipline].
fn positioned(items: Vec<Map<String, Value>>) -> Vec<Value> {
    let last = items.len().saturating_sub(1);
    items
        .into_iter()
        .enumerate()
        .map(|(index, mut item)| {
            let final_item = index == last;
            put(&mut item, "first", index == 0);
            put(&mut item, "last", final_item);
            put(&mut item, "index", index);
            put(&mut item, "comma", if final_item { "" } else { "," });
            Value::Object(item)
        })
        .collect()
}

// A separate file only because context.rs is at the 500-line ceiling.
#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
