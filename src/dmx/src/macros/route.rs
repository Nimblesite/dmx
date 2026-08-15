//! `@dmx('route')` [catalogue.macros] — deep links that cannot be typo'd.
//!
//! `location` builds the URL from typed fields and `parse` takes a `Uri` and
//! gives back a `Result`. Both come from the one pattern, so they can never
//! disagree about what `/orders/:id/refund` means. The parser is a list
//! pattern over `uri.pathSegments` — Dart doing the matching, not a regular
//! expression over a URL.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field};
use crate::render;
use crate::types::DartType;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/route.mustache");

#[derive(Content)]
/// One query-string parameter, on the way out.
pub struct QueryCtx {
    /// The query-string name, as a Dart string literal.
    pub key: String,
    /// The field, as the `String?` the query map takes.
    pub encodeExpr: String,
}

#[derive(Content)]
/// One path parameter that arrives as text and narrows.
pub struct TypedCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The Dart type this binds at.
    pub typeName: String,
    /// The `T?`-returning parse of the segment this field was bound from.
    pub parseExpr: String,
}

#[derive(Content)]
/// One constructor argument of the parsed route.
pub struct ParamCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// What the constructor receives once the patterns have matched.
    pub valueExpr: String,
}

#[derive(Content)]
/// The whole context `route.mustache` renders against.
pub struct RouteCtx {
    /// The class the members are generated into.
    pub className: String,
    /// The route pattern, as a Dart string literal.
    pub pattern: String,
    /// A sibling `@dmx('router')` declares `location`, so this one overrides it.
    pub overridesLocation: bool,
    /// At least one field is carried in the query string.
    pub hasQuery: bool,
    /// Every field carried in the query string.
    pub query: Vec<QueryCtx>,
    /// The path half of the URL, interpolating the path parameters.
    pub pathExpr: String,
    /// The list pattern that matches this route's shape.
    pub segmentPattern: String,
    /// At least one path parameter needs a parse.
    pub hasTypedSegments: bool,
    /// Every path parameter that needs a parse to reach its own type.
    pub typed: Vec<TypedCtx>,
    /// What the constructor receives, in field order.
    pub params: Vec<ParamCtx>,
}

/// One piece of a route pattern between the slashes.
enum Segment {
    /// A fixed segment, matched literally.
    Literal(String),
    /// A `:name` segment, bound to the field of that name.
    Param(String),
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "route")?;
    render::render(TEMPLATE, &build(decl, file)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<RouteCtx> {
    let annotation = decl
        .annotation("route")
        .context("DMX2000: internal error — reached the route builder without @dmx('route')")?;
    let pattern = annotation.arg("pattern").map(casing::unquote).context(
        "DMX2010: `@dmx('route')` needs its pattern, e.g. \
             `@dmx('route', {'pattern': '/orders/:id'})`",
    )?;
    let segments = parse_pattern(&pattern);

    let fields = macros::typed_fields(decl)?;
    let mut query = Vec::new();
    let mut typed = Vec::new();
    let mut params = Vec::new();

    for field in &fields {
        if segments
            .iter()
            .any(|s| matches!(s, Segment::Param(p) if p == field.name()))
        {
            if let Some(parse) = segment_parse(&field.ty, field.name()) {
                typed.push(TypedCtx {
                    name: field.name().to_owned(),
                    typeName: field.ty.source.clone(),
                    parseExpr: parse,
                });
            }
            params.push(ParamCtx {
                name: field.name().to_owned(),
                valueExpr: field.name().to_owned(),
            });
        } else {
            let key = query_key(field);
            params.push(ParamCtx {
                name: field.name().to_owned(),
                valueExpr: read_query(field, &key, &decl.name)?,
            });
            query.push(QueryCtx {
                encodeExpr: macros::query_string(&field.ty, field.name()),
                key: casing::dart_string(&key),
            });
        }
    }

    Ok(RouteCtx {
        className: decl.name.clone(),
        // A sibling `@dmx('router')` declares `location` on the base it shares with
        // every other route, so this one is an override [catalogue.macros].
        overridesLocation: macros::base_with(decl, file, "router").is_some(),
        hasQuery: !query.is_empty(),
        pathExpr: path_expression(&segments, &pattern),
        segmentPattern: segment_pattern(&segments, &fields),
        hasTypedSegments: !typed.is_empty(),
        pattern: casing::dart_string(&pattern),
        query,
        typed,
        params,
    })
}

/// Splits a route pattern into its segments.
fn parse_pattern(pattern: &str) -> Vec<Segment> {
    pattern
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| match part.strip_prefix(':') {
            Some(name) => Segment::Param(name.to_owned()),
            None => Segment::Literal(part.to_owned()),
        })
        .collect()
}

/// The list pattern matching this route's shape.
///
/// A parameter binds as `String` whatever it is finally typed as: the segment
/// arrives as text, and anything narrower is a parse that can fail.
fn segment_pattern(segments: &[Segment], fields: &[Field<'_>]) -> String {
    if segments.is_empty() {
        // `Uri.parse('/')` has no segments and `Uri.parse('')` has one empty
        // one. Both are the root, and neither is worth a 404.
        return "[] || ['']".to_owned();
    }
    let parts: Vec<String> = segments
        .iter()
        .map(|segment| match segment {
            Segment::Literal(text) => casing::dart_string(text),
            // A parameter nothing declares is still part of the shape, and
            // still must not bind a name no constructor takes.
            Segment::Param(name) if fields.iter().any(|f| f.name() == name) => {
                format!("final String {name}")
            }
            Segment::Param(_) => "_".to_owned(),
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

/// The path half of `location`, interpolating each parameter where it sits.
fn path_expression(segments: &[Segment], pattern: &str) -> String {
    if segments.iter().all(|s| matches!(s, Segment::Literal(_))) {
        return casing::dart_string(pattern);
    }
    let path: String = segments
        .iter()
        .map(|segment| match segment {
            Segment::Literal(text) => format!("/{text}"),
            // A segment ends at a `/` or at the closing quote, so the simple
            // `$name` form can never run into the text that follows it.
            Segment::Param(name) => format!("/${name}"),
        })
        .collect();
    format!("'{path}'")
}

/// The `T?` parse a non-`String` path parameter needs, or `None` when the
/// segment already is what the field wants.
///
/// `tryParse` is the shape, not a list of blessed types: `int`, `double`,
/// `num`, `BigInt`, `Uri` and `DateTime` all have one, and so does anything an
/// author writes that wants to sit in a path.
fn segment_parse(ty: &DartType, name: &str) -> Option<String> {
    match ty.name.as_str() {
        "String" => None,
        other => Some(format!("{other}.tryParse({name})")),
    }
}

/// The query-string name this field travels under.
fn query_key(field: &Field<'_>) -> String {
    field
        .raw
        .annotation("query")
        .and_then(|q| q.arg("name"))
        .map_or_else(|| field.name().to_owned(), casing::unquote)
}

/// Reading a query parameter back into the field's own type.
///
/// A malformed query string takes the field's declared default rather than
/// failing the whole route: a cold start from a shared link is not the moment
/// to show a 404 because someone truncated `?page=12` to `?page=1x`.
fn read_query(field: &Field<'_>, key: &str, class: &str) -> Result<String> {
    let read = format!("uri.queryParameters['{key}']");
    let (name, ty) = (field.name(), &field.ty);
    let absent = match (&field.raw.default_value, ty.nullable) {
        (Some(default), _) => default.clone(),
        (None, true) => "null".to_owned(),
        (None, false) => bail!(
            "DMX2011: `{class}.{name}` is not in the route pattern and has no default, \
             so an absent `?{key}=` could not be constructed; give it a default or \
             make it nullable"
        ),
    };
    // A nullable `String` *is* what the query map returns, absence included.
    if ty.nullable && ty.non_null().name == "String" {
        return Ok(read);
    }
    let present = match ty.non_null().name.as_str() {
        "String" => "value".to_owned(),
        // `?? null` would be noise the analyzer is right to point at.
        other if absent == "null" => format!("{other}.tryParse(value)"),
        other => format!("{other}.tryParse(value) ?? {absent}"),
    };
    Ok(format!(
        "switch ({read}) {{ final String value => {present}, null => {absent} }}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, refusal, rendered};

    /// The pattern drives both halves, so they cannot disagree.
    #[test]
    fn a_path_parameter_builds_and_parses_from_one_pattern() {
        emits(
            &rendered(
                expand,
                "@dmx('route', {'pattern': '/orders/:orderId'}) class R { final String orderId; }",
            ),
            &[
                "static const String pattern = '/orders/:orderId';",
                "'/orders/$orderId';",
                "['orders', final String orderId] =>",
                "orderId: orderId,",
            ],
        );
    }

    /// The root is both `/` and the empty path, and neither is a 404.
    #[test]
    fn the_root_route_matches_the_empty_path_too() {
        emits(
            &rendered(expand, "@dmx('route', {'pattern': '/'}) class R {}"),
            &["[] || [''] =>", "String get location =>\n      '/';"],
        );
    }

    /// A typed segment arrives as text and narrows through a parse that fails
    /// as data rather than handing a screen a number it cannot use.
    #[test]
    fn a_typed_segment_narrows_through_a_parse() {
        emits(
            &rendered(
                expand,
                "@dmx('route', {'pattern': '/orders/:n/refund'}) class R { final int n; }",
            ),
            &[
                "['orders', final String n, 'refund'] =>",
                "int.tryParse(n),",
                "final int n,",
            ],
        );
    }

    /// [catalogue.macros]: a malformed query takes the declared default.
    #[test]
    fn a_query_parameter_falls_back_on_its_declared_default() {
        emits(
            &rendered(
                expand,
                "@dmx('route', {'pattern': '/products'}) class R { const R({this.tag, this.page = 1}); \
                 @dmx('query') final String? tag; @dmx('query') final int page; }",
            ),
            &[
                "'tag': tag,",
                "'page': '$page',",
                "tag: uri.queryParameters['tag'],",
                "page: switch (uri.queryParameters['page']) \
                 { final String value => int.tryParse(value) ?? 1, null => 1 },",
            ],
        );
    }

    /// [diagnostics]: a field that is neither in the path nor defaulted could
    /// never be constructed, and saying so beats emitting code that says it.
    #[test]
    fn an_unconstructible_query_parameter_is_refused() {
        let err = refusal(
            expand,
            "@dmx('route', {'pattern': '/x'}) class R { @dmx('query') final int page; }",
        );
        assert!(err.contains("DMX2011"), "{err}");
    }

    /// A sibling `@dmx('router')` owns `location`, so a route implements it.
    #[test]
    fn a_route_under_a_router_overrides_location() {
        emits(
            &rendered(
                expand,
                "@dmx('route', {'pattern': '/x'}) class R extends AppRoute {}\n@dmx('router') sealed class AppRoute {}",
            ),
            &["@override\n  String get location"],
        );
    }
}
