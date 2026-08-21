//! The resolved model in typeDiagram's own JSON shape
//! [typediagram.delivery.baseline].
//!
//! This is the *compatibility surface*, not the template context. It exists so
//! the Rust front end can be held to the upstream parser and model builder by a
//! differential corpus: same definition in, structurally identical model JSON
//! out. Nothing in the generation path reads it — the context builder works
//! from the model directly — so the shape is free to track upstream exactly,
//! `resolution` fields stripped, keys present only where upstream emits them.

use serde_json::{Map, Value, json};

use super::ast::{Decl, Field, Signature, Targeting, TypeRef, Variant};
use super::model::Model;

/// The upstream model-JSON schema this build is pinned to.
pub const SCHEMA_VERSION: u64 = 1;

/// The whole model, in upstream's `ModelJson` shape.
#[must_use]
pub fn to_json(model: &Model) -> Value {
    json!({
        "version": SCHEMA_VERSION,
        "decls": model.decls().iter().map(decl_json).collect::<Vec<_>>(),
    })
}

/// One declaration, with the keys upstream emits for its kind and no others.
fn decl_json(decl: &Decl) -> Value {
    let mut out = Map::new();
    let _ = out.insert("kind".to_owned(), json!(kind_name(decl)));
    let _ = out.insert("name".to_owned(), json!(decl.name()));
    let _ = out.insert("generics".to_owned(), json!(decl.generics()));
    match decl {
        Decl::Record(record) => {
            let _ = out.insert("fields".to_owned(), fields_json(&record.fields));
        }
        Decl::Union(union) => {
            if union.untagged {
                let _ = out.insert("untagged".to_owned(), json!(true));
            }
            let _ = out.insert(
                "variants".to_owned(),
                json!(union.variants.iter().map(variant_json).collect::<Vec<_>>()),
            );
        }
        Decl::Alias(alias) => {
            let _ = out.insert("target".to_owned(), ref_json(&alias.target));
        }
        Decl::Function(function) => {
            let _ = out.insert(
                "signatures".to_owned(),
                json!(
                    function
                        .signatures
                        .iter()
                        .map(signature_json)
                        .collect::<Vec<_>>()
                ),
            );
        }
    }
    if let Some(targeting) = decl.targeting() {
        let _ = out.insert("targeting".to_owned(), targeting_json(targeting));
    }
    Value::Object(out)
}

/// The `kind` discriminator upstream writes.
fn kind_name(decl: &Decl) -> &'static str {
    match decl {
        Decl::Record(_) => "record",
        Decl::Union(_) => "union",
        Decl::Alias(_) => "alias",
        Decl::Function(_) => "function",
    }
}

/// A field list, which is also a parameter list.
fn fields_json(fields: &[Field]) -> Value {
    json!(
        fields
            .iter()
            .map(|field| json!({ "name": field.name, "type": ref_json(&field.ty) }))
            .collect::<Vec<_>>()
    )
}

/// One variant, with `discriminant` present only where the author pinned one.
fn variant_json(variant: &Variant) -> Value {
    let mut out = Map::new();
    let _ = out.insert("name".to_owned(), json!(variant.name));
    let _ = out.insert("fields".to_owned(), fields_json(&variant.fields));
    if let Some(discriminant) = &variant.discriminant {
        let _ = out.insert("discriminant".to_owned(), json!(discriminant));
    }
    Value::Object(out)
}

/// One signature, with `async` present only where it was written.
fn signature_json(signature: &Signature) -> Value {
    let mut out = Map::new();
    let _ = out.insert("params".to_owned(), fields_json(&signature.params));
    let _ = out.insert("returns".to_owned(), ref_json(&signature.returns));
    if signature.is_async {
        let _ = out.insert("async".to_owned(), json!(true));
    }
    Value::Object(out)
}

/// A target filter, with each list present only where it was written.
fn targeting_json(targeting: &Targeting) -> Value {
    let mut out = Map::new();
    if let Some(targets) = &targeting.targets {
        let _ = out.insert("targets".to_owned(), json!(targets));
    }
    if let Some(skipped) = &targeting.skip_targets {
        let _ = out.insert("skipTargets".to_owned(), json!(skipped));
    }
    Value::Object(out)
}

/// One type reference: the name as written and its arguments, recursively.
fn ref_json(reference: &TypeRef) -> Value {
    json!({
        "name": reference.name,
        "args": reference.args.iter().map(ref_json).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::model::Model;
    use super::super::parser::parse;
    use super::to_json;

    /// The model JSON for `source`.
    fn model_json(source: &str) -> serde_json::Value {
        to_json(&Model::resolve(parse(source).expect("parse")).expect("resolve"))
    }

    /// [typediagram.delivery.baseline]: a record carries kind, name, generics,
    /// and fields — and no `resolution`.
    #[test]
    fn a_record_matches_the_upstream_shape() {
        assert_eq!(
            model_json("type Pair<A, B> { first: A, second: List<B> }"),
            json!({
                "version": 1,
                "decls": [{
                    "kind": "record",
                    "name": "Pair",
                    "generics": ["A", "B"],
                    "fields": [
                        {"name": "first", "type": {"name": "A", "args": []}},
                        {"name": "second", "type": {"name": "List", "args": [{"name": "B", "args": []}]}},
                    ],
                }],
            })
        );
    }

    /// [typediagram.delivery.baseline]: optional keys appear only where the
    /// author wrote them.
    #[test]
    fn optional_keys_are_absent_unless_written() {
        assert_eq!(
            model_json("untagged union U { A = -1\n B(Int)\n C }"),
            json!({
                "version": 1,
                "decls": [{
                    "kind": "union",
                    "name": "U",
                    "generics": [],
                    "untagged": true,
                    "variants": [
                        {"name": "A", "fields": [], "discriminant": "-1"},
                        {"name": "B", "fields": [{"name": "_0", "type": {"name": "Int", "args": []}}]},
                        {"name": "C", "fields": []},
                    ],
                }],
            })
        );
    }

    /// [typediagram.delivery.baseline]: aliases, functions, overloads, and
    /// targeting all match upstream, `async` included only where written.
    #[test]
    fn aliases_functions_and_targeting_match_upstream() {
        assert_eq!(
            model_json(
                "@skipTargets(go)\nalias Email = String\nfunction read {\n (path: String) -> Bytes\n async (path: String) -> Unit\n}"
            ),
            json!({
                "version": 1,
                "decls": [
                    {
                        "kind": "alias",
                        "name": "Email",
                        "generics": [],
                        "target": {"name": "String", "args": []},
                        "targeting": {"skipTargets": ["go"]},
                    },
                    {
                        "kind": "function",
                        "name": "read",
                        "generics": [],
                        "signatures": [
                            {
                                "params": [{"name": "path", "type": {"name": "String", "args": []}}],
                                "returns": {"name": "Bytes", "args": []},
                            },
                            {
                                "params": [{"name": "path", "type": {"name": "String", "args": []}}],
                                "returns": {"name": "Unit", "args": []},
                                "async": true,
                            },
                        ],
                    },
                ],
            })
        );
    }
}
