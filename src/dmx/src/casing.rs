//! Identifier casing helpers [context.helpers].
//!
//! The complete set the spec pins, and no more: a macro that wants a *seventh*
//! casing is asking for a context variable it should have been given
//! [context.discipline].
//!
//! Everything routes through [`words`], so `orderId`, `order_id`, `ORDER_ID`
//! and `order-id` are the same identifier seen four ways.

/// Splits an identifier into its words. Runs of capitals stay together, so
/// `parseHTTPResponse` is `parse | HTTP | Response`, not `parse | H | T | T | P …`.
#[must_use]
pub fn words(ident: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut word = String::new();
    let chars: Vec<char> = ident.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' || c == '-' || c == ' ' || c == '$' {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            continue;
        }
        let previous = i.checked_sub(1).and_then(|before| chars.get(before));
        let starts_word = c.is_uppercase()
            && !word.is_empty()
            && (previous.is_some_and(|p| p.is_lowercase() || p.is_numeric())
                || chars
                    .get(i.saturating_add(1))
                    .is_some_and(|n| n.is_lowercase()));
        if starts_word {
            out.push(std::mem::take(&mut word));
        }
        word.push(c);
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// The word with its first character recased, and the rest left alone.
fn recase_first(word: &str, first: fn(char) -> String) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(head) => first(head).chars().chain(chars).collect(),
        None => String::new(),
    }
}

/// The word with its first character upper-cased.
fn capitalize(word: &str) -> String {
    recase_first(word, |c| c.to_uppercase().to_string())
}

/// The words, each mapped, joined by `separator`. The three separated casings
/// differ in exactly those two things and in nothing else.
fn joined(ident: &str, separator: &str, case: fn(&str) -> String) -> String {
    words(ident)
        .iter()
        .map(|word| case(word))
        .collect::<Vec<_>>()
        .join(separator)
}

/// `createdAt` as `created_at`.
#[must_use]
pub fn snake(ident: &str) -> String {
    joined(ident, "_", str::to_lowercase)
}

/// `dryRun` as `dry-run` — how an option is typed on a command line.
#[must_use]
pub fn kebab(ident: &str) -> String {
    joined(ident, "-", str::to_lowercase)
}

/// `itemNotReceived` as `ITEM_NOT_RECEIVED`.
#[must_use]
pub fn screaming_snake(ident: &str) -> String {
    joined(ident, "_", str::to_uppercase)
}

/// `order_state` as `OrderState` — a Dart type or accessor name.
#[must_use]
pub fn pascal(ident: &str) -> String {
    words(ident).iter().map(|w| capitalize(w)).collect()
}

/// `AwaitingPayment` as `awaitingPayment` — a Dart identifier.
#[must_use]
pub fn camel(ident: &str) -> String {
    recase_first(&pascal(ident), |c| c.to_lowercase().to_string())
}

/// A human label: `inProgress` reads as `In progress`, and a word already
/// written as an acronym survives intact — `csvURL` is `Csv URL`, never
/// `Csv Url`. A lowercase word is not an acronym, because nothing here could
/// tell one from an ordinary word.
#[must_use]
pub fn label(ident: &str) -> String {
    let words = words(ident);
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let is_acronym = w.len() > 1 && w.chars().all(|c| c.is_uppercase() || c.is_numeric());
            match (i, is_acronym) {
                (_, true) => w.clone(),
                (0, false) => capitalize(&w.to_lowercase()),
                (_, false) => w.to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Wraps `text` as a single-quoted Dart string literal, escaping what must be
/// escaped. Every generated literal goes through here [hygiene].
#[must_use]
pub fn dart_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('\'');
    for c in text.chars() {
        match c {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// The text of a Dart string literal the author wrote in an annotation, with
/// its quotes removed — `'/orders/:id'` is the path `/orders/:id`.
#[must_use]
pub fn unquote(literal: &str) -> String {
    let t = literal.trim();
    for quote in ['\'', '"'] {
        if let Some(inner) = t.strip_prefix(quote).and_then(|s| s.strip_suffix(quote)) {
            return inner.replace("\\'", "'").replace("\\\"", "\"");
        }
    }
    t.to_owned()
}

/// Applies a `@dmx('model', {'fieldRename': …})` policy to a field name.
#[must_use]
pub fn rename(policy: &str, name: &str) -> String {
    match policy {
        "snake" | "snake_case" => snake(name),
        "kebab" | "kebab-case" => kebab(name),
        "pascal" | "PascalCase" => pascal(name),
        "screaming" | "screaming_snake" | "SCREAMING_SNAKE_CASE" => screaming_snake(name),
        _ => camel(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_identifiers() {
        assert_eq!(words("orderId"), ["order", "Id"]);
        assert_eq!(words("parseHTTPResponse"), ["parse", "HTTP", "Response"]);
        assert_eq!(words("order_id"), ["order", "id"]);
        assert_eq!(words("ORDER_ID"), ["ORDER", "ID"]);
    }

    #[test]
    fn casings() {
        assert_eq!(snake("createdAt"), "created_at");
        assert_eq!(kebab("dryRun"), "dry-run");
        assert_eq!(screaming_snake("maxRetries"), "MAX_RETRIES");
        assert_eq!(pascal("order_line"), "OrderLine");
        assert_eq!(camel("order_line"), "orderLine");
        assert_eq!(snake("parseHTTPResponse"), "parse_http_response");
    }

    #[test]
    fn labels_keep_acronyms() {
        assert_eq!(label("inProgress"), "In progress");
        assert_eq!(label("csvURL"), "Csv URL");
        assert_eq!(label("parseHTTPResponse"), "Parse HTTP response");
        assert_eq!(label("shipped"), "Shipped");
    }

    /// [hygiene]: a literal dmx emits can never break out of its quotes.
    #[test]
    fn escapes_string_literals() {
        assert_eq!(dart_string("it's"), r"'it\'s'");
        assert_eq!(dart_string("a$b"), r"'a\$b'");
        assert_eq!(unquote("'/orders/:id'"), "/orders/:id");
    }

    /// Every name the shared corpus pins, chosen to cover what actually breaks
    /// a word splitter: each written casing, acronyms, digits, and separators
    /// that are missing, doubled, leading, or trailing.
    const CORPUS: &[&str] = &[
        "",
        "a",
        "A",
        "rate",
        "orderId",
        "order_id",
        "order-id",
        "ORDER_ID",
        "createdAt",
        "iso_code",
        "iso_numeric",
        "start_date",
        "publish_cadence",
        "publishes_missed",
        "CurrencyDetail",
        "getRates",
        "csvURL",
        "HTTPResponse",
        "parseHTTPResponse",
        "order2Id",
        "__leading",
        "trailing__",
        "double__separator",
        "dollar$sign",
    ];

    /// [context.helpers]: Rust and the Dart port agree, name for name.
    ///
    /// `src/dart_packages/dmx/lib/src/macros/casing.dart` mirrors this module so a macro
    /// authored in Dart spells an identifier exactly as a built-in does. The
    /// two implementations are pinned to one corpus rather than to each
    /// other's source, so drift in either language fails a gate in that
    /// language — Rust here, Dart in `test/casing_test.dart`.
    ///
    /// `UPDATE_GOLDEN=1 cargo test casing` rewrites the file after a
    /// deliberate change, exactly as the golden suite does.
    #[test]
    fn matches_the_shared_casing_corpus() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/casing_corpus.json");
        let expected = serde_json::json!({
            "comment": "Generated by `UPDATE_GOLDEN=1 cargo test casing`. \
                        Read by src/dmx/src/casing.rs and \
                        src/dart_packages/dmx/test/casing_test.dart \
                        so the two implementations cannot drift [context.helpers].",
            "cases": CORPUS
                .iter()
                .map(|name| {
                    serde_json::json!({
                        "input": name,
                        "words": words(name),
                        "camel": camel(name),
                        "pascal": pascal(name),
                        "snake": snake(name),
                    })
                })
                .collect::<Vec<_>>(),
        });
        let rendered = format!("{}\n", serde_json::to_string_pretty(&expected).unwrap());
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            std::fs::write(path, &rendered).expect("write the casing corpus");
        }
        let on_disk = std::fs::read_to_string(path).expect(
            "tests/casing_corpus.json is missing — regenerate it with \
             `UPDATE_GOLDEN=1 cargo test casing`",
        );
        assert_eq!(
            on_disk, rendered,
            "casing drifted from the shared corpus; if the change is deliberate, \
             regenerate with `UPDATE_GOLDEN=1 cargo test casing` and make the Dart \
             port in src/dart_packages/dmx/lib/src/macros/casing.dart match"
        );
    }
}
