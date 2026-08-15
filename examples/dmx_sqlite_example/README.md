# A dmx macro written in Dart, generating one file per table from a live SQLite schema

A complete, self-contained project showing everything you need to write your own macro — no Rust, no fork of dmx, no config file. Nobody types a table name, a class name, or a file name: the macro takes all three from the database [dartmacros.files].

```
tool/dmx/macros.dart        the MACRO, written in Dart      <- you write this
tool/dmx/schema.sql         the database schema             <- and this
lib/schema.dart             ONE annotated seed class        <- and this. That's all.
lib/customer_row.dart       AUTHORED by the macro           <- one file per table
lib/product_row.dart        AUTHORED by the macro
lib/order_row.dart          AUTHORED by the macro
lib/order_line_row.dart     AUTHORED by the macro
lib/customer_spend_row.dart AUTHORED by the macro (a view)
lib/rows.dart               AUTHORED by the macro (the barrel)
test/                       Dart tests over the output      <- proof it works
```

## Run it

```bash
sqlite3 tool/dmx/app.db < tool/dmx/schema.sql   # build the database
dart pub get
dmx build lib --insert-regions                  # generate
dart test                                       # prove it
```

Or from the repo root: `make example-sqlite`. Run `dmx` **from this directory** — the worker is found at `tool/dmx/macros.dart` relative to the working directory [dartmacros.discovery].

## The input — all of it

```dart
@dmx('sqliteSchema')
class Schema {
  //#region
  //#endregion
}
```

No arguments. No table names. The database beside the worker is found by convention, its tables are enumerated by the macro, and each one becomes a whole file the macro names itself: `products` → `ProductRow` in `product_row.dart`, `order_lines` → `OrderLineRow` in `order_line_row.dart`, the `customer_spend` view → a read-only `CustomerSpendRow`. The seed's own region receives the manifest — `Schema.tables`, `Schema.views`, `Schema.rowFiles` — so the code states where everything went.

## Why this needs a custom macro

The macro reads a **live SQLite database** — tables, views, columns, declared types, nullability, primary keys and foreign keys, via `sqlite3` — and writes all of `lib/` from it. No built-in macro could do this, because the answer is in a database file that only this project has [dartmacros].

Every generated file opens with the driver's ownership marker:

```dart
// dmx: generated from schema.dart — do not edit.
```

That line is the ownership protocol [dartmacros.files]: dmx will never overwrite a file that lacks it, and a file that carries it is collected when its table is dropped — the generated tree tracks the schema in **both** directions.

## The output

Per table, read off what the database decided, not what anyone typed:

| Schema | Generated |
| --- | --- |
| `price_cents INTEGER NOT NULL` | `final int priceCents;`, `required` in the constructor |
| `in_stock BOOLEAN NOT NULL` | `final bool inStock;` — stored and read back as `0`/`1` |
| `published_at TEXT` (no NOT NULL) | `final String? publishedAt;`, optional in the constructor |
| `PRIMARY KEY (order_id, product_id)` | `primaryKeyColumns`, `keyValues`, `selectByKeySql` with both columns |
| `REFERENCES customers(id)` | `references` map and a `selectByCustomerIdSql` lookup |
| a `CREATE VIEW` | a read-only class: no `insertSql`, no keys — nothing the database cannot back |

Plus, per table: `tableName`, `columnNames`, `selectAllSql`, `insertSql`/`insertValues`, `toRow()` and a `fromRow` that returns `Result` — a row that is not the table's is a value, never a throw.

## The schema is live

Add a table to `tool/dmx/schema.sql`, rebuild the database, run `dmx build` — **a new Dart file appears**, named after the table, exported from `rows.dart`, listed in `Schema.rowFiles`. Drop the table and the file is collected. Add a column and a field appears in the right file. No Dart file is edited by a human at any step.

## It refuses rather than generating something broken

Give a column a type this macro has no Dart mapping for, and the build stops:

```
error: DMX2100: `@dmx('sqliteSchema')` on `Schema`: DMX3914:
  `products.sku_code` is declared `GEOMETRY`, which sqliteSchema has no Dart
  type for. Use TEXT, INTEGER, REAL or BOOLEAN.
```

A wrong Dart type is a runtime bug in every caller, so it is refused rather than guessed at. A refusal is a returned value (`DmxRefusal`), never a thrown exception. Delete the database and you get `DMX3911`; put two `.db` files beside the worker and you get `DMX3910` — with `{'db': '…'}` as the escape hatch.

## The macro itself

[tool/dmx/macros.dart](tool/dmx/macros.dart) — ordinary Dart. `dmx build` discovers it by path convention, runs it once per build, and talks to it over a JSON protocol the `package:dmx/macros.dart` library hides completely:

```dart
final class SqliteSchema extends DmxMacro {
  @override
  String get name => 'sqliteSchema';

  @override
  DmxOutput expand(DmxInvocation invocation) =>
      DmxFragment(manifest,
          introduced: ['tables', 'views', 'rowFiles'],
          files: [DmxGeneratedFile('product_row.dart', source), /* … */]);
}

Future<void> main() => dmxServeMacros([const SqliteSchema()]);
```

A project with no `tool/dmx/macros.dart` never starts a Dart process at all.
