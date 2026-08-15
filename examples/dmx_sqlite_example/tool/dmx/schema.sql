-- The database schema — the SOURCE OF TRUTH the macro reads [dartmacros].
--
-- Every field of every class in lib/ comes from this file, through the live
-- database. Add a column here, run `make example-sqlite`, and the Dart field
-- appears. Change a type here and the Dart type changes with it. Nothing in
-- lib/ is written by hand, and nothing there restates this — not even the
-- table name, which the macro works out from the class name.
--
-- The declared type is what the macro maps: TEXT to String, INTEGER to int,
-- REAL to double, BOOLEAN to bool. SQLite keeps the word you wrote, so
-- `BOOLEAN` is how this schema says "this one is a bool".
--
-- The macro also reads what a column IS, not only what it holds: a PRIMARY KEY
-- becomes a keyed SELECT, a REFERENCES becomes a lookup by that foreign key,
-- and a view — which SQLite cannot insert into — gets no INSERT at all.

CREATE TABLE customers (
  id TEXT NOT NULL PRIMARY KEY,
  email TEXT NOT NULL,
  display_name TEXT NOT NULL,
  signed_up_at TEXT NOT NULL,
  marketing_opt_in BOOLEAN NOT NULL,
  loyalty_points INTEGER
);

CREATE TABLE products (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  price_cents INTEGER NOT NULL,
  in_stock BOOLEAN NOT NULL,
  published_at TEXT,
  weight_grams REAL
);

CREATE TABLE orders (
  id TEXT NOT NULL PRIMARY KEY,
  customer_id TEXT NOT NULL REFERENCES customers(id),
  placed_at TEXT NOT NULL,
  status TEXT NOT NULL,
  note TEXT
);

-- Two foreign keys and a composite primary key: the generated class carries a
-- two-column key and a lookup for each parent.
CREATE TABLE order_lines (
  order_id TEXT NOT NULL REFERENCES orders(id),
  product_id TEXT NOT NULL REFERENCES products(id),
  quantity INTEGER NOT NULL,
  unit_price_cents INTEGER NOT NULL,
  discount_ratio REAL,
  PRIMARY KEY (order_id, product_id)
);

-- A view is a table to the macro, minus what a view cannot do. CAST is not
-- decoration: `PRAGMA table_info` reports a computed column's declared type as
-- empty, and a column with no declared type is refused rather than guessed at.
CREATE VIEW customer_spend AS
SELECT
  c.id AS customer_id,
  c.display_name AS display_name,
  CAST(COUNT(DISTINCT o.id) AS INTEGER) AS order_count,
  CAST(COALESCE(SUM(l.quantity * l.unit_price_cents), 0) AS INTEGER) AS spent_cents
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.id
LEFT JOIN order_lines l ON l.order_id = o.id
GROUP BY c.id;
