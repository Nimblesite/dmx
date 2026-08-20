//! Dynamic Mustache contexts [dartmacros.render].
//!
//! A built-in macro renders a `#[derive(Content)]` struct the Rust context
//! builder filled in [context]. A macro authored in Dart has no Rust struct —
//! its model arrives as JSON over the worker protocol
//! [extensions.worker-protocol] — so this module makes `serde_json::Value`
//! itself a `ramhorns::Content`. One engine, one template dialect, and one
//! whitespace normalizer therefore serve a project's own macros exactly as
//! they serve the catalogue [dartmacros.render].

use ramhorns::encoding::Encoder;
use ramhorns::traits::ContentSequence;
use ramhorns::{Content, Section, Template};
use serde_json::Value;

/// A JSON value read in Mustache's terms [dartmacros.render].
///
/// Truthiness follows the Mustache specification rather than JavaScript:
/// `null`, `false`, `0`, `""`, `[]`, and `{}` are falsy and everything else is
/// truthy, so `{{#items}}` and `{{^items}}` mean what a template author
/// expects of an empty list.
#[derive(Debug, Clone, Copy)]
pub struct Json<'a>(pub &'a Value);

impl Content for Json<'_> {
    fn is_truthy(&self) -> bool {
        match self.0 {
            Value::Null => false,
            Value::Bool(flag) => *flag,
            Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0),
            Value::String(text) => !text.is_empty(),
            Value::Array(items) => !items.is_empty(),
            Value::Object(fields) => !fields.is_empty(),
        }
    }

    fn capacity_hint(&self, _tpl: &Template<'_>) -> usize {
        match self.0 {
            Value::String(text) => text.len(),
            _ => 8,
        }
    }

    /// A variable tag naming a scalar writes that scalar. One naming a list or
    /// an object writes nothing: a container has no single textual reading, and
    /// serializing it back to JSON inside generated Dart would be a bug that
    /// compiles.
    fn render_escaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        match self.0 {
            Value::String(text) => encoder.write_escaped(text),
            Value::Bool(flag) => encoder.write_unescaped(if *flag { "true" } else { "false" }),
            Value::Number(number) => encoder.format_unescaped(number),
            Value::Null | Value::Array(_) | Value::Object(_) => Ok(()),
        }
    }

    fn render_unescaped<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        match self.0 {
            Value::String(text) => encoder.write_unescaped(text),
            _ => self.render_escaped(encoder),
        }
    }

    /// A list section repeats its body once per element; every other truthy
    /// value renders its body once with that value pushed onto the context
    /// stack.
    fn render_section<C, E>(&self, section: Section<'_, C>, encoder: &mut E) -> Result<(), E::Error>
    where
        C: ContentSequence,
        E: Encoder,
    {
        match self.0 {
            Value::Array(items) => {
                for item in items {
                    section.with(&Json(item)).render(encoder)?;
                }
                Ok(())
            }
            _ if self.is_truthy() => section.with(self).render(encoder),
            _ => Ok(()),
        }
    }

    fn render_field_escaped<E: Encoder>(
        &self,
        _hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        self.render_member::<E>(name, |value| value.render_escaped(encoder))
    }

    fn render_field_unescaped<E: Encoder>(
        &self,
        _hash: u64,
        name: &str,
        encoder: &mut E,
    ) -> Result<bool, E::Error> {
        self.render_member::<E>(name, |value| value.render_unescaped(encoder))
    }

    fn render_field_section<C, E>(
        &self,
        _hash: u64,
        name: &str,
        section: Section<'_, C>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        C: ContentSequence,
        E: Encoder,
    {
        self.render_member::<E>(name, |value| value.render_section(section, encoder))
    }

    fn render_field_inverse<C, E>(
        &self,
        _hash: u64,
        name: &str,
        section: Section<'_, C>,
        encoder: &mut E,
    ) -> Result<bool, E::Error>
    where
        C: ContentSequence,
        E: Encoder,
    {
        self.render_member::<E>(name, |value| value.render_inverse(section, encoder))
    }
}

impl<'a> Json<'a> {
    /// One member of a JSON object, or nothing when this value is not an
    /// object or does not carry that name.
    ///
    /// A dotted name walks into nested objects, so `{{source.path}}` reads what
    /// its spelling says it reads. Mustache calls this a dotted name and the
    /// template engine does not resolve it — the tag arrives here whole — so
    /// this is where it has to be understood [dartmacros.render].
    ///
    /// Returning `None` rather than a null is what lets ramhorns walk out to
    /// the enclosing context, so a nested section still reads a name declared
    /// at the root of the model.
    fn field(self, name: &str) -> Option<&'a Value> {
        name.split('.')
            .try_fold(self.0, |value, segment| match value {
                Value::Object(fields) => fields.get(segment),
                _ => None,
            })
    }

    /// Renders the member called `name` with `render`, reporting whether this
    /// value had one.
    ///
    /// The four `render_field_*` methods differ only in which rendering they
    /// ask for; that difference is this closure, and the `Ok(false)` that tells
    /// ramhorns to look further up the context stack is written once.
    fn render_member<E: Encoder>(
        self,
        name: &str,
        render: impl FnOnce(Json<'a>) -> Result<(), E::Error>,
    ) -> Result<bool, E::Error> {
        match self.field(name) {
            Some(value) => render(Json(value)).map(|()| true),
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use serde_json::json;

    /// Renders `template` against `model` the way the driver does.
    fn render(template: &str, model: &serde_json::Value) -> String {
        crate::render::render_json(template, model).expect("render")
    }

    /// [dartmacros.render]: scalars, lists, and nesting read as Mustache says.
    #[test]
    fn renders_a_nested_dynamic_model() {
        let model = json!({
            "class": "Rate",
            "fields": [
                {"name": "base", "type": "String"},
                {"name": "amount", "type": "double"},
            ],
        });
        let out = render(
            "class {{class}} {\n{{#fields}}\n  final {{type}} {{name}};\n{{/fields}}\n}",
            &model,
        );
        assert_eq!(
            out,
            "class Rate {\n  final String base;\n  final double amount;\n}"
        );
    }

    /// [dartmacros.render]: an empty list is falsy, so `{{^}}` is the fallback.
    #[test]
    fn an_empty_list_takes_the_inverse_section() {
        let out = render(
            "{{#fields}}{{name}}{{/fields}}{{^fields}}none{{/fields}}",
            &json!({"fields": []}),
        );
        assert_eq!(out, "none");
    }

    /// [dartmacros.render]: a dotted name walks into a nested object, and one
    /// that names nothing renders nothing.
    #[test]
    fn a_dotted_name_reads_a_nested_member() {
        let model = json!({"source": {"path": "docs/a.dmx.md", "fence": {"line": 7}}});
        assert_eq!(
            render(
                "{{source.path}}:{{source.fence.line}}|{{source.missing}}|{{a.b}}",
                &model
            ),
            "docs/a.dmx.md:7||"
        );
    }

    /// [dartmacros.render]: a section reads names from the enclosing model.
    #[test]
    fn a_section_still_sees_the_root_model() {
        let out = render(
            "{{#methods}}{{client}}.{{name}}\n{{/methods}}",
            &json!({"client": "Api", "methods": [{"name": "getRate"}]}),
        );
        assert_eq!(out, "Api.getRate");
    }

    /// [dartmacros.render]: a Dart type argument needs the triple stache —
    /// the same law the catalogue's own templates obey.
    #[test]
    fn triple_stache_keeps_dart_generics_intact() {
        let model = json!({"type": "List<Rate>"});
        assert_eq!(
            render("final {{{type}}} rates;", &model),
            "final List<Rate> rates;"
        );
        assert_eq!(
            render("final {{type}} rates;", &model),
            "final List&lt;Rate&gt; rates;"
        );
    }

    /// [dartmacros.render]: booleans and numbers render, containers do not.
    #[test]
    fn scalars_render_and_containers_do_not() {
        let model = json!({"nullable": true, "count": 3, "list": [1], "map": {"a": 1}});
        assert_eq!(
            render(
                "{{nullable}}|{{count}}|{{list}}|{{map}}|{{missing}}",
                &model
            ),
            "true|3|||"
        );
    }
}
