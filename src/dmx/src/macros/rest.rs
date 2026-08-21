//! `@dmx('restClient')` [catalogue.macros] — the HTTP layer nobody should hand-write.
//!
//! The interface is hand-written and readable. The implementation — build the
//! URL, set the headers, encode the body, check the status, decode the
//! payload, classify the failure — is the same dozen lines per endpoint in
//! every app, and that is exactly the work a generator should be doing.
//!
//! Nothing here names an HTTP package: the client talks to a `DmxTransport`,
//! so a test hands it one that returns canned payloads and exercises the real
//! generated code without a mock library [catalogue.macros].

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl, RawMethod, RawParam};
use crate::macros;
use crate::render;
use crate::types::{self, DartType};

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/rest.mustache");

/// The transport a generated method sends through, when the class has none.
const NO_TRANSPORT: &str = "transport";

#[derive(Content)]
/// One endpoint, as the template names its parts.
pub struct MethodCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The `T` of the method's `Future<Result<T, ApiError>>`.
    pub returnType: String,
    /// The parameter list, with the annotations stripped off.
    pub signature: String,
    /// The HTTP verb, upper-cased.
    pub httpMethod: String,
    /// The field the request is sent through.
    pub transport: String,
    /// The field merged over the default headers.
    pub headers: String,
    /// The request URL, already assembled.
    pub urlExpr: String,
    /// The request carries an encoded body.
    pub hasBody: bool,
    /// The body, already encoded.
    pub bodyExpr: String,
    /// A `void` result decodes nothing: the status is the answer.
    pub decodes: bool,
    /// Reading the payload back, as one expression yielding a `Result`.
    pub decodeExpr: String,
}

#[derive(Content)]
/// The whole context `rest.mustache` renders against.
pub struct RestCtx {
    /// The service root, as a Dart string literal.
    pub baseUrl: String,
    /// One entry per endpoint the interface declares.
    pub methods: Vec<MethodCtx>,
}

/// The HTTP verb each binding annotation stands for.
const VERBS: &[(&str, &str)] = &[
    ("get", "GET"),
    ("post", "POST"),
    ("put", "PUT"),
    ("delete", "DELETE"),
];

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "restClient")?;
    render::render(TEMPLATE, &build(decl, file)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl, file: &[RawDecl]) -> Result<RestCtx> {
    let annotation = decl
        .annotation("restClient")
        .context("DMX2000: internal error — reached the rest builder without @dmx('restClient')")?;

    // What a client implements is exactly what the interface leaves open
    // [frontend.name-index]. Its own methods are its own business.
    let interface = decl
        .interfaces
        .iter()
        .find_map(|name| file.iter().find(|d| &d.name == name))
        .with_context(|| {
            format!(
                "DMX2014: `@dmx('restClient')` on `{}` has no interface in this file to \
                 implement; a client implements the abstract class that declares \
                 its endpoints [frontend.name-index]",
                decl.name
            )
        })?;

    let transport = field_named(decl, "DmxTransport").unwrap_or(NO_TRANSPORT.to_owned());
    let headers =
        field_named(decl, "Map<String,String>").unwrap_or_else(|| "<String, String>{}".to_owned());

    let methods = interface
        .methods
        .iter()
        .filter(|method| verb(method).is_some())
        .map(|method| method_context(method, &transport, &headers))
        .collect::<Result<Vec<_>>>()?;
    if methods.is_empty() {
        bail!(
            "DMX2015: `{}` declares no endpoints; an endpoint is an abstract \
             method carrying `@dmx('get')`, `@dmx('post')`, `@dmx('put')`, or `@dmx('delete')`",
            interface.name
        );
    }

    Ok(RestCtx {
        baseUrl: casing::dart_string(
            &annotation
                .arg("baseUrl")
                .map(casing::unquote)
                .unwrap_or_default(),
        ),
        methods,
    })
}

/// The name of the first field whose type is `wanted`, ignoring spacing.
fn field_named(decl: &RawDecl, wanted: &str) -> Option<String> {
    decl.fields
        .iter()
        .find(|field| {
            field
                .type_text
                .as_ref()
                .is_some_and(|text| text.replace(' ', "") == wanted)
        })
        .map(|field| field.name.clone())
}

/// The binding annotation on a method, as (annotation, verb, path).
fn verb(method: &RawMethod) -> Option<(&str, String)> {
    VERBS.iter().find_map(|(name, http)| {
        method
            .annotation(name)
            .and_then(|a| a.arg("path"))
            .map(|path| (*http, casing::unquote(path)))
    })
}

/// Everything the template names about one endpoint.
fn method_context(method: &RawMethod, transport: &str, headers: &str) -> Result<MethodCtx> {
    let (http, path) = verb(method).context("DMX2000: internal error — endpoint without a verb")?;
    let payload = payload_type(method)?;

    let body = method
        .params
        .iter()
        .find(|p| p.annotation("body").is_some());
    let query: Vec<&RawParam> = method
        .params
        .iter()
        .filter(|p| p.annotation("query").is_some())
        .collect();

    Ok(MethodCtx {
        signature: signature(&method.params)?,
        httpMethod: http.to_owned(),
        transport: transport.to_owned(),
        headers: headers.to_owned(),
        urlExpr: url(&path, &query)?,
        hasBody: body.is_some(),
        bodyExpr: match body {
            Some(param) => encode_body(param)?,
            None => String::new(),
        },
        decodes: payload.is_some(),
        decodeExpr: match &payload {
            Some(ty) => decode(ty, &method.name)?,
            None => String::new(),
        },
        returnType: payload.map_or_else(|| "void".to_owned(), |ty| ty.source),
        name: method.name.clone(),
    })
}

/// The `T` a `Future<Result<T, ApiError>>` carries, or `None` for `void`.
fn payload_type(method: &RawMethod) -> Result<Option<DartType>> {
    let declared = method.return_type.as_deref().with_context(|| {
        format!(
            "DMX2016: endpoint `{}` has no return type; an endpoint returns \
             `Future<Result<T, ApiError>>`",
            method.name
        )
    })?;
    let future = DartType::parse(declared)?;
    let result = match future.args.as_slice() {
        [result] if future.name == "Future" => result,
        _ => bail!(
            "DMX2016: endpoint `{}` returns `{declared}`, not \
             `Future<Result<T, ApiError>>`",
            method.name
        ),
    };
    match result.args.as_slice() {
        [payload, _] if result.name == "Result" && payload.name != "void" => {
            Ok(Some(payload.clone()))
        }
        [_, _] if result.name == "Result" => Ok(None),
        _ => bail!(
            "DMX2016: endpoint `{}` returns `{declared}`, not \
             `Future<Result<T, ApiError>>`",
            method.name
        ),
    }
}

/// The implementation's parameter list: the interface's, minus the annotations
/// that said how each one travels.
fn signature(params: &[RawParam]) -> Result<String> {
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for param in params {
        let ty = param.type_text.as_deref().with_context(|| {
            format!("DMX2001: parameter `{}` needs an explicit type", param.name)
        })?;
        match (param.is_named, param.is_required, &param.default_value) {
            (false, ..) => positional.push(format!("{ty} {}", param.name)),
            (true, true, _) => named.push(format!("required {ty} {}", param.name)),
            (true, false, Some(default)) => named.push(format!("{ty} {} = {default}", param.name)),
            // The interface may leave it out — an abstract declaration never
            // has to construct anything — but the implementation cannot.
            (true, false, None) if DartType::parse(ty)?.nullable => {
                named.push(format!("{ty} {}", param.name));
            }
            (true, false, None) => bail!(
                "DMX2017: optional parameter `{}` has no default, so the \
                 generated implementation could not declare it; give it one or \
                 make it nullable",
                param.name
            ),
        }
    }
    if named.is_empty() {
        Ok(positional.join(", "))
    } else {
        positional.push(format!("{{{}}}", named.join(", ")));
        Ok(positional.join(", "))
    }
}

/// The request URL: the path with its `{placeholders}` interpolated, and the
/// query parameters appended without a stray `?` when there are none.
fn url(path: &str, query: &[&RawParam]) -> Result<String> {
    let interpolated: String = path
        .split('/')
        .map(
            |part| match part.strip_prefix('{').and_then(|p| p.strip_suffix('}')) {
                Some(name) => format!("${name}"),
                None => part.to_owned(),
            },
        )
        .collect::<Vec<_>>()
        .join("/");
    let parse = format!("Uri.parse('$baseUrl{interpolated}')");
    if query.is_empty() {
        return Ok(parse);
    }

    let mut entries = Vec::new();
    for param in query {
        let ty = param.type_text.as_deref().with_context(|| {
            format!("DMX2001: parameter `{}` needs an explicit type", param.name)
        })?;
        let key = param
            .annotation("query")
            .and_then(|q| q.arg("name"))
            .map_or_else(|| param.name.clone(), casing::unquote);
        entries.push(format!(
            "{}: {}",
            casing::dart_string(&key),
            macros::query_string(&DartType::parse(ty)?, &param.name)
        ));
    }
    Ok(format!(
        "{parse}.replace(queryParameters: dmxQuery(<String, String?>{{{}}}))",
        entries.join(", ")
    ))
}

/// The request body, encoded the way the codec table encodes anything.
fn encode_body(param: &RawParam) -> Result<String> {
    let ty = param
        .type_text
        .as_deref()
        .with_context(|| format!("DMX2001: parameter `{}` needs an explicit type", param.name))?;
    Ok(types::encode(&DartType::parse(ty)?, &param.name, 0))
}

/// Reading the response payload back, as one expression yielding a `Result`.
fn decode(ty: &DartType, method: &str) -> Result<String> {
    if ty.is_declared() {
        // `Product.fromJson` already takes `(Object? json, [String path])`.
        return Ok(format!("{}.fromJson(response.body, '{method}')", ty.name));
    }
    // Explicit type arguments on the failure arm: without them the two arms'
    // least upper bound widens to `Object` and the enclosing switch stops
    // being a `Result` at all [model.json-codec].
    Ok(format!(
        "switch (response.body) {{ final {shape} body => {bound},          _ => Err<{source}, DecodeError>(DecodeError('{method}', '{source}', response.body)) }}",
        shape = types::json_shape(ty),
        bound = types::decode_bound(
            ty,
            "body",
            &format!("'{method}'"),
            12,
            types::Runtime::IN_CLASS
        )?,
        source = ty.source,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, refusal, rendered_on};

    const API: &str = "abstract interface class Api {\n\
        @dmx('get', {'path': '/products/{id}'}) Future<Result<Product, ApiError>> product(String id);\n\
        @dmx('get', {'path': '/products'}) Future<Result<List<Product>, ApiError>> search(\
            {@dmx('query') required String q, @dmx('query') int page = 1});\n\
        @dmx('post', {'path': '/orders'}) Future<Result<Placed, ApiError>> placeOrder(@dmx('body') Draft draft);\n\
        @dmx('delete', {'path': '/carts/{cartId}'}) Future<Result<void, ApiError>> abandon(String cartId);\n\
        }\n\
        @dmx('restClient', {'baseUrl': 'https://api.example/v1'}) class Client implements Api {\n\
        final DmxTransport transport;\n\
        final Map<String, String> headers;\n\
        }";

    /// The client, which is written after the interface it implements.
    fn client() -> String {
        rendered_on(expand, API, "restClient")
    }

    /// [frontend.name-index]: the endpoints come from the sibling interface.
    #[test]
    fn a_path_parameter_is_interpolated_into_the_url() {
        emits(
            &client(),
            &[
                "static const String baseUrl = 'https://api.example/v1';",
                "method: 'GET',",
                "url: Uri.parse('$baseUrl/products/$id'),",
                "Product.fromJson(response.body, 'product')",
            ],
        );
    }

    /// Query parameters are appended through `dmxQuery`, never concatenated.
    #[test]
    fn query_parameters_go_through_the_query_builder() {
        emits(
            &client(),
            &[
                "Uri.parse('$baseUrl/products').replace(queryParameters: \
                 dmxQuery(<String, String?>{'q': q, 'page': '$page'}))",
                // The implementation declares the interface's own defaults.
                "search({required String q, int page = 1})",
            ],
        );
    }

    /// A body is encoded, and says so in the headers.
    #[test]
    fn a_body_is_encoded_and_declared() {
        emits(
            &client(),
            &[
                "body: draft.toJson(),",
                "'content-type': 'application/json',",
            ],
        );
    }

    /// A `void` result decodes nothing: the status is the answer.
    #[test]
    fn a_void_endpoint_decodes_nothing() {
        emits(
            &client(),
            &[
                "Future<Result<void, ApiError>> abandon(String cartId)",
                "Ok() => Ok<void, ApiError>(null),",
            ],
        );
    }

    /// A collection payload shape-checks before it decodes.
    #[test]
    fn a_collection_payload_is_shape_checked() {
        emits(
            &client(),
            &[
                "final List<dynamic> body => dmxList<Product>(body, 'search', Product.fromJson)",
                // Typed, or the two arms' least upper bound stops being a `Result`.
                "_ => Err<List<Product>, DecodeError>(",
            ],
        );
    }

    /// [diagnostics]: with no interface there is nothing to implement.
    #[test]
    fn a_client_with_no_interface_is_refused() {
        let err = refusal(expand, "@dmx('restClient') class Client {}");
        assert!(err.contains("DMX2014"), "{err}");
    }
}
