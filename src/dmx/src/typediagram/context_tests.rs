//! What one generation group's Mustache context has to contain
//! [typediagram.model].
//!
//! Every name a template may place is asserted here, because the context *is*
//! the contract: a template author reads these names out of `dmx explain` and
//! writes them into a fence, and renaming one silently breaks every document
//! in every project that used it.

use serde_json::{Value, json};

use super::super::markdown::groups;
use super::super::model::Model;
use super::super::parser::parse;
use super::super::target;
use super::build;

/// The context a one-template document over `definition` produces.
fn context(definition: &str) -> Value {
    let document = format!(
        "```typeDiagram\n{definition}\n```\n\n```mustache {{\"dmx\":{{\"output\":\"lib/a.dart\"}}}}\nx\n```\n"
    );
    let bound = groups(&document).expect("bind");
    let model = Model::resolve(parse(&bound[0].definition.body).expect("parse")).expect("resolve");
    let target = target::find("dart").expect("dart target");
    model.validate_for_target("dart").expect("resolvable");
    build(
        "docs/a.dmx.md",
        &bound[0],
        &bound[0].templates[0],
        &model,
        target,
    )
    .expect("context")
}

/// The first declaration of the context for `definition`.
fn first(definition: &str) -> Value {
    context(definition)["declarations"][0].clone()
}

/// [typediagram.model]: the root names the document, both fences, and the
/// context version.
#[test]
fn the_root_locates_the_group_in_its_document() {
    let root = context("type A { x: Int }");
    assert_eq!(root["modelVersion"], json!(1));
    assert_eq!(root["target"], json!("dart"));
    assert_eq!(root["source"]["path"], json!("docs/a.dmx.md"));
    assert_eq!(root["source"]["group"], json!(1));
    assert_eq!(root["source"]["definitionFence"], json!(1));
    assert_eq!(root["source"]["templateFence"], json!(2));
    assert_eq!(root["source"]["definitionLine"], json!(1));
    assert_eq!(root["source"]["output"], json!("lib/a.dart"));
}

/// [typediagram.model]: kind flags are mutually exclusive and every
/// declaration appears exactly once, in source order.
#[test]
fn kind_flags_replace_per_kind_lists() {
    let declarations =
        context("type A { x: Int }\nunion B { C }\nalias D = String\nfunction e() -> Unit");
    let declarations = declarations["declarations"].as_array().expect("array");
    assert_eq!(
        declarations
            .iter()
            .map(|d| d["name"].as_str().unwrap_or_default())
            .collect::<Vec<_>>(),
        ["A", "B", "D", "e"]
    );
    for (index, flag) in ["isRecord", "isUnion", "isAlias", "isFunction"]
        .into_iter()
        .enumerate()
    {
        for (other, declaration) in declarations.iter().enumerate() {
            assert_eq!(
                declaration[flag],
                json!(index == other),
                "{flag} on {}",
                declaration["name"]
            );
        }
    }
    assert_eq!(declarations[0]["first"], json!(true));
    assert_eq!(declarations[3]["last"], json!(true));
    assert_eq!(declarations[0]["comma"], json!(","));
    assert_eq!(declarations[3]["comma"], json!(""));
}

/// [typediagram.model]: a field arrives with its casings, its target type,
/// its typeDiagram spelling, and its constructor fragment.
#[test]
fn a_field_is_ready_to_place() {
    let record = first("type Order<T> { order_id: Uuid, lines: List<T>, note: Option<String> }");
    assert_eq!(record["genericDeclaration"], json!("<T>"));
    assert_eq!(record["hasGenerics"], json!(true));
    assert_eq!(record["generics"][0]["name"], json!("T"));
    assert_eq!(
        record["constructorParameters"],
        json!("{required this.order_id, required this.lines, this.note}")
    );
    let field = &record["fields"][0];
    assert_eq!(field["camelName"], json!("orderId"));
    assert_eq!(field["pascalName"], json!("OrderId"));
    assert_eq!(field["snakeName"], json!("order_id"));
    assert_eq!(field["screamingSnakeName"], json!("ORDER_ID"));
    assert_eq!(field["dartType"], json!("String"));
    assert_eq!(field["targetType"], json!("String"));
    assert_eq!(field["typeDiagram"], json!("Uuid"));
    assert_eq!(field["isRequired"], json!(true));
    assert_eq!(record["fields"][1]["type"]["isList"], json!(true));
    assert_eq!(
        record["fields"][1]["type"]["arguments"][0]["isTypeParam"],
        json!(true)
    );
    assert_eq!(record["fields"][2]["isOptional"], json!(true));
    assert_eq!(record["fields"][2]["parameter"], json!("this.note"));
    assert_eq!(record["fields"][2]["dartType"], json!("String?"));
}

/// [typediagram.model]: every variant form arrives distinguishable.
#[test]
fn variants_carry_their_shape() {
    let union =
        first("union Shape { Circle { radius: Float }\n Pair(Int, Int)\n Point\n Code = -32700 }");
    assert_eq!(union["untagged"], json!(false));
    assert_eq!(union["hasVariants"], json!(true));
    let variants = union["variants"].as_array().expect("array");
    assert_eq!(
        variants[0]["constructorParameters"],
        json!("{required this.radius}")
    );
    assert_eq!(variants[1]["isTuple"], json!(true));
    assert_eq!(variants[1]["fields"][1]["name"], json!("_1"));
    assert_eq!(variants[2]["isBare"], json!(true));
    assert_eq!(variants[2]["constructorParameters"], json!(""));
    assert_eq!(variants[0]["owner"], json!("Shape"));
    assert_eq!(variants[0]["ownerGenericDeclaration"], json!(""));
    assert_eq!(variants[3]["hasDiscriminant"], json!(true));
    assert_eq!(variants[3]["discriminant"], json!("-32700"));
    assert_eq!(variants[0]["hasDiscriminant"], json!(false));
}

/// [typediagram.model]: a variant reaches the union it belongs to, whose
/// own name its own would otherwise hide.
#[test]
fn a_variant_names_the_union_it_belongs_to() {
    let union = first("union Result<T, E> { Ok { value: T }\n Err { error: E } }");
    let variants = union["variants"].as_array().expect("array");
    assert_eq!(variants[0]["name"], json!("Ok"));
    assert_eq!(variants[0]["owner"], json!("Result"));
    assert_eq!(variants[0]["ownerGenericDeclaration"], json!("<T, E>"));
    assert_eq!(variants[1]["owner"], json!("Result"));
}

/// [typediagram.model]: an alias exposes its target both ways, and a
/// function exposes a ready parameter list per signature.
#[test]
fn aliases_and_functions_are_ready_to_place() {
    let alias = first("alias Ids = List<Uuid>");
    assert_eq!(alias["dartType"], json!("List<String>"));
    assert_eq!(alias["target"]["typeDiagram"], json!("List<Uuid>"));

    let function = first(
        "function read {\n (path: String) -> Bytes\n async (path: String, timeout: Float) -> Unit\n}",
    );
    let signatures = function["signatures"].as_array().expect("array");
    assert_eq!(signatures[0]["parameterList"], json!("String path"));
    assert_eq!(signatures[0]["returnType"], json!("List<int>"));
    assert_eq!(signatures[0]["isAsync"], json!(false));
    assert_eq!(
        signatures[1]["parameterList"],
        json!("String path, double timeout")
    );
    assert_eq!(signatures[1]["isAsync"], json!(true));
    assert_eq!(signatures[1]["returnType"], json!("void"));
}

/// [typediagram.model]: a declaration another target owns is not in this
/// target's context at all.
#[test]
fn targeting_removes_a_declaration_from_the_context() {
    let declarations = context("@skipTargets(dart)\ntype Hidden { x: Int }\ntype Shown { y: Int }");
    let declarations = declarations["declarations"].as_array().expect("array");
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0]["name"], json!("Shown"));
    assert_eq!(declarations[0]["first"], json!(true));
    assert_eq!(declarations[0]["last"], json!(true));
}
