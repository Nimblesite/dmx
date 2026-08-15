//! `@dmx('table')` [catalogue.macros] — the schema and the row mapper, from one class.
//!
//! Two things drift in every app with a local database: the `CREATE TABLE` in
//! a migration, and the row mapper next to the model. They drift because they
//! are two hand-written encodings of one fact. Here the class is the fact.
//!
//! The SQL is plain text and every statement is parameterised — no value is
//! ever interpolated into it, because that is how injection happens.

#![allow(non_snake_case)] // context keys are camelCase, matching [context]

use anyhow::{Context as _, Result, bail};
use ramhorns::Content;

use crate::casing;
use crate::frontend::{Annotated, DeclKind, RawDecl};
use crate::macros::{self, Field};
use crate::render;
use crate::types::DartType;

/// The template this macro renders [rendering].
const TEMPLATE: &str = include_str!("../../templates/table.mustache");

/// The column a table is keyed by when `@dmx('table')` does not name one.
const DEFAULT_PRIMARY_KEY: &str = "id";

#[derive(Content)]
/// One column, as the template names its parts.
// Each of these is one mustache section, and a section is a boolean. The
// state enum the lint asks for is not something a template can switch on.
#[allow(clippy::struct_excessive_bools)]
pub struct ColumnCtx {
    /// The Dart name this entry is for.
    pub name: String,
    /// The column name, bare, for the SQL text.
    pub column: String,
    /// The same name as a Dart string literal, for `columns` and `toRow`.
    pub key: String,
    /// The `SQLite` storage class.
    pub sqlType: String,
    /// ` NOT NULL PRIMARY KEY`, ` NOT NULL REFERENCES orders(id)`, or nothing.
    pub constraints: String,
    /// `@dmx('column', {'indexed': true})`. Read by the index list, not by the DDL.
    pub indexed: bool,
    /// This entry on the way out.
    pub encodeExpr: String,
    /// The local name the pattern binds this entry to.
    pub bind: String,
    /// The Dart type the pattern binds this entry at.
    pub patternType: String,
    /// Destructured by the map pattern rather than read by key.
    pub inPattern: bool,
    /// Contributes a `Result` to the record that sequences the decode.
    pub isComplex: bool,
    /// The `Result` this entry contributes.
    pub resultExpr: String,
    /// Record pattern selecting this entry's failure.
    pub errPattern: String,
    /// What the constructor receives once the patterns have matched.
    pub ctorExpr: String,
    /// Marks the final entry, so a template lays out separators without arithmetic.
    pub isLast: bool,
}

#[derive(Content)]
/// One index the schema declares.
pub struct IndexCtx {
    /// One complete SQL statement.
    pub statement: String,
}

#[derive(Content)]
/// One column an upsert overwrites.
pub struct UpdateCtx {
    /// The column name, bare, for the SQL text.
    pub column: String,
    /// Marks the final entry, so a template lays out separators without arithmetic.
    pub isLast: bool,
}

#[derive(Content)]
/// The whole context `table.mustache` renders against.
pub struct TableCtx {
    /// The class the members are generated into.
    pub className: String,
    /// The table name, bare, for the SQL text.
    pub table: String,
    /// The same name as a Dart string literal.
    pub tableName: String,
    /// The column the table is keyed by.
    pub primaryKey: String,
    /// Every column, in field order.
    pub columns: Vec<ColumnCtx>,
    /// One `CREATE INDEX` per indexed column.
    pub indexes: Vec<IndexCtx>,
    /// Every column an upsert overwrites — all of them but the key.
    pub updates: Vec<UpdateCtx>,
    /// `id, title, price_cents`.
    pub columnList: String,
    /// One `?` per column.
    pub placeholders: String,
    /// At least one required entry, so the decode opens with a map pattern.
    pub hasPattern: bool,
    /// At least one entry decodes to a `Result`, so a record sequences them.
    pub hasComplex: bool,
}

/// How one Dart type is stored and written out.
struct Storage {
    /// The `SQLite` storage class.
    sql: &'static str,
    /// The Dart type the column binds as on the way in.
    pattern: &'static str,
    /// The stored value as an expression over the field.
    encode: String,
    /// Reading it back cannot fail once the pattern has proved its shape, so
    /// no `Result` sequences it.
    direct: bool,
}

/// This macro's fragment for `decl`, rendered [rendering].
pub fn expand(decl: &RawDecl, _file: &[RawDecl]) -> Result<String> {
    macros::require(decl, DeclKind::Class, "table")?;
    render::render(TEMPLATE, &build(decl)?)
}

/// Everything the template names, computed here [authoring.intelligence].
fn build(decl: &RawDecl) -> Result<TableCtx> {
    let annotation = decl
        .annotation("table")
        .context("DMX2000: internal error — reached the table builder without @dmx('table')")?;
    let table = annotation
        .arg("name")
        .map_or_else(|| casing::snake(&decl.name), casing::unquote);
    let key = annotation
        .arg("primaryKey")
        .map_or_else(|| DEFAULT_PRIMARY_KEY.to_owned(), casing::unquote);

    let mut columns = Vec::new();
    for field in macros::typed_fields(decl)? {
        if column_flag(&field, "ignore") {
            continue;
        }
        columns.push(column(&field, &key, &decl.name)?);
    }
    if columns.is_empty() {
        bail!(
            "DMX2013: `@dmx('table')` on `{}` found no columns; a table with no \
             columns has no DDL to write",
            decl.name
        );
    }

    let arity = columns.iter().filter(|c| c.isComplex).count();
    let mut patterns = macros::error_patterns(arity).into_iter();
    for column in columns.iter_mut().filter(|c| c.isComplex) {
        column.errPattern = patterns.next().unwrap_or_default();
    }
    macros::mark_last(&mut columns, |c| c.isLast = true);

    let names: Vec<&str> = columns.iter().map(|c| c.column.as_str()).collect();
    let mut updates: Vec<UpdateCtx> = names
        .iter()
        .filter(|name| **name != key)
        .map(|name| UpdateCtx {
            column: (*name).to_owned(),
            isLast: false,
        })
        .collect();
    macros::mark_last(&mut updates, |u| u.isLast = true);

    Ok(TableCtx {
        className: decl.name.clone(),
        tableName: casing::dart_string(&table),
        columnList: names.join(", "),
        placeholders: vec!["?"; names.len()].join(", "),
        indexes: indexes(&columns, &table),
        hasPattern: columns.iter().any(|c| c.inPattern),
        hasComplex: arity > 0,
        primaryKey: key,
        table,
        columns,
        updates,
    })
}

/// `@dmx('column', {'indexed': true})` on a field, or the flag's absence.
fn column_flag(field: &Field<'_>, flag: &str) -> bool {
    field
        .raw
        .annotation("column")
        .and_then(|c| c.flag(flag))
        .unwrap_or(false)
}

/// One `@dmx('column')` argument, unquoted.
fn column_arg(field: &Field<'_>, arg: &str) -> Option<String> {
    field
        .raw
        .annotation("column")
        .and_then(|c| c.arg(arg))
        .map(casing::unquote)
}

/// One `CREATE INDEX` statement per indexed column.
fn indexes(columns: &[ColumnCtx], table: &str) -> Vec<IndexCtx> {
    columns
        .iter()
        .filter(|column| column.indexed)
        .map(|column| IndexCtx {
            statement: format!(
                "CREATE INDEX IF NOT EXISTS {table}_{0}_idx ON {table} ({0})",
                column.column
            ),
        })
        .collect()
}

/// Everything the template names about one column.
fn column(field: &Field<'_>, key: &str, class: &str) -> Result<ColumnCtx> {
    let name = field.name();
    let column = column_arg(field, "name").unwrap_or_else(|| casing::snake(name));
    let literal = casing::dart_string(&column);
    let bind = macros::binding_name(name);
    let path = format!("'$path.{column}'");
    let ty = &field.ty;
    let storage = storage(ty, name, class)?;

    // A required column is proved by the map pattern; a nullable one is read
    // through `dmxKey`, so an absent column decodes as null rather than as a
    // shape mismatch.
    let (decoder, yields) = leaf(&ty.non_null());
    let (is_complex, result, ctor) = match (ty.nullable, storage.direct) {
        (false, true) => (false, String::new(), transform(&ty.non_null(), &bind)),
        (false, false) => (true, format!("{decoder}({bind}, {path})"), bind.clone()),
        (true, _) => (
            true,
            format!("dmxNullable<{yields}>(dmxKey(row, {literal}), {path}, {decoder})"),
            revive(ty, &bind),
        ),
    };

    Ok(ColumnCtx {
        sqlType: storage.sql.to_owned(),
        constraints: constraints(field, &column, key),
        indexed: column_flag(field, "indexed"),
        encodeExpr: storage.encode,
        patternType: if ty.nullable {
            String::new()
        } else {
            storage.pattern.to_owned()
        },
        inPattern: !ty.nullable,
        isComplex: is_complex,
        resultExpr: result,
        errPattern: String::new(), // arity is only known once all columns are in
        ctorExpr: ctor,
        key: literal,
        name: name.to_owned(),
        column,
        bind,
        isLast: false,
    })
}

/// Turning a decoded nullable binding back into the field's own type.
///
/// The binding is `T?`, so a transform runs only where there is a value — and
/// where there is no transform, there is no conditional either.
fn revive(ty: &DartType, bind: &str) -> String {
    let revived = transform(&ty.non_null(), bind);
    if revived == bind {
        bind.to_owned()
    } else {
        format!("{bind} == null ? null : {revived}")
    }
}

/// The leaf decoder for a stored value, and the Dart type it yields.
///
/// A `bool` is stored as `0`/`1`, so its decoder is `dmxInt` and the
/// constructor expression is what turns the integer back into a `bool`.
fn leaf(ty: &DartType) -> (&'static str, &'static str) {
    match ty.name.as_str() {
        "int" | "bool" | "Duration" => ("dmxInt", "int"),
        "double" | "num" => ("dmxDouble", "double"),
        "DateTime" => ("dmxDateTime", "DateTime"),
        "Uri" => ("dmxUri", "Uri"),
        _ => ("dmxString", "String"),
    }
}

/// The pure expression turning a stored value back into `ty`.
fn transform(ty: &DartType, value: &str) -> String {
    match ty.name.as_str() {
        "bool" => format!("{value} != 0"),
        "double" => format!("{value}.toDouble()"),
        "Duration" => format!("Duration(microseconds: {value})"),
        _ => value.to_owned(),
    }
}

/// [catalogue.macros]: the `SQLite` storage class for one Dart type.
///
/// `DateTime` and `Uri` store as text and come back through a decode that can
/// fail; everything else is stored as itself, or as the integer a boolean is.
fn storage(ty: &DartType, name: &str, class: &str) -> Result<Storage> {
    let (sql, pattern, encode, direct) = match ty.non_null().name.as_str() {
        "String" => ("TEXT", "String", name.to_owned(), true),
        "int" => ("INTEGER", "int", name.to_owned(), true),
        "bool" => ("INTEGER", "int", format!("{name} ? 1 : 0"), true),
        "double" | "num" => ("REAL", "num", name.to_owned(), true),
        "Duration" => ("INTEGER", "int", format!("{name}.inMicroseconds"), true),
        "DateTime" => ("TEXT", "String", format!("{name}.toIso8601String()"), false),
        "Uri" => ("TEXT", "String", format!("{name}.toString()"), false),
        other => bail!(
            "DMX2012: `{class}.{name}` is a `{other}`, which has no SQLite \
             storage class; store it as a column type SQLite has, or mark it \
             `@dmx('column', {{'ignore': true}})`"
        ),
    };
    Ok(Storage {
        sql,
        pattern,
        encode,
        direct,
    })
}

/// The DDL constraints for one column, in the order `SQLite` expects them.
fn constraints(field: &Field<'_>, column: &str, key: &str) -> String {
    let mut out = String::new();
    if !field.ty.nullable {
        out.push_str(" NOT NULL");
    }
    if column == key {
        out.push_str(" PRIMARY KEY");
    }
    if column_flag(field, "unique") {
        out.push_str(" UNIQUE");
    }
    if let Some(references) = column_arg(field, "references") {
        out.push_str(" REFERENCES ");
        out.push_str(&references);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::testing::{emits, omits, refusal, rendered};

    const PRODUCTS: &str = "@dmx('table', {'name': 'products', 'primaryKey': 'id'}) class ProductRow { \
                            final String id; \
                            @dmx('column', {'indexed': true}) final String title; \
                            final int priceCents; \
                            final DateTime publishedAt; \
                            final bool inStock; \
                            final String? description; }";

    /// Each Dart type takes the storage class `SQLite` actually has.
    #[test]
    fn the_ddl_follows_from_the_field_list() {
        emits(
            &rendered(expand, PRODUCTS),
            &[
                "'  id TEXT NOT NULL PRIMARY KEY,\\n'",
                "'  price_cents INTEGER NOT NULL,\\n'",
                "'  published_at TEXT NOT NULL,\\n'",
                "'  in_stock INTEGER NOT NULL,\\n'",
                // Nullable, so no NOT NULL — and last, so no trailing comma.
                "'  description TEXT\\n'",
            ],
        );
    }

    /// [catalogue.macros]: values are bound, never interpolated.
    #[test]
    fn every_statement_is_parameterised() {
        emits(
            &rendered(expand, PRODUCTS),
            &[
                "'VALUES (?, ?, ?, ?, ?, ?)'",
                "'$selectAllSql WHERE id = ?'",
                "'DELETE FROM products WHERE id = ?'",
            ],
        );
    }

    /// An upsert overwrites everything except the key it conflicted on.
    #[test]
    fn an_upsert_never_overwrites_the_key() {
        let out = rendered(expand, PRODUCTS);
        emits(
            &out,
            &[
                "'ON CONFLICT(id) DO UPDATE SET '",
                "'title = excluded.title, '",
            ],
        );
        omits(&out, &["'id = excluded.id"]);
    }

    /// `@dmx('column')` contributes the index and the foreign key, not the DDL type.
    #[test]
    fn column_overrides_reach_the_ddl_and_the_indexes() {
        emits(
            &rendered(
                expand,
                "@dmx('table', {'name': 'order_lines'}) class L { final int id; \
                 @dmx('column', {'references': 'orders(id)', 'indexed': true}) final String orderId; }",
            ),
            &[
                "'  order_id TEXT NOT NULL REFERENCES orders(id)\\n'",
                "'CREATE INDEX IF NOT EXISTS order_lines_order_id_idx \
                 ON order_lines (order_id)',",
            ],
        );
    }

    /// A row is data from outside the program, so it decodes like JSON does.
    #[test]
    fn a_row_decodes_through_patterns_and_a_result() {
        emits(
            &rendered(expand, PRODUCTS),
            &[
                "'in_stock': final int inStock,",
                "inStock: inStock != 0,",
                "dmxDateTime(publishedAt, '$path.published_at'),",
                "dmxNullable<String>(dmxKey(row, 'description'), '$path.description', dmxString)",
            ],
        );
    }

    /// [diagnostics]: a type `SQLite` cannot store is refused in the author's terms.
    #[test]
    fn a_type_with_no_storage_class_is_refused() {
        let err = refusal(
            expand,
            "@dmx('table') class T { final int id; final Address home; }",
        );
        assert!(err.contains("DMX2012"), "{err}");
    }

    /// An ignored field is not a column, so it is not in any statement.
    #[test]
    fn an_ignored_field_is_not_a_column() {
        omits(
            &rendered(
                expand,
                "@dmx('table', {'name': 't'}) class T { final int id; \
                 @dmx('column', {'ignore': true}) final String cached; }",
            ),
            &["cached"],
        );
    }
}
