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

/// The suffix that names a declaration's JSON extension [typediagram.canonical].
///
/// `User` decodes and encodes through `UserJson`. One constant, so the name the
/// codec table calls and the name a template declares can never drift apart.
pub const JSON_EXTENSION: &str = "Json";

/// Where a declared type keeps its decoder [model.json-codec].
///
/// `Address.fromJson` and `AddressJson.fromJson` decode the same value; which
/// one exists depends on where the members were written. The inline backend
/// generates into the class body, so the decoder is a static on the class
/// itself [emission.inline-backend]. Whole-file generation writes the class as
/// a pure data declaration and puts its codec on the `<Name>Json` extension
/// beside it [typediagram.canonical], so the same call has to name the
/// extension.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Decoders {
    /// `Address.fromJson` — the declaration carries its own decoder.
    #[default]
    OnTheType,
    /// `AddressJson.fromJson` — the decoder lives on the type's JSON extension.
    OnTheExtension,
}

/// How generated code reaches the dmx runtime [model.json-codec].
///
/// Every expression the codec table builds names something the runtime exports
/// — `Ok`, `Err`, `DecodeError`, `dmxList` — and what those names resolve to
/// depends on how the file that holds them imported the runtime. The inline
/// backend generates into a file somebody else wrote, whose import it does not
/// control, so it spells the names bare. Whole-file generation writes the
/// import itself and prefixes it [typediagram.canonical]: a diagram is free to
/// declare a type called `Result`, and a local declaration hides an imported
/// name, so bare names would quietly resolve to the wrong type.
///
/// One value threaded through the table, so no caller ever spells a runtime
/// name of its own.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Runtime {
    /// What the runtime import was bound to, trailing dot included, or `""`
    /// when it was imported without a prefix.
    pub prefix: &'static str,
    /// Where a declared type keeps its decoder.
    pub decoders: Decoders,
}

impl Runtime {
    /// The inline backend's: an unprefixed import somebody else wrote, and a
    /// decoder on the class [emission.inline-backend].
    pub const IN_CLASS: Self = Self {
        prefix: "",
        decoders: Decoders::OnTheType,
    };

    /// Whole-file generation's: a prefixed import this generator writes, and a
    /// decoder on the type's JSON extension [typediagram.canonical]. The
    /// prefix is the package's own name, which is what the generated import
    /// binds it to.
    pub const PREFIXED: Self = Self {
        prefix: "dmx.",
        decoders: Decoders::OnTheExtension,
    };

    /// `name`, as generated code has to spell it to reach the runtime.
    #[must_use]
    pub fn name(self, name: &str) -> String {
        format!("{}{name}", self.prefix)
    }

    /// What generated code writes to reach `name`'s decoder. The declaration
    /// and its extension are both local, so neither takes the prefix.
    #[must_use]
    pub fn callee(self, name: &str) -> String {
        match self.decoders {
            Decoders::OnTheType => name.to_owned(),
            Decoders::OnTheExtension => format!("{name}{JSON_EXTENSION}"),
        }
    }
}

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
pub fn decode_bound(
    ty: &DartType,
    value: &str,
    path: &str,
    indent: usize,
    runtime: Runtime,
) -> Result<String> {
    if let Some(expr) = pure_transform(ty, value) {
        return Ok(format!("{}({expr})", runtime.name("Ok")));
    }
    Ok(match ty.name.as_str() {
        // Explicit type arguments: without them the arms' least upper bound
        // widens to `Object` and the enclosing record stops being exhaustive.
        "DateTime" | "Uri" | "BigInt" => format!(
            "switch ({name}.tryParse({value})) {{ \
             final {name} parsed => {ok}<{name}, {error}>(parsed), \
             null => {err}<{name}, {error}>({error}({path}, '{name}', {value})) }}",
            name = ty.name,
            ok = runtime.name("Ok"),
            err = runtime.name("Err"),
            error = runtime.name("DecodeError"),
        ),
        "List" | "Set" | "Iterable" => {
            let [elem] = ty.args.as_slice() else {
                bail!("DMX2102: `{}` needs exactly one type argument", ty.source);
            };
            let combinator = runtime.name(if ty.name == "Set" {
                "dmxSet"
            } else {
                "dmxList"
            });
            format!(
                "{combinator}<{}>({value}, {path}, {})",
                elem.source,
                decoder(elem, indent, runtime)?
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
                "{}<{}>({value}, {path}, {})",
                runtime.name("dmxMap"),
                v.source,
                decoder(v, indent, runtime)?
            )
        }
        // Name-level resolution: an unrecognized simple type decodes itself.
        _ if ty.is_declared() => {
            format!("{}.fromJson({value}, {path})", runtime.callee(&ty.name))
        }
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
pub fn decoder(ty: &DartType, indent: usize, runtime: Runtime) -> Result<String> {
    if ty.nullable {
        let inner = ty.non_null();
        return Ok(format!(
            "(value, path) => {}<{}>(value, path, {})",
            runtime.name("dmxNullable"),
            inner.source,
            decoder(&inner, indent, runtime)?
        ));
    }
    if ty.is_declared() {
        return Ok(format!("{}.fromJson", runtime.callee(&ty.name)));
    }
    let pad = " ".repeat(indent);
    Ok(format!(
        "(value, path) => switch (value) {{\n\
         {pad}  final {} value => {},\n\
         {pad}  _ => {}({}(path, '{}', value)),\n\
         {pad}}}",
        json_shape(ty),
        decode_bound(ty, "value", "path", indent.saturating_add(2), runtime)?,
        runtime.name("Err"),
        runtime.name("DecodeError"),
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
#[path = "types_tests.rs"]
mod tests;
