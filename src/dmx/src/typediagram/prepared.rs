//! The little prepared values every context object is built out of
//! [context.discipline].
//!
//! Casings, generic lists, constructor fragments, and the first/last markers
//! that let a template lay out a list without arithmetic. None of it knows
//! what a declaration is; all of it is what stops a template counting.
//!
//! A separate file only because [`super::context`] is at the 500-line ceiling.

use serde_json::{Map, Value};

use crate::casing;

/// Adds one prepared value to a context object.
///
/// `Map::insert` returns whatever it displaced, which is never anything here
/// and which `unused_results` obliges every caller to discard. Written out, the
/// builders below would be `let _ =` noise wrapped around the one thing that
/// matters — the name and the value.
pub(super) fn put(out: &mut Map<String, Value>, name: &str, value: impl Into<Value>) {
    drop(out.insert(name.to_owned(), value.into()));
}

/// A name in every casing a template might place it in
/// [context.helpers].
pub(super) fn named(name: &str) -> Map<String, Value> {
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
pub(super) fn generic_list(generics: &[String]) -> String {
    if generics.is_empty() {
        return String::new();
    }
    format!("<{}>", generics.join(", "))
}

/// The named-parameter list a constructor takes, braces included, or the empty
/// string when there is nothing to take.
pub(super) fn constructor_parameters(fields: &[Map<String, Value>]) -> String {
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
pub(super) fn parameter(name: &str, optional: bool) -> String {
    if optional {
        return format!("this.{name}");
    }
    format!("required this.{name}")
}

/// The positional parameter list a free function takes.
pub(super) fn parameter_list(params: &[Map<String, Value>]) -> String {
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
pub(super) fn positioned(items: Vec<Map<String, Value>>) -> Vec<Value> {
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
