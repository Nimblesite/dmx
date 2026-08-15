//! `@dmx('model')` [model] — the immutable data class.
//!
//! JSON codec, `==`/`hashCode`, `toString`, `copyWith`. Rich context, dumb
//! template [authoring.intelligence]: every decode/encode/equals/hash/copy
//! expression is finished here, in Rust, and reaches the template as a string.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field, union};
use crate::render;
use crate::types::{self, DartType};

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/model.mustache");

#[derive(Content)]
/// One field, as the template names its parts.
pub struct FieldCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// Local name the pattern binds this field to. Equal to `name` unless that
    /// would be illegal as a pattern variable.
    pub bind: String,
    /// What the constructor receives: the binding, or a pure transform of it.
    pub ctorExpr: String,
    /// The JSON key this field is read from and written to.
    pub jsonKey: String,
    /// Dart type the map pattern binds this field at — its *JSON* shape, so a
    /// `DateTime` binds as `String` and a `List<T>` as `List<dynamic>`.
    pub patternType: String,
    /// Required fields are destructured by the map pattern; nullable fields are
    /// read with `dmxKey` so that an absent key decodes as null.
    pub inPattern: bool,
    /// Contributes a `Result` to the record that sequences the decode.
    pub isComplex: bool,
    /// The `Result` this entry contributes.
    pub resultExpr: String,
    /// Record pattern selecting this field's failure, e.g. `(_, Err(error: final e))`.
    pub errPattern: String,
    /// This entry on the way out.
    pub encodeExpr: String,
    /// This field, compared against `otherParam`.
    pub equalsExpr: String,
    /// This field, as a hash component consistent with `equalsExpr`.
    pub hashExpr: String,
    /// This field's `copyWith` parameter.
    pub copyParam: String,
    /// What `copyWith` passes to the constructor for it.
    pub copyArg: String,
    /// This field inside the interpolated `toString`.
    pub toStringExpr: String,
    /// Marks the final entry, so a template lays out separators without arithmetic.
    pub isLast: bool,
}

#[derive(Content)]
/// The whole context `model.mustache` renders against.
// Each of these is one mustache section, and a section is a boolean. The
// state enum the lint asks for is not something a template can switch on.
#[allow(clippy::struct_excessive_bools)]
pub struct ModelCtx {
    /// The class the members are generated into.
    pub className: String,
    /// Every field the macro generates for, in source order.
    pub fields: Vec<FieldCtx>,
    /// The parameter the comparison is against.
    pub otherParam: String,
    /// `Object.hash` or `Object.hashAll`, depending on the field count.
    pub hashCombiner: String,
    /// `Object.hash` takes at most 20 components, so a wide class hashes a list.
    pub useHashAll: bool,
    /// `Object.hash` takes at least two components, and a class with no fields
    /// would offer it one. Its identity is then its type and nothing else.
    pub hasFields: bool,
    /// At least one required field, so the decode opens with a map pattern.
    /// Otherwise the shape check narrows the value to a map on its own.
    pub hasPattern: bool,
    /// At least one field decodes to a `Result`, so a record sequences them.
    pub hasComplex: bool,
    /// The declaration also encodes, so a fixture can be handed to a decoder.
    pub wantsJson: bool,
    /// Generate `copyWith` [model.copywith].
    pub wantsCopyWith: bool,
    /// Generate `toString`.
    pub wantsToString: bool,
    /// Generate `==` and `hashCode` [model.equality].
    pub wantsEquality: bool,
    /// This class is a variant of a sibling `@dmx('union')`, so its `toJson` overrides
    /// the base's declaration and writes the discriminator itself.
    pub isVariant: bool,
    /// The union's discriminator key, as a Dart string literal.
    pub unionKey: String,
    /// This class's tag, as a Dart string literal.
    pub unionTag: String,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "model")?;
    render::render(TEMPLATE, &build(decl, file)?)
}

/// Everything the template names, computed here [authoring.intelligence].
pub fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<ModelCtx> {
    let model = decl
        .annotation("model")
        .context("DMX2000: internal error — reached the model builder without @dmx('model')")?;
    let policy = model.arg("fieldRename").map(casing::unquote);

    let names: Vec<String> = decl.fields.iter().map(|f| f.name.clone()).collect();
    // `fromJson` is static, so no instance field can shadow `json` or `path`.
    let other_param = macros::fresh_name(&["other", "that", "operand"], &names).to_owned();

    let mut fields = Vec::new();
    for field in macros::typed_fields(decl)? {
        fields.push(field_context(&field, &other_param, policy.as_deref())?);
    }

    // The record pattern that selects each failing field, binding the error
    // payload it carries: `(_, Err(error: final e), _)`.
    let arity = fields.iter().filter(|f| f.isComplex).count();
    let mut patterns = macros::error_patterns(arity).into_iter();
    for field in fields.iter_mut().filter(|f| f.isComplex) {
        field.errPattern = patterns.next().unwrap_or_default();
    }
    macros::mark_last(&mut fields, |f| f.isLast = true);
    // Object.hash takes at most 20 positional components; runtimeType is one.
    let use_hash_all = fields.len() > 19;

    // A variant tags itself on the way out, so the base's `fromJson` can read
    // back what the base's `toJson` produced [catalogue.macros].
    let union = macros::base_with(decl, file, "union").and_then(|base| base.annotation("union"));

    Ok(ModelCtx {
        className: decl.name.clone(),
        otherParam: other_param,
        hashCombiner: if use_hash_all {
            "Object.hashAll".into()
        } else {
            "Object.hash".into()
        },
        useHashAll: use_hash_all,
        hasFields: !fields.is_empty(),
        hasPattern: fields.iter().any(|f| f.inPattern),
        hasComplex: arity > 0,
        wantsJson: model.flag("json").unwrap_or(true),
        wantsCopyWith: model.flag("copyWith").unwrap_or(true),
        wantsToString: model.flag("toString").unwrap_or(true),
        wantsEquality: model.flag("equality").unwrap_or(true),
        isVariant: union.is_some(),
        unionKey: union.map_or_else(String::new, |u| {
            casing::dart_string(&union::discriminator(u))
        }),
        unionTag: union.map_or_else(String::new, |u| {
            casing::dart_string(&union::tag(u, &decl.name))
        }),
        fields,
    })
}

/// The JSON key a field is read from and written to: `@dmx('key', {'name': })`, else the
/// `fieldRename` policy, else the field's own name.
pub fn json_key(field: &Field<'_>, policy: Option<&str>) -> String {
    match field.raw.annotation("key").and_then(|k| k.arg("name")) {
        Some(explicit) => explicit.to_owned(),
        None => casing::dart_string(&match policy {
            Some(policy) => casing::rename(policy, field.name()),
            None => field.name().to_owned(),
        }),
    }
}

/// Everything the template names about one field.
fn field_context(field: &Field<'_>, other: &str, policy: Option<&str>) -> Result<FieldCtx> {
    let (name, ty) = (field.name(), &field.ty);
    let bind = macros::binding_name(name);
    let key = json_key(field, policy);
    // Interpolated, not baked: a nested failure reports the path it was reached
    // by — `Order.lines[2].product` — rather than the type it happened in. The
    // path names the *wire* key, because that is what the payload in front of
    // whoever reads the error actually says.
    let path = format!("'$path.{}'", key.trim_matches('\''));

    // A required field whose decode cannot fail needs no result: the map
    // pattern has already proved its shape.
    let direct = !ty.nullable && types::pure_transform(ty, &bind).is_some();
    let result_expr = if direct {
        String::new()
    } else if ty.nullable {
        let inner = ty.non_null();
        format!(
            "dmxNullable<{}>(dmxKey(json, {key}), {path}, {})",
            inner.source,
            types::decoder(&inner, 12)?
        )
    } else {
        types::decode_bound(ty, &bind, &path, 12)?
    };

    Ok(FieldCtx {
        patternType: if ty.nullable {
            String::new()
        } else {
            types::json_shape(ty)
        },
        inPattern: !ty.nullable,
        isComplex: !direct,
        resultExpr: result_expr,
        errPattern: String::new(), // arity is only known once all fields are in
        encodeExpr: types::encode(ty, name, 0),
        equalsExpr: comparison(ty, other, name, true),
        hashExpr: hash_component(ty, name),
        copyParam: copy_param(ty, name),
        copyArg: copy_arg(ty, name),
        toStringExpr: format!("{name}: ${name}"),
        // Direct fields carry their transform into the constructor call;
        // everything else arrives already decoded, bound by the record pattern.
        ctorExpr: if direct {
            types::pure_transform(ty, &bind).unwrap_or_else(|| bind.clone())
        } else {
            bind.clone()
        },
        jsonKey: key,
        name: name.to_owned(),
        bind,
        isLast: false,
    })
}

/// [model.equality]: collections compare by content, everything else by `==`.
///
/// `equal` picks the sense. `@dmx('diff')` asks for the negation rather than forming
/// its own opinion, so "changed" and "unequal" can never drift apart.
pub fn comparison(ty: &DartType, other: &str, name: &str, equal: bool) -> String {
    match (ty.is_collection(), equal) {
        (true, true) => format!("dmxDeepEquals({other}.{name}, {name})"),
        (true, false) => format!("!dmxDeepEquals({other}.{name}, {name})"),
        (false, true) => format!("{other}.{name} == {name}"),
        (false, false) => format!("{other}.{name} != {name}"),
    }
}

/// [model.equality]: a hash consistent with [`comparison`].
pub fn hash_component(ty: &DartType, name: &str) -> String {
    if ty.is_collection() {
        format!("dmxDeepHash({name})")
    } else {
        name.to_owned()
    }
}

/// [model.copywith]: a nullable field takes a patch, so omitting it and
/// clearing it are different calls; everything else takes `T?` and `??`.
fn copy_param(ty: &DartType, name: &str) -> String {
    if ty.nullable {
        format!("DmxPatch<{}> {name} = const DmxKeep()", ty.source)
    } else {
        format!("{}? {name}", ty.source)
    }
}

/// [model.copywith]: what `copyWith` passes on for one field.
fn copy_arg(ty: &DartType, name: &str) -> String {
    if ty.nullable {
        format!(
            "{name}: switch ({name}) {{ \
             DmxKeep() => this.{name}, DmxTo(value: final value) => value }}"
        )
    } else {
        format!("{name}: {name} ?? this.{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, omits, rendered};

    /// [model.json-codec]: one shape check, then the fields.
    #[test]
    fn decodes_from_an_untyped_value() {
        emits(
            &rendered(expand, "@dmx('model') class A { final String id; }"),
            &[
                "static Result<A, DecodeError> fromJson(Object? json, [String path = 'A'])",
                "'id': final String id,",
                "_ => Err(DecodeError(path, 'A', json)),",
            ],
        );
    }

    /// With no required field there is no map pattern, so the arm narrows.
    #[test]
    fn all_nullable_narrows_instead_of_destructuring() {
        emits(
            &rendered(expand, "@dmx('model') class A { final String? id; }"),
            &["final Map<String, dynamic> json =>", "dmxKey(json, 'id')"],
        );
    }

    /// [surface.annotations]: `fieldRename` renames every key at once.
    #[test]
    fn field_rename_policy_applies_to_every_key() {
        emits(
            &rendered(
                expand,
                "@dmx('model', {'fieldRename': 'snake'}) class A { final String createdAt; \
                 @dmx('key', {'name': 'ID'}) final String id; }",
            ),
            // An explicit @dmx('key') still wins over the policy.
            &["'created_at'", "'ID'"],
        );
    }

    /// [hygiene]: generated code never throws, casts, or asserts non-null.
    #[test]
    fn output_obeys_the_house_rules() {
        omits(
            &rendered(
                expand,
                "@dmx('model') class A { final String id; final DateTime at; \
                 final List<int>? scores; final Address home; }",
            ),
            &["throw", " as ", "!", "_$"],
        );
    }
}
