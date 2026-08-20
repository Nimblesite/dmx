//! The built-in `typeDiagram` macro [typediagram.macro].
//!
//! Every other entry in the registry is triggered by an annotation on a Dart
//! declaration and contributes a region fragment. This one is triggered by a
//! Markdown generation group and contributes whole files — but it is the same
//! registry, the same Mustache engine, the same whitespace normalizer, and the
//! same "validate before anything is written" rule. The different trigger buys
//! a different input, not a second pipeline.
//!
//! What the macro owns is everything target-shaped: which target a template
//! named, whether that target can render every type the definition uses,
//! whether the declared output is a file that target generates, and whether
//! what came out is source that target will accept. What it does not own is
//! where the file goes — that is emission's question [typediagram.output].

use anyhow::{Context as _, Result, bail};

use crate::emit::GeneratedFile;
use crate::render;
use crate::typediagram::{Invocation, context, file_text, target};

/// Every file one generation group produces [typediagram.macro].
///
/// Each template renders once, against its own context, in document order.
/// Rendering one output cannot reach another's context: each is built fresh
/// from the model, which is immutable [typediagram.templates].
///
/// # Errors
///
/// Fails when a template names an unknown target (`DMX8007`), when the
/// definition uses a type the target cannot render (`DMX8004`), when the
/// declared output is not a file that target generates (`DMX8005`), when the
/// template does not compile (`DMX8008`), or when the rendered source is not
/// valid for that target (`DMX4001`, `DMX4003`).
pub fn expand(invocation: &Invocation<'_>) -> Result<Vec<GeneratedFile>> {
    invocation
        .group
        .templates
        .iter()
        .map(|template| {
            let target = target::find(&template.target)
                .with_context(|| where_it_is(invocation, template.fence.line))?;
            invocation
                .model
                .validate_for_target(target.name)
                .map_err(|found| {
                    anyhow::anyhow!(
                        "DMX8004 [typediagram.model]: the typeDiagram definition in {} (fence {}, \
                         line {}) uses types the `{}` target cannot generate:\n{}",
                        invocation.document,
                        invocation.group.definition.ordinal,
                        invocation.group.definition.line,
                        target.name,
                        found.in_document(invocation.group.definition.line)
                    )
                })?;
            require_target_extension(&template.output, target)
                .with_context(|| where_it_is(invocation, template.fence.line))?;

            let model = context::build(
                invocation.document,
                invocation.group,
                template,
                invocation.model,
                target,
            )
            .with_context(|| where_it_is(invocation, template.fence.line))?;
            let body = render::render_json(&template.fence.body, &model).with_context(|| {
                format!(
                    "DMX8008 [typediagram.templates]: the Mustache template generating `{}` does \
                     not compile ({})",
                    template.output,
                    where_it_is(invocation, template.fence.line)
                )
            })?;

            let text = file_text(invocation.document, invocation.group, template, &body);
            (target.validate)(&text, &format!("`{}`", template.output)).with_context(|| {
                format!(
                    "DMX8008 [typediagram.output]: the Mustache template generating `{}` produced \
                     source the `{}` target refuses ({})",
                    template.output,
                    target.name,
                    where_it_is(invocation, template.fence.line)
                )
            })?;
            Ok(GeneratedFile {
                name: template.output.clone(),
                text,
            })
        })
        .collect()
}

/// Where in the document a failure happened, in the terms the author reads.
fn where_it_is(invocation: &Invocation<'_>, line: usize) -> String {
    format!(
        "in {} group {}, definition fence on line {}, template fence on line {line}",
        invocation.document, invocation.group.ordinal, invocation.group.definition.line
    )
}

/// Refuses an output the named target does not generate [typediagram.output].
fn require_target_extension(output: &str, target: &target::Target) -> Result<()> {
    if std::path::Path::new(output)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(target.extension))
    {
        return Ok(());
    }
    bail!(
        "DMX8005 [typediagram.output]: `{output}` does not end in `.{}`, which the `{}` target \
         generates",
        target.extension,
        target.name
    )
}

#[cfg(test)]
mod tests {
    use crate::typediagram::{Invocation, markdown::groups, resolve};

    /// The files a one-group document produces, or the failure it reports.
    fn run(definition: &str, meta: &str, template: &str) -> anyhow::Result<Vec<String>> {
        let document =
            format!("```typeDiagram\n{definition}\n```\n\n```mustache {meta}\n{template}\n```\n");
        let bound = groups(&document)?;
        let group = &bound[0];
        let model = resolve("docs/a.dmx.md", group)?;
        let invocation = Invocation {
            document: "docs/a.dmx.md",
            group,
            model: &model,
        };
        super::expand(&invocation).map(|files| files.into_iter().map(|f| f.text).collect())
    }

    /// The default one-output metadata.
    const OUT: &str = "{\"dmx\":{\"output\":\"lib/a.dart\"}}";

    /// [typediagram.macro]: a template that only places prepared values
    /// generates a complete, owned Dart file.
    #[test]
    fn a_logic_free_template_generates_dart() {
        let files = run(
            "type Product { id: Uuid\n name: String\n price: Decimal\n note: Option<String> }",
            OUT,
            "{{#declarations}}\n{{#isRecord}}\nfinal class {{name}}{{genericDeclaration}} {\n  const {{name}}({{{constructorParameters}}});\n{{#fields}}\n  final {{{dartType}}} {{name}};\n{{/fields}}\n}\n{{/isRecord}}\n{{/declarations}}",
        )
        .expect("generate");
        assert_eq!(files.len(), 1);
        let text = &files[0];
        assert!(
            text.starts_with("// dmx: generated from docs/a.dmx.md"),
            "{text}"
        );
        assert!(text.contains("final class Product {"), "{text}");
        assert!(
            text.contains("const Product({required this.id, required this.name, required this.price, this.note});"),
            "{text}"
        );
        assert!(text.contains("  final String? note;"), "{text}");
    }

    /// [typediagram.templates]: one definition, several templates, each with
    /// its own context and its own output.
    #[test]
    fn one_definition_generates_several_files() {
        let document = "```typeDiagram\ntype A { x: Int }\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/a.dart\"}}\n// {{source.output}}\nfinal class {{#declarations}}{{name}}{{/declarations}} {}\n```\n\n```mustache {\"dmx\":{\"output\":\"lib/b.dart\"}}\n// {{source.output}}\nfinal class {{#declarations}}{{name}}Dto{{/declarations}} {}\n```\n";
        let bound = groups(document).expect("bind");
        let model = resolve("docs/a.dmx.md", &bound[0]).expect("resolve");
        let files = super::expand(&Invocation {
            document: "docs/a.dmx.md",
            group: &bound[0],
            model: &model,
        })
        .expect("generate");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].name, "lib/a.dart");
        assert!(files[0].text.contains("// lib/a.dart"), "{}", files[0].text);
        assert!(files[0].text.contains("final class A {}"));
        assert_eq!(files[1].name, "lib/b.dart");
        assert!(files[1].text.contains("final class ADto {}"));
        assert!(files[1].text.contains("fences 1/3"), "{}", files[1].text);
    }

    /// [typediagram.output]: a template whose render is not valid Dart, or is
    /// Dart that generated code may not contain, fails before any write.
    #[test]
    fn invalid_or_unhygienic_output_is_refused() {
        let error = format!(
            "{:#}",
            run(
                "type A { x: Int }",
                OUT,
                "final class {{#declarations}}{{name}}{{/declarations}} {"
            )
            .expect_err("unbalanced Dart")
        );
        assert!(error.contains("DMX4001"), "{error}");
        assert!(error.contains("template fence on line 5"), "{error}");

        let error = format!(
            "{:#}",
            run(
                "type A { x: Int }",
                OUT,
                "int probe(Object? v) => throw StateError('{{#declarations}}{{name}}{{/declarations}}');",
            )
            .expect_err("throwing Dart")
        );
        assert!(error.contains("DMX4003"), "{error}");
        assert!(error.contains("never throws"), "{error}");
    }

    /// [typediagram.model]: a type the target cannot render fails before the
    /// template runs, naming the document line.
    #[test]
    fn an_unrenderable_type_fails_before_rendering() {
        let error = format!(
            "{:#}",
            run("type A { at: Timestamp }", OUT, "// {{name}}").expect_err("unknown type")
        );
        assert!(error.contains("DMX8004"), "{error}");
        assert!(error.contains("unknown type 'Timestamp'"), "{error}");
    }

    /// [typediagram.output]: a target only generates its own kind of file, and
    /// only targets dmx knows may be named.
    #[test]
    fn targets_and_extensions_are_checked() {
        let error = format!(
            "{:#}",
            run(
                "type A { x: Int }",
                "{\"dmx\":{\"output\":\"lib/a.txt\"}}",
                "// x"
            )
            .expect_err("not a Dart file")
        );
        assert!(error.contains("DMX8005"), "{error}");
        assert!(error.contains("does not end in `.dart`"), "{error}");

        let error = format!(
            "{:#}",
            run(
                "type A { x: Int }",
                "{\"dmx\":{\"output\":\"lib/a.dart\",\"target\":\"kotlin\"}}",
                "// x"
            )
            .expect_err("no such target")
        );
        assert!(error.contains("DMX8007"), "{error}");
    }

    /// [typediagram.templates]: a template that does not compile names the
    /// document, the group, and its own fence.
    #[test]
    fn a_broken_template_names_where_it_is() {
        let error = format!(
            "{:#}",
            run("type A { x: Int }", OUT, "{{> nowhere}}").expect_err("unresolvable partial")
        );
        assert!(error.contains("DMX8008"), "{error}");
        assert!(error.contains("docs/a.dmx.md group 1"), "{error}");
    }
}
