//! Dart types and the JSON codec table [model.json-codec].
//!
//! Shared by every macro that touches JSON — `@dmx('model')`, `@dmx('union')`, `@dmx('enum')`,
//! `@dmx('table')`, `@dmx('route')`, `@dmx('restClient')` — so a codec improvement lands in all of
//! them at once, and none of them decides anything about types on its own.
//!
//! Decoding never throws. A field is decoded one of two ways:
//!
//! * **directly** — a required field whose JSON representation *is* its Dart
//!   representation (`String`, `int`, …). The map pattern binds it and it goes
//!   straight into the constructor.
//! * **as a result** — everything else. It contributes a `Result<T, DecodeError>`
//!   expression to a record, and one exhaustive record pattern sequences them.
//!
//! ## The uniform decoder [model.json-codec]
//!
//! Every `fromJson` dmx generates — for a model, an enum, or a union — has the
//! same shape:
//!
//! ```dart
//! static Result<T, DecodeError> fromJson(Object? json, [String path = 'T'])
//! ```
//!
//! `Object?` rather than `Map<String, dynamic>`, because `jsonDecode` returns
//! `dynamic` and the alternative is a cast at every entry point — the exact
//! cast the house rules forbid. Shape-checking moves *inside* the decoder,
//! where a wrong shape becomes an `Err` instead of a crash.
//!
//! Two things fall out. A nested type needs no shape check from its parent, so
//! `Address.fromJson` is directly a `DmxDecode<Address>` and the whole codec
//! table stops caring whether `Address` is a model, an enum, or a union — which
//! is what makes the macros compose without a cross-file type resolver
//! [frontend.no-type-inference]. And `path` threads through, so a failure five
//! levels down reports `Order.lines[2].product.price`, not `Money.price`.

use anyhow::{Result, bail};

/// A parsed Dart type: `Map<String, List<int>>?` and friends.
#[derive(Debug, Clone)]
pub struct DartType {
    /// The bare name, without type arguments or `?`.
    pub name: String,
    /// The type arguments, parsed in turn.
    pub args: Vec<DartType>,
    /// Whether the source ended in `?`.
    pub nullable: bool,
    /// Canonical source text, e.g. `List<String>?`.
    pub source: String,
}

impl DartType {
    /// Parses one complete Dart type, or says why it could not.
    ///
    /// # Errors
    ///
    /// Fails when `src` is not a complete Dart type.
    pub fn parse(src: &str) -> Result<Self> {
        let (ty, rest) = Self::parse_prefix(src.trim())?;
        if !rest.is_empty() {
            bail!("unexpected `{rest}` after type");
        }
        Ok(ty)
    }

    /// Parses one type off the front of `src`, returning what is left.
    fn parse_prefix(src: &str) -> Result<(Self, &str)> {
        let name_len = src
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$' && c != '.')
            .unwrap_or(src.len());
        if name_len == 0 {
            bail!("cannot parse type `{src}`");
        }
        let name = &src[..name_len];
        let mut rest = &src[name_len..];
        let mut args = Vec::new();
        if let Some(stripped) = rest.strip_prefix('<') {
            rest = stripped;
            loop {
                let (arg, r) = Self::parse_prefix(rest.trim_start())?;
                args.push(arg);
                rest = r.trim_start();
                match rest.chars().next() {
                    Some(',') => rest = &rest[1..],
                    Some('>') => {
                        rest = &rest[1..];
                        break;
                    }
                    _ => bail!("cannot parse type arguments in `{src}`"),
                }
            }
        }
        let nullable = rest.starts_with('?');
        if nullable {
            rest = &rest[1..];
        }
        Ok((Self::assembled(name.to_owned(), args, nullable), rest))
    }

    /// Builds a type with canonical source text (`Map<String, int>?`),
    /// independent of the author's whitespace — it feeds generated code.
    fn assembled(name: String, args: Vec<DartType>, nullable: bool) -> Self {
        let mut source = name.clone();
        if !args.is_empty() {
            let inner: Vec<&str> = args.iter().map(|a| a.source.as_str()).collect();
            source = format!("{source}<{}>", inner.join(", "));
        }
        if nullable {
            source.push('?');
        }
        Self {
            name,
            args,
            nullable,
            source,
        }
    }

    /// The same type with its `?` removed.
    #[must_use]
    pub fn non_null(&self) -> Self {
        Self::assembled(self.name.clone(), self.args.clone(), false)
    }

    /// Whether values of this type compare by content [model.equality].
    #[must_use]
    pub fn is_collection(&self) -> bool {
        matches!(self.name.as_str(), "List" | "Set" | "Map" | "Iterable")
    }

    /// Not a built-in: a sibling model, enum, or union, resolved by name
    /// [frontend.no-type-inference].
    #[must_use]
    pub fn is_declared(&self) -> bool {
        self.args.is_empty() && !is_builtin(&self.name)
    }

    /// Types whose JSON encoding is the value itself (`jsonEncode` handles them).
    pub fn is_identity(&self) -> bool {
        match self.name.as_str() {
            "String" | "int" | "double" | "bool" | "num" | "dynamic" | "Object" => true,
            "List" | "Map" => self.args.iter().all(Self::is_identity),
            _ => false,
        }
    }

    /// Whether the decoded Dart value *is* the JSON value, so a pattern binding
    /// it needs no further work. `double` is excluded: JSON `1` arrives as `int`.
    #[must_use]
    pub fn is_identity_shape(&self) -> bool {
        matches!(
            self.name.as_str(),
            "String" | "int" | "bool" | "num" | "dynamic" | "Object"
        )
    }
}

/// Whether the name is one Dart or its core libraries already define.
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "int"
            | "double"
            | "bool"
            | "num"
            | "dynamic"
            | "Object"
            | "DateTime"
            | "Uri"
            | "BigInt"
            | "Duration"
            | "List"
            | "Set"
            | "Map"
            | "Iterable"
    )
}

/// The Dart expression for types that cannot fail once the pattern has bound
/// them — the JSON shape already proves the value is good. `None` means the
/// decode can fail and must go through a `Result`.
#[must_use]
pub fn pure_transform(ty: &DartType, value: &str) -> Option<String> {
    match ty.name.as_str() {
        _ if ty.is_identity_shape() => Some(value.to_owned()),
        "double" => Some(format!("{value}.toDouble()")),
        "Duration" => Some(format!("Duration(microseconds: {value})")),
        _ => None,
    }
}

/// The Dart type a JSON value must have before [`decode_bound`] can use it.
///
/// A declared type binds as `Object?`: its own `fromJson` does the checking,
/// which is what lets one expression decode a model, an enum, or a union.
#[must_use]
pub fn json_shape(ty: &DartType) -> String {
    match ty.name.as_str() {
        "DateTime" | "Uri" | "BigInt" => "String".into(),
        "Duration" => "int".into(),
        "double" => "num".into(),
        "List" | "Set" | "Iterable" => "List<dynamic>".into(),
        "Map" => "Map<String, dynamic>".into(),
        _ if ty.is_declared() => "Object?".into(),
        _ => ty.source.clone(),
    }
}

/// Decodes `value`, which already has the type [`json_shape`] requires, into a
/// `Result<T, DecodeError>` [model.json-codec]. `path` is a Dart *expression* — usually
/// an interpolation like `'$path.email'` — so failures name their location.
///
/// # Errors
///
/// Fails when the type has the wrong number of type arguments, a map key that
/// is not a `String`, or no codec at all.
pub fn decode_bound(ty: &DartType, value: &str, path: &str, indent: usize) -> Result<String> {
    if let Some(expr) = pure_transform(ty, value) {
        return Ok(format!("Ok({expr})"));
    }
    Ok(match ty.name.as_str() {
        // Explicit type arguments: without them the arms' least upper bound
        // widens to `Object` and the enclosing record stops being exhaustive.
        "DateTime" | "Uri" | "BigInt" => format!(
            "switch ({}.tryParse({value})) {{ \
             final {0} parsed => Ok<{0}, DecodeError>(parsed), \
             null => Err<{0}, DecodeError>(DecodeError({path}, '{0}', {value})) }}",
            ty.name
        ),
        "List" | "Set" | "Iterable" => {
            let [elem] = ty.args.as_slice() else {
                bail!("DMX2102: `{}` needs exactly one type argument", ty.source);
            };
            let combinator = if ty.name == "Set" {
                "dmxSet"
            } else {
                "dmxList"
            };
            format!(
                "{combinator}<{}>({value}, {path}, {})",
                elem.source,
                decoder(elem, indent)?
            )
        }
        "Map" => {
            let [k, v] = ty.args.as_slice() else {
                bail!("DMX2102: `{}` needs exactly two type arguments", ty.source);
            };
            if k.name != "String" || k.nullable {
                bail!("DMX2101: map key type `{}` is not String", k.source);
            }
            format!(
                "dmxMap<{}>({value}, {path}, {})",
                v.source,
                decoder(v, indent)?
            )
        }
        // Name-level resolution: an unrecognized simple type decodes itself.
        _ if ty.is_declared() => format!("{}.fromJson({value}, {path})", ty.name),
        _ => bail!("DMX2102: cannot build a codec for `{}`", ty.source),
    })
}

/// A `DmxDecode<T>` for the element of a collection: shape-checks an untyped
/// JSON value, then decodes it.
///
/// A declared type is already one — `Address.fromJson` *is* a `DmxDecode`,
/// because its `(Object? json, [String path])` signature matches. Passing the
/// tear-off keeps the nested case free of a wrapper that would only re-check a
/// shape the callee checks anyway.
///
/// `indent` is the column the expression starts at, so nested combinators stay
/// readable instead of collapsing into one very long line.
///
/// # Errors
///
/// Fails when the element type has no codec.
pub fn decoder(ty: &DartType, indent: usize) -> Result<String> {
    if ty.nullable {
        let inner = ty.non_null();
        return Ok(format!(
            "(value, path) => dmxNullable<{}>(value, path, {})",
            inner.source,
            decoder(&inner, indent)?
        ));
    }
    if ty.is_declared() {
        return Ok(format!("{}.fromJson", ty.name));
    }
    let pad = " ".repeat(indent);
    Ok(format!(
        "(value, path) => switch (value) {{\n\
         {pad}  final {} value => {},\n\
         {pad}  _ => Err(DecodeError(path, '{}', value)),\n\
         {pad}}}",
        json_shape(ty),
        decode_bound(ty, "value", "path", indent.saturating_add(2))?,
        ty.source
    ))
}

/// `?` when the type is nullable — for null-aware chains in encodes.
fn nq(ty: &DartType) -> &'static str {
    if ty.nullable { "?" } else { "" }
}

/// [model.deferred] codec table, encode side. `expr` is the field access.
#[must_use]
pub fn encode(ty: &DartType, expr: &str, depth: usize) -> String {
    if ty.is_identity() {
        return expr.to_owned();
    }
    let q = nq(ty);
    let inner = depth.saturating_add(1);
    match (ty.name.as_str(), ty.args.as_slice()) {
        ("DateTime", _) => format!("{expr}{q}.toIso8601String()"),
        ("Uri" | "BigInt", _) => format!("{expr}{q}.toString()"),
        ("Duration", _) => format!("{expr}{q}.inMicroseconds"),
        ("List" | "Set" | "Iterable", [element]) if element.is_identity() => {
            format!("{expr}{q}.toList()")
        }
        ("List" | "Set" | "Iterable", [element]) => {
            let var = format!("e{depth}");
            format!(
                "{expr}{q}.map(({var}) => {}).toList()",
                encode(element, &var, inner)
            )
        }
        ("Map", [_, values]) => {
            let (kv, vv) = (format!("k{depth}"), format!("v{depth}"));
            format!(
                "{expr}{q}.map(({kv}, {vv}) => MapEntry({kv}, {}))",
                encode(values, &vv, inner)
            )
        }
        // A collection with the wrong arity never reaches here: the decode side
        // refuses it first [model.json-codec], and encoding it as itself is the
        // one answer that cannot invent a shape.
        _ => format!("{expr}{q}.toJson()"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(ty: &str) -> DartType {
        DartType::parse(ty).unwrap()
    }

    /// [model.json-codec]: decoding is total — no `throw`, no `as`, no `!`.
    #[test]
    fn decoding_never_throws_or_casts() {
        for ty in [
            "int",
            "String",
            "DateTime",
            "List<int>",
            "Set<String>",
            "Address",
        ] {
            let out = decode_bound(&parse(ty), "value", "'$path.f'", 0).unwrap();
            for forbidden in ["throw", " as ", "!"] {
                assert!(
                    !out.contains(forbidden),
                    "`{forbidden}` in decode of {ty}: {out}"
                );
            }
        }
        for ty in ["String?", "List<String>?", "Map<String, int>?"] {
            let out = decoder(&parse(ty), 0).unwrap();
            for forbidden in ["throw", " as ", "!"] {
                assert!(
                    !out.contains(forbidden),
                    "`{forbidden}` in decoder for {ty}"
                );
            }
        }
    }

    #[test]
    fn decode_shapes() {
        assert_eq!(
            decode_bound(&parse("int"), "age", "'$path.age'", 0).unwrap(),
            "Ok(age)"
        );
        assert_eq!(
            decode_bound(&parse("DateTime"), "at", "'$path.at'", 0).unwrap(),
            "switch (DateTime.tryParse(at)) { \
             final DateTime parsed => Ok<DateTime, DecodeError>(parsed), \
             null => Err<DateTime, DecodeError>(DecodeError('$path.at', 'DateTime', at)) }"
        );
        assert_eq!(
            decode_bound(&parse("Address"), "a", "'$path.a'", 0).unwrap(),
            "Address.fromJson(a, '$path.a')"
        );
        assert_eq!(
            decode_bound(&parse("List<String>"), "tags", "'$path.tags'", 0).unwrap(),
            "dmxList<String>(tags, '$path.tags', (value, path) => switch (value) {\n\
             \x20 final String value => Ok(value),\n\
             \x20 _ => Err(DecodeError(path, 'String', value)),\n\
             })"
        );
        // Nested nullability composes through dmxNullable.
        assert!(
            decoder(&parse("List<String?>"), 0)
                .unwrap()
                .contains("dmxNullable<String>(value, path,")
        );
    }

    /// A declared type is its own decoder, whatever kind of declaration it is.
    #[test]
    fn declared_types_decode_themselves() {
        assert_eq!(decoder(&parse("Address"), 0).unwrap(), "Address.fromJson");
        assert_eq!(json_shape(&parse("Address")), "Object?");
        assert_eq!(
            decode_bound(&parse("List<Status>"), "s", "'$path.s'", 0).unwrap(),
            "dmxList<Status>(s, '$path.s', Status.fromJson)"
        );
    }

    /// The JSON shape a map pattern must bind before decoding [model.json-codec].
    #[test]
    fn json_shapes() {
        assert_eq!(json_shape(&parse("DateTime")), "String");
        assert_eq!(json_shape(&parse("List<Address>")), "List<dynamic>");
        assert_eq!(
            json_shape(&parse("Map<String, int>")),
            "Map<String, dynamic>"
        );
        assert_eq!(json_shape(&parse("double")), "num");
        assert_eq!(json_shape(&parse("String")), "String");
    }

    #[test]
    fn encode_expressions() {
        let encode_of = |ty: &str, e: &str| encode(&parse(ty), e, 0);
        assert_eq!(encode_of("List<String>", "tags"), "tags");
        assert_eq!(encode_of("Set<int>", "ids"), "ids.toList()");
        assert_eq!(encode_of("Address?", "home"), "home?.toJson()");
        assert_eq!(encode_of("DateTime", "at"), "at.toIso8601String()");
        assert_eq!(
            encode_of("List<Address>", "stops"),
            "stops.map((e0) => e0.toJson()).toList()"
        );
    }
}
