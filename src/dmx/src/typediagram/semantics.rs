//! Value semantics and the JSON codec one class gets [typediagram.canonical].
//!
//! A record declared in a diagram is an immutable value, and a value that
//! cannot be compared is not one. This module finishes every `==`, `hashCode`,
//! `toString`, `copyWith`, decode and encode expression a generated class
//! needs, in Rust, exactly as `@dmx('model')` finishes them for a class
//! somebody wrote by hand — the same functions, called with a different
//! [`Runtime`], so the two can never say different things about the same type
//! [authoring.intelligence].
//!
//! Two things differ, and both follow from the file being written whole rather
//! than spliced into one somebody else owns.
//!
//! The runtime import is this generator's to write, so it is written prefixed:
//! a diagram is free to declare a type called `Result`, a local declaration
//! hides an imported name, and `dmx.Result` cannot be hidden by anything.
//!
//! The JSON members go on a `<Name>Json` extension rather than into the class,
//! so the class stays what the diagram said it was — a constructor, its fields,
//! and value semantics — and serialization is something added to it.
//!
//! Not every declaration can have a codec. A type parameter, a generic
//! declaration, an untagged union, `Unit`, and a map keyed by anything but a
//! string all refuse one [typediagram.canonical], and a declaration that
//! contains one of them keeps its class and its value semantics and simply has
//! no JSON extension.

use anyhow::{Result, bail};
use serde_json::{Map, Value};

use super::ast::Field;
use super::model::Model;
use super::prepared::put;
use super::target::Target;
use crate::casing;
use crate::macros::{self, model as datamodel};
use crate::types::{DartType, JSON_EXTENSION, Runtime};

/// The import whole-file generation writes to reach the runtime
/// [typediagram.canonical].
pub const RUNTIME_IMPORT: &str = "import 'package:dmx/dmx.dart' as dmx;";

/// How generated code in a file dmx wrote whole reaches the runtime.
const RUNTIME: Runtime = Runtime::PREFIXED;

/// The Dart type a `toJson` returns. `Object?` rather than `dynamic`, because
/// generated code never needs the one thing `dynamic` adds.
const JSON_MAP: &str = "Map<String, Object?>";

/// `Object.hash` takes at most 20 positional components, and `runtimeType` is
/// the first of them.
const HASH_ARITY: usize = 19;

/// One class the canonical model template writes out
/// [typediagram.canonical].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Class<'a> {
    /// Its Dart name — a record's own, or a variant's owner-qualified one.
    pub name: String,
    /// Its Dart type, type parameters included.
    pub ty: String,
    /// Whether it is generic, which is what stops it having a codec.
    pub generic: bool,
    /// Its members, in declaration order.
    pub fields: &'a [Field],
}

/// Places everything the canonical template needs onto one class and its
/// members [typediagram.canonical].
///
/// `members` are the context objects [`super::context`] already built for
/// `class.fields`, in the same order, and they are finished in place.
///
/// # Errors
///
/// Fails when a member reached here without the name or the target type the
/// context builder puts on every one of them, which is a bug in this crate.
pub fn place(
    out: &mut Map<String, Value>,
    members: &mut [Map<String, Value>],
    class: &Class<'_>,
    model: &Model,
    target: &Target,
) -> Result<()> {
    let names = members.iter().map(text).collect::<Result<Vec<_>>>()?;
    let other = macros::fresh_name(&["other", "that", "operand"], &names).to_owned();

    let mut codecs = Vec::new();
    let mut values = 0usize;
    let mut opaque = false;
    for (member, field) in members.iter_mut().zip(class.fields) {
        let name = text(member)?;
        let declared = DartType::parse(&declared_type(member)?)?;
        // Dart's `void` is not a value: it cannot be compared, interpolated, or
        // passed on, so a member of that type takes part in nothing.
        let is_value = declared.name != "void";
        put(member, "isValue", is_value);
        put(member, "isLastValue", false);
        if is_value {
            values = values.saturating_add(1);
            put(
                member,
                "equalsExpr",
                datamodel::comparison(&declared, &other, &name, true, RUNTIME),
            );
            put(
                member,
                "hashExpr",
                datamodel::hash_component(&declared, &name, RUNTIME),
            );
            put(
                member,
                "copyParam",
                datamodel::copy_param(&declared, &name, RUNTIME),
            );
            put(
                member,
                "copyArg",
                datamodel::copy_arg(&declared, &name, RUNTIME),
            );
            put(member, "toStringExpr", format!("{name}: ${name}"));
        } else {
            opaque = true;
        }
        codecs.push(codec(&name, field, model, target));
    }

    let has_json = !class.generic && codecs.iter().all(Result::is_ok);
    let mut complex = 0;
    if has_json {
        complex = place_codecs(members, codecs)?;
    } else {
        put(out, "jsonRefusals", refusals(class, codecs));
    }

    // `toString` separates the members that carry a value, which is not the
    // same list as the members [context.discipline] already marked `last`.
    if let Some(member) = members.iter_mut().rev().find(|m| flag(m, "isValue")) {
        put(member, "isLastValue", true);
    }

    // A `void` member cannot be handed back to the constructor, so a class
    // holding one has nothing to copy into.
    let can_copy = !class.fields.is_empty() && !opaque;
    let wide = values > HASH_ARITY;
    put(out, "otherParam", other);
    put(
        out,
        "hashCombiner",
        if wide {
            "Object.hashAll"
        } else {
            "Object.hash"
        },
    );
    put(out, "useHashAll", wide);
    put(out, "hasValues", values > 0);
    put(out, "canCopy", can_copy);
    put(out, "hasComplex", complex > 0);
    put(
        out,
        "hasPattern",
        members.iter().any(|member| flag(member, "inPattern")),
    );
    codec_names(out, has_json, &class.name, &class.ty, members.is_empty());
    // Exactly the expressions the canonical template will actually render: an
    // import this file does not use is an analyzer error, not a stray line.
    let copies = can_copy && touches(members, &["copyParam", "copyArg"]);
    put(
        out,
        "usesRuntime",
        has_json || copies || touches(members, &["equalsExpr", "hashExpr"]),
    );
    Ok(())
}

/// The names a codec is written in terms of, for a class or a union.
///
/// Prepared rather than composed by the template, because every one of them
/// names something in the runtime and the prefix that reaches it is this
/// module's business, not a template author's [context.discipline].
pub fn codec_names(
    out: &mut Map<String, Value>,
    has_json: bool,
    name: &str,
    ty: &str,
    empty: bool,
) {
    put(out, "className", name.to_owned());
    put(out, "classType", ty.to_owned());
    put(out, "hasJson", has_json);
    put(out, "jsonExtension", format!("{name}{JSON_EXTENSION}"));
    put(out, "jsonMap", JSON_MAP);
    put(
        out,
        "decodeResult",
        format!(
            "{}<{ty}, {}>",
            RUNTIME.name("Result"),
            RUNTIME.name("DecodeError")
        ),
    );
    put(out, "decodeOk", RUNTIME.name("Ok"));
    put(
        out,
        "decodeFailure",
        format!(
            "{}({}(path, '{name}', json))",
            RUNTIME.name("Err"),
            RUNTIME.name("DecodeError")
        ),
    );
    put(out, "decodeErr", format!("{}(e)", RUNTIME.name("Err")));
    // A class with no members has nothing to read out of the map it matched,
    // and a binding nothing uses is an analyzer error.
    put(
        out,
        "jsonShape",
        if empty {
            format!("{JSON_MAP}()")
        } else {
            format!("final {JSON_MAP} json")
        },
    );
}

/// Places one member's codec on it, and reports how many of them decode
/// through a `Result`.
fn place_codecs(
    members: &mut [Map<String, Value>],
    codecs: Vec<Result<datamodel::Codec>>,
) -> Result<usize> {
    for (member, built) in members.iter_mut().zip(codecs) {
        let built = built?;
        put(member, "bind", built.bind);
        put(member, "ctorExpr", built.ctor_expr);
        put(member, "jsonKey", built.json_key);
        put(member, "patternType", built.pattern_type);
        put(member, "inPattern", built.in_pattern);
        put(member, "isComplex", built.is_complex);
        put(member, "resultExpr", built.result_expr);
        put(member, "encodeExpr", built.encode_expr);
    }
    // The record pattern that selects each failing member, binding the error it
    // carries: `(_, dmx.Err(error: final e), _)`.
    let complex = members.iter().filter(|m| flag(m, "isComplex")).count();
    let mut patterns = macros::error_patterns(complex, RUNTIME).into_iter();
    for member in members.iter_mut().filter(|m| flag(m, "isComplex")) {
        put(member, "errPattern", patterns.next().unwrap_or_default());
    }
    Ok(complex)
}

/// One member's codec, in the terms whole-file generation writes it in.
fn codec(name: &str, field: &Field, model: &Model, target: &Target) -> Result<datamodel::Codec> {
    let text = (target.codec_text)(&field.ty, model)?;
    let ty = DartType::parse(&text)?;
    datamodel::codec(name, &ty, casing::dart_string(name), RUNTIME)
}

/// Why a class has no JSON extension, one reason per member that refused one.
///
/// A missing codec is a deliberate outcome rather than a failure — the class
/// and its value semantics are generated either way — but it is never silent:
/// `dmx explain` prints `hasJson` beside these, so a reader who expected a
/// codec is told which member decided otherwise.
fn refusals(class: &Class<'_>, codecs: Vec<Result<datamodel::Codec>>) -> Vec<Value> {
    let mut out: Vec<Value> = codecs
        .into_iter()
        .filter_map(Result::err)
        .map(|refusal| Value::String(refusal.to_string()))
        .collect();
    if class.generic {
        out.push(Value::String(format!(
            "DMX8009 [typediagram.canonical]: `{}` is generic, and a codec for a \
             type parameter is not known until it is applied",
            class.ty
        )));
    }
    out
}

/// Whether any of `keys` on any member reaches the runtime, which is what
/// decides whether the file imports it at all [typediagram.canonical].
fn touches(members: &[Map<String, Value>], keys: &[&str]) -> bool {
    members.iter().any(|member| {
        keys.iter()
            .filter_map(|key| member.get(*key))
            .filter_map(Value::as_str)
            .any(|value| value.contains(RUNTIME.prefix))
    })
}

/// One member's `name`.
fn text(member: &Map<String, Value>) -> Result<String> {
    match member.get("name").and_then(Value::as_str) {
        Some(name) => Ok(name.to_owned()),
        None => bail!("DMX2000: internal error — a member reached the canonical builder unnamed"),
    }
}

/// One member's target type text.
fn declared_type(member: &Map<String, Value>) -> Result<String> {
    match member.get("dartType").and_then(Value::as_str) {
        Some(text) => Ok(text.to_owned()),
        None => bail!("DMX2000: internal error — a member reached the canonical builder untyped"),
    }
}

/// One boolean a member carries, absent reading as false.
fn flag(member: &Map<String, Value>, name: &str) -> bool {
    member.get(name).and_then(Value::as_bool).unwrap_or(false)
}
