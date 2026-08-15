//! `@dmx('cli')` [catalogue.macros] — argv in, a typed object or a usage error out.
//!
//! The flags, the abbreviations, the defaults, the required checks, and the
//! usage text are all restatements of one field list — and the usage text is
//! the one that goes stale first, because nothing checks it.
//!
//! The scan itself lives in the runtime and takes its tables as arguments, so
//! two commands in one file cannot end up sharing one. What is generated here
//! is the tables, the usage text, and the conversion of a scanned argument
//! list into the author's own type.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field};
use crate::render;
use crate::types::DartType;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/cli.mustache");

/// The gutter every usage description is aligned past.
const DESCRIPTION_GAP: usize = 1;

#[derive(Content)]
/// One line of the usage text.
pub struct UsageLineCtx {
    /// One line of the usage text, already escaped.
    pub text: String,
}

#[derive(Content)]
/// One abbreviation and the long name it stands for.
pub struct NameCtx {
    /// The single-letter form of the long name.
    pub abbr: String,
    /// The Dart name this entry is for.
    pub name: String,
}

#[derive(Content)]
/// One argument, by the name it is typed under.
pub struct WireCtx {
    /// The name this is spelled by outside the program.
    pub wire: String,
}

#[derive(Content)]
/// One admissible value.
pub struct LiteralCtx {
    /// The value as a Dart literal.
    pub literal: String,
}

#[derive(Content)]
/// One option with a fixed set of admissible values.
pub struct ConstrainedCtx {
    /// The option name in Pascal case, naming its `allowed` set.
    pub Name: String,
    /// The values this option admits.
    pub allowed: Vec<LiteralCtx>,
}

#[derive(Content)]
/// One argument that can fail, and how it says so.
pub struct CheckedCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The Dart type once the failure patterns have ruled null out.
    pub typeName: String,
    /// A `T?` expression: null is exactly "this argument was not usable".
    pub expr: String,
    /// The record pattern that selects this failure.
    pub failPattern: String,
    /// The `UsageError` that failure produces.
    pub failure: String,
}

#[derive(Content)]
/// One field, as the template names its parts.
pub struct FieldCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// What the constructor receives for this entry.
    pub valueExpr: String,
}

#[derive(Content)]
/// The whole context `cli.mustache` renders against.
pub struct CliCtx {
    /// The class the members are generated into.
    pub className: String,
    /// The command name, as it is typed.
    pub command: String,
    /// How the scan's three components are bound. A component nothing reads is
    /// a wildcard, not a name the analyzer would rightly call dead.
    pub flagsBind: String,
    /// How the scanned option map is bound, or `_` when nothing reads it.
    pub optionsBind: String,
    /// How the scanned positionals are bound, or `_` when nothing reads them.
    pub restBind: String,
    /// What the command is for, shown under the usage line.
    pub description: String,
    /// ` <paths...>` when the command takes positional arguments.
    pub restUsage: String,
    /// The option list, aligned so the descriptions line up.
    pub usageLines: Vec<UsageLineCtx>,
    /// Every abbreviation, paired with the long name it stands for.
    pub abbreviations: Vec<NameCtx>,
    /// Every `--flag` / `--no-flag` pair.
    pub flags: Vec<WireCtx>,
    /// Every `--name value` option.
    pub options: Vec<WireCtx>,
    /// Every option with a fixed set of allowed values.
    pub constrained: Vec<ConstrainedCtx>,
    /// Every argument that can fail, in the order the record checks them.
    pub checked: Vec<CheckedCtx>,
    /// Every field the macro generates for, in source order.
    pub fields: Vec<FieldCtx>,
}

/// What one field is on the command line.
enum Role {
    /// A `--flag` / `--no-flag` pair.
    Flag,
    /// A `--name value` option.
    Option,
    /// A bare positional argument.
    Rest,
}

/// One field, resolved to its role and its command-line spelling.
struct Arg<'a> {
    /// The field this argument fills.
    field: &'a Field<'a>,
    /// What it is on the command line.
    role: Role,
    /// The long name, as it is typed: `--dry-run`.
    wire: String,
    /// The single-letter form, when the author gave one.
    abbr: Option<String>,
    /// What the usage text says about it.
    help: Option<String>,
    /// The values it admits, when it is constrained.
    allowed: Vec<String>,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, _file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "cli")?;
    render::render(TEMPLATE, &build(decl)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl) -> Result<CliCtx> {
    let annotation = decl
        .annotation("cli")
        .context("DMX2000: internal error — reached the cli builder without @dmx('cli')")?;
    let command = annotation.arg("name").map(casing::unquote).context(
        "DMX2019: `@dmx('cli')` needs its command name, e.g. \
             `@dmx('cli', {'name': 'storefront'})`",
    )?;

    let fields = macros::typed_fields(decl)?;
    let args: Vec<Arg<'_>> = fields.iter().map(argument).collect();

    let mut checked = Vec::new();
    let mut values = Vec::new();
    for arg in &args {
        match value_of(arg, &decl.name)? {
            Value::Inline(expr) => values.push(FieldCtx {
                name: arg.field.name().to_owned(),
                valueExpr: expr,
            }),
            Value::Checked(entry) => {
                values.push(FieldCtx {
                    name: arg.field.name().to_owned(),
                    valueExpr: entry.name.clone(),
                });
                checked.push(entry);
            }
        }
    }
    let mut patterns = macros::slot_patterns(checked.len(), "null").into_iter();
    for entry in &mut checked {
        entry.failPattern = patterns.next().unwrap_or_default();
    }

    let bind = |used: bool, declaration: &str, name: &str| {
        if used {
            format!("final {declaration} {name}")
        } else {
            "_".to_owned()
        }
    };

    Ok(CliCtx {
        className: decl.name.clone(),
        flagsBind: bind(
            args.iter().any(|a| matches!(a.role, Role::Flag)),
            "Set<String>",
            "flags",
        ),
        optionsBind: bind(
            args.iter().any(|a| matches!(a.role, Role::Option)),
            "Map<String, String>",
            "options",
        ),
        restBind: bind(
            args.iter().any(|a| matches!(a.role, Role::Rest)),
            "List<String>",
            "rest",
        ),
        description: annotation
            .arg("description")
            .map(casing::unquote)
            .unwrap_or_default(),
        restUsage: if args.iter().any(|a| matches!(a.role, Role::Rest)) {
            " <paths...>".to_owned()
        } else {
            String::new()
        },
        usageLines: usage_lines(&args),
        abbreviations: args
            .iter()
            .filter_map(|arg| {
                arg.abbr.as_ref().map(|abbr| NameCtx {
                    abbr: abbr.clone(),
                    name: arg.wire.clone(),
                })
            })
            .collect(),
        flags: wires(&args, |role| matches!(role, Role::Flag)),
        options: wires(&args, |role| matches!(role, Role::Option)),
        constrained: args
            .iter()
            .filter(|arg| !arg.allowed.is_empty())
            .map(|arg| ConstrainedCtx {
                Name: casing::pascal(arg.field.name()),
                allowed: arg
                    .allowed
                    .iter()
                    .map(|value| LiteralCtx {
                        literal: casing::dart_string(value),
                    })
                    .collect(),
            })
            .collect(),
        command,
        checked,
        fields: values,
    })
}

/// Every argument in one role, by the name it is typed under.
fn wires(args: &[Arg<'_>], wanted: impl Fn(&Role) -> bool) -> Vec<WireCtx> {
    args.iter()
        .filter(|arg| wanted(&arg.role))
        .map(|arg| WireCtx {
            wire: arg.wire.clone(),
        })
        .collect()
}

/// The role and spelling one field takes on the command line.
fn argument<'a>(field: &'a Field<'a>) -> Arg<'a> {
    let (role, annotation) = match (
        field.raw.annotation("flag"),
        field.raw.annotation("opt"),
        field.raw.annotation("rest"),
    ) {
        (Some(flag), ..) => (Role::Flag, Some(flag)),
        (_, Some(opt), _) => (Role::Option, Some(opt)),
        (.., Some(rest)) => (Role::Rest, Some(rest)),
        // An unannotated field is an option: that is what most of them are,
        // and saying so twice helps nobody.
        _ => (Role::Option, None),
    };
    Arg {
        wire: casing::kebab(field.name()),
        abbr: annotation.and_then(|a| a.arg("abbr")).map(casing::unquote),
        help: annotation.and_then(|a| a.arg("help")).map(casing::unquote),
        allowed: annotation
            .and_then(|a| a.arg("allowed"))
            .map(literals)
            .unwrap_or_default(),
        role,
        field,
    }
}

/// The strings of a Dart list literal written in an annotation.
///
/// The argument is source text, not data: `<String>['pretty', 'json']` is
/// taken apart by its own punctuation rather than parsed as anything.
fn literals(list: &str) -> Vec<String> {
    list.split_once('[')
        .and_then(|(_, rest)| rest.rsplit_once(']'))
        .map_or(list, |(inner, _)| inner)
        .split(',')
        .map(|item| casing::unquote(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

/// What the constructor receives for one argument.
enum Value {
    /// Nothing about it can fail, so it reads straight out of the scan.
    Inline(String),
    /// It can fail, so it joins the record that is checked first.
    Checked(CheckedCtx),
}

/// What the constructor receives for one argument.
fn value_of(arg: &Arg<'_>, class: &str) -> Result<Value> {
    let (name, ty) = (arg.field.name(), &arg.field.ty);
    match arg.role {
        Role::Flag if ty.non_null().name != "bool" => bail!(
            "DMX2020: `{class}.{name}` carries `@dmx('flag')` but is a `{}`; a flag is \
             a `bool`",
            ty.source
        ),
        Role::Flag => Ok(Value::Inline(format!("flags.contains('{}')", arg.wire))),
        Role::Rest if ty.source.replace(' ', "") != "List<String>" => bail!(
            "DMX2021: `{class}.{name}` carries `@dmx('rest')` but is a `{}`; the \
             positional arguments are a `List<String>`",
            ty.source
        ),
        Role::Rest => Ok(Value::Inline("rest".to_owned())),
        Role::Option => option(arg, class),
    }
}

/// An option: the one role that can fail on its way in.
fn option(arg: &Arg<'_>, class: &str) -> Result<Value> {
    let (name, ty) = (arg.field.name(), &arg.field.ty);
    let read = format!("options['{}']", arg.wire);
    let default = arg.field.raw.default_value.clone();
    let bare = ty.non_null();
    let parse = converter(&bare, name, class)?;

    // A `String` that is defaulted or nullable, and unconstrained, can never
    // fail to be what it is.
    if parse.is_none() && arg.allowed.is_empty() {
        return Ok(Value::Inline(match (&default, ty.nullable) {
            (Some(default), _) => format!("{read} ?? {default}"),
            (None, true) => read,
            (None, false) => return Ok(checked(arg, &read, None)),
        }));
    }

    let expr = match (&parse, &default) {
        (Some(parse), Some(default)) => {
            format!("switch ({read}) {{ null => {default}, final String value => {parse}(value) }}")
        }
        (Some(parse), None) => {
            format!("switch ({read}) {{ null => null, final String value => {parse}(value) }}")
        }
        (None, Some(default)) => format!("{read} ?? {default}"),
        (None, None) => read,
    };
    Ok(checked(arg, &expr, parse.as_deref()))
}

/// Adds the membership test, when the option is constrained, and states which
/// of the two ways it can fail the message should describe.
fn checked(arg: &Arg<'_>, expr: &str, parse: Option<&str>) -> Value {
    let name = arg.field.name();
    let bare = arg.field.ty.non_null();
    let (expr, message) = match (arg.allowed.is_empty(), parse) {
        (false, _) => (
            format!(
                "switch ({expr}) {{ final {} value when allowed{}.contains(value) => value, \
                 _ => null }}",
                bare.source,
                casing::pascal(name)
            ),
            format!(
                "\"--{}\" must be one of {}.",
                arg.wire,
                arg.allowed.join(", ")
            ),
        ),
        (true, Some(_)) => (
            expr.to_owned(),
            format!("\"--{}\" must be {}.", arg.wire, quantity(&bare)),
        ),
        (true, None) => (
            expr.to_owned(),
            format!("Option \"--{}\" is required.", arg.wire),
        ),
    };
    Value::Checked(CheckedCtx {
        name: name.to_owned(),
        typeName: bare.source.clone(),
        failure: format!("const UsageError({}, usage)", casing::dart_string(&message)),
        failPattern: String::new(), // arity is only known once all of them are in
        expr,
    })
}

/// The `T?`-returning conversion an option needs, or `None` when the argument
/// already is what the field wants.
fn converter(bare: &DartType, name: &str, class: &str) -> Result<Option<String>> {
    match bare.name.as_str() {
        "String" => Ok(None),
        "int" | "double" | "num" | "BigInt" | "Uri" | "DateTime" => {
            Ok(Some(format!("{}.tryParse", bare.name)))
        }
        other => bail!(
            "DMX2022: `{class}.{name}` is a `{other}`, which no command-line \
             argument converts to; an option is a `String` or something with a \
             `tryParse`"
        ),
    }
}

/// What a person is told the option should have been.
fn quantity(bare: &DartType) -> &'static str {
    match bare.name.as_str() {
        "int" | "BigInt" => "a whole number",
        "double" | "num" => "a number",
        "DateTime" => "a date",
        _ => "a valid value",
    }
}

/// The option list, aligned so the descriptions line up under each other.
fn usage_lines(args: &[Arg<'_>]) -> Vec<UsageLineCtx> {
    let specs: Vec<(&Arg<'_>, String)> = args.iter().map(|arg| (arg, spec(arg))).collect();
    let width = specs
        .iter()
        .map(|(_, spec)| spec.len())
        .max()
        .unwrap_or(0)
        .saturating_add(DESCRIPTION_GAP);

    let mut lines = Vec::new();
    for (arg, spec) in specs.iter().filter(|(a, _)| !matches!(a.role, Role::Rest)) {
        lines.push(line(spec, width, arg.help.as_deref().unwrap_or("")));
        // The default belongs under the description, not beside it: the
        // alternative is a column that only some rows have.
        if let (Role::Option, Some(default)) = (&arg.role, &arg.field.raw.default_value) {
            lines.push(line(
                "",
                width,
                &format!("(defaults to {})", shown(default)),
            ));
        }
    }
    for (arg, spec) in specs.iter().filter(|(a, _)| matches!(a.role, Role::Rest)) {
        lines.push(UsageLineCtx {
            text: String::new(),
        });
        lines.push(line(spec, width, arg.help.as_deref().unwrap_or("")));
    }
    lines
}

/// One usage line, its description aligned past the gutter.
fn line(spec: &str, width: usize, description: &str) -> UsageLineCtx {
    UsageLineCtx {
        text: escape(&format!("{spec:width$}{description}")),
    }
}

/// A default as a person reads it: `4`, and `"pretty"` for a string.
fn shown(default: &str) -> String {
    if default.starts_with('\'') || default.starts_with('"') {
        format!("\"{}\"", casing::unquote(default))
    } else {
        default.to_owned()
    }
}

/// How the argument is typed, as the usage text shows it.
fn spec(arg: &Arg<'_>) -> String {
    let name = &arg.wire;
    let long = match &arg.role {
        Role::Rest => return format!("  <{name}...>"),
        Role::Flag => format!("--[no-]{name}"),
        Role::Option if !arg.allowed.is_empty() => {
            format!("--{name}=<{}>", arg.allowed.join("|"))
        }
        Role::Option => format!(
            "--{name}=<{}>",
            arg.field
                .raw
                .annotation("opt")
                .and_then(|opt| opt.arg("valueHelp"))
                .map_or_else(|| "value".to_owned(), casing::unquote)
        ),
    };
    match &arg.abbr {
        Some(abbr) => format!("  -{abbr}, {long}"),
        None => format!("      {long}"),
    }
}

/// Usage text is spliced into a single-quoted Dart literal by the template, so
/// it escapes here rather than going through [`casing::dart_string`].
fn escape(text: &str) -> String {
    casing::dart_string(text).trim_matches('\'').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, refusal, rendered};

    const BUILD: &str = "@dmx('cli', {'name': 'storefront', 'description': 'Generate the example.'}) \
        class BuildOptions { \
        const BuildOptions({required this.out, this.check = false, this.jobs = 4, \
        this.format = 'pretty', this.paths = const <String>[]}); \
        @dmx('opt', {'abbr': 'o', 'help': 'Directory to write into.', 'valueHelp': 'dir'}) final String out; \
        @dmx('flag', {'abbr': 'c', 'help': 'Report what would change.'}) final bool check; \
        @dmx('opt', {'abbr': 'j', 'help': 'Parallel workers.', 'valueHelp': 'n'}) final int jobs; \
        @dmx('opt', {'help': 'Output style.', 'allowed': <String>['pretty', 'json']}) final String format; \
        @dmx('rest', {'help': 'Files to process.'}) final List<String> paths; }";

    /// The tables are what the runtime scanner is handed.
    #[test]
    fn the_tables_come_from_the_field_list() {
        emits(
            &rendered(expand, BUILD),
            &[
                "'o': 'out',",
                "'c': 'check',",
                "static const Set<String> flagNames = <String>{\n    'check',",
                "'out',",
                "static const Set<String> allowedFormat",
            ],
        );
    }

    /// [catalogue.macros]: the usage text is generated, so it cannot go stale.
    #[test]
    fn the_usage_text_lists_every_argument_with_its_default() {
        emits(
            &rendered(expand, BUILD),
            &[
                "Usage: storefront [options] <paths...>",
                "Generate the example.",
                "-o, --out=<dir>",
                "-c, --[no-]check",
                "--format=<pretty|json>",
                "(defaults to 4)",
                // A string default is quoted for the reader. Double quotes, so
                // the single-quoted Dart literal carrying this line needs no
                // escape [hygiene].
                "(defaults to \"pretty\")",
            ],
        );
    }

    /// Every way an argument can be wrong is one arm, and says so.
    #[test]
    fn each_failure_names_what_was_wrong() {
        emits(
            &rendered(expand, BUILD),
            &[
                "Option \"--out\" is required.",
                "must be a whole number",
                "must be one of pretty, json",
            ],
        );
    }

    /// A flag reads out of the scanned set; positionals are the rest.
    #[test]
    fn flags_and_positionals_need_no_checking() {
        emits(
            &rendered(expand, BUILD),
            &["check: flags.contains('check'),", "paths: rest,"],
        );
    }

    /// A long name is what a person types, not what Dart calls the field.
    #[test]
    fn a_camel_case_field_is_a_kebab_case_option() {
        emits(
            &rendered(
                expand,
                "@dmx('cli', {'name': 'seed'}) class S { const S({this.dryRun = false}); \
                 @dmx('flag', {'help': 'Print instead.'}) final bool dryRun; }",
            ),
            &["'dry-run',", "dryRun: flags.contains('dry-run'),"],
        );
    }

    /// [diagnostics]: a flag that is not a boolean is refused.
    #[test]
    fn a_flag_that_is_not_a_bool_is_refused() {
        let err = refusal(
            expand,
            "@dmx('cli', {'name': 'x'}) class S { @dmx('flag') final String loud; }",
        );
        assert!(err.contains("DMX2020"), "{err}");
    }
}
