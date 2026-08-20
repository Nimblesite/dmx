//! The typeDiagram tokenizer [typediagram.model].
//!
//! A direct port of the upstream lexer's rules, because the compatibility
//! baseline is behavioural: the same bytes must tokenize the same way here as
//! they do in the package that renders the diagram
//! [typediagram.delivery.baseline]. Newlines are tokens rather than
//! whitespace, since the grammar accepts a newline *or* a comma as a separator
//! inside a brace block.

use super::diagnostic::{Diagnostic, Diagnostics};

/// What one token is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// `type`.
    Type,
    /// `union`.
    Union,
    /// `untagged`.
    Untagged,
    /// `alias`.
    Alias,
    /// `function`.
    Function,
    /// `async`.
    Async,
    /// The optional `typeDiagram` file header.
    Header,
    /// A bare identifier.
    Ident,
    /// A (possibly negative, possibly `_`-grouped) integer discriminant.
    Number,
    /// `{`.
    LBrace,
    /// `}`.
    RBrace,
    /// `(`.
    LParen,
    /// `)`.
    RParen,
    /// `<`.
    LAngle,
    /// `>`.
    RAngle,
    /// `@`, which opens a targeting annotation.
    At,
    /// `,`.
    Comma,
    /// `:`.
    Colon,
    /// `=`.
    Equals,
    /// `->`.
    Arrow,
    /// A line break, which separates fields and variants.
    Newline,
    /// The end of the definition.
    Eof,
}

impl Kind {
    /// How the token reads in a diagnostic, in the author's terms.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Self::Type => "'type'",
            Self::Union => "'union'",
            Self::Untagged => "'untagged'",
            Self::Alias => "'alias'",
            Self::Function => "'function'",
            Self::Async => "'async'",
            Self::Header => "'typeDiagram'",
            Self::Ident => "a name",
            Self::Number => "a number",
            Self::LBrace => "'{'",
            Self::RBrace => "'}'",
            Self::LParen => "'('",
            Self::RParen => "')'",
            Self::LAngle => "'<'",
            Self::RAngle => "'>'",
            Self::At => "'@'",
            Self::Comma => "','",
            Self::Colon => "':'",
            Self::Equals => "'='",
            Self::Arrow => "'->'",
            Self::Newline => "a newline",
            Self::Eof => "the end of the definition",
        }
    }
}

/// One token with the position it was read from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// What it is.
    pub kind: Kind,
    /// The exact source text, so a name keeps the author's spelling.
    pub text: String,
    /// One-based line within the definition.
    pub line: usize,
    /// One-based column within the line, counted in characters.
    pub col: usize,
    /// How many characters the token spans.
    pub length: usize,
}

impl Token {
    /// How this token reads in a diagnostic: the kind, plus the spelling when
    /// the kind alone does not identify it.
    #[must_use]
    pub fn describe(&self) -> String {
        match self.kind {
            Kind::Ident | Kind::Number => format!("{} \"{}\"", self.kind.describe(), self.text),
            other => other.describe().to_owned(),
        }
    }
}

/// The keyword a bare word turns out to be, or [`Kind::Ident`].
fn keyword(word: &str) -> Kind {
    match word {
        "type" => Kind::Type,
        "union" => Kind::Union,
        "untagged" => Kind::Untagged,
        "alias" => Kind::Alias,
        "function" => Kind::Function,
        "async" => Kind::Async,
        "typeDiagram" => Kind::Header,
        _ => Kind::Ident,
    }
}

/// The single-character token `c` is, if it is one.
fn punctuation(c: char) -> Option<Kind> {
    match c {
        '{' => Some(Kind::LBrace),
        '}' => Some(Kind::RBrace),
        '(' => Some(Kind::LParen),
        ')' => Some(Kind::RParen),
        '<' => Some(Kind::LAngle),
        '>' => Some(Kind::RAngle),
        '@' => Some(Kind::At),
        ',' => Some(Kind::Comma),
        ':' => Some(Kind::Colon),
        '=' => Some(Kind::Equals),
        _ => None,
    }
}

/// Whether `c` may open an identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Whether `c` may continue one.
fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

/// The scanner's position in the definition.
struct Scanner {
    /// Every character, so lookahead is a plain index.
    chars: Vec<char>,
    /// The index of the next character to read.
    next: usize,
    /// The one-based line that index sits on.
    line: usize,
    /// The one-based column that index sits at.
    col: usize,
}

impl Scanner {
    /// A scanner over `source`, positioned at its first character.
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            next: 0,
            line: 1,
            col: 1,
        }
    }

    /// The character `ahead` positions from here, if the source has one.
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.next.saturating_add(ahead)).copied()
    }

    /// Consumes `count` characters on the current line.
    fn advance(&mut self, count: usize) {
        self.next = self.next.saturating_add(count);
        self.col = self.col.saturating_add(count);
    }

    /// Consumes a line break, wherever the next line begins.
    fn newline(&mut self, count: usize) {
        self.next = self.next.saturating_add(count);
        self.line = self.line.saturating_add(1);
        self.col = 1;
    }

    /// The run of characters starting here that all satisfy `accept`.
    fn run(&self, skip: usize, accept: fn(char) -> bool) -> String {
        self.chars
            .iter()
            .skip(self.next)
            .take(skip)
            .chain(
                self.chars
                    .iter()
                    .skip(self.next.saturating_add(skip))
                    .take_while(|c| accept(**c)),
            )
            .collect()
    }
}

/// Every token in `source`, or the first character that is not typeDiagram.
///
/// # Errors
///
/// Fails on a character the language has no meaning for, naming its position.
pub fn tokenize(source: &str) -> Result<Vec<Token>, Diagnostics> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();
    while let Some(c) = scanner.peek(0) {
        let (line, col) = (scanner.line, scanner.col);
        match c {
            ' ' | '\t' => scanner.advance(1),
            '\r' | '\n' => {
                let width =
                    usize::from(c == '\r' && scanner.peek(1) == Some('\n')).saturating_add(1);
                tokens.push(token(Kind::Newline, "\n".to_owned(), line, col));
                scanner.newline(width);
            }
            // A comment runs to the end of the line, and the line break after
            // it is still a separator [typediagram.model].
            '#' => {
                let comment = scanner.run(1, |c| c != '\n' && c != '\r');
                scanner.advance(comment.chars().count());
            }
            _ if is_ident_start(c) => {
                let word = scanner.run(1, is_ident_continue);
                scanner.advance(word.chars().count());
                tokens.push(token(keyword(&word), word, line, col));
            }
            '-' if scanner.peek(1) == Some('>') => {
                scanner.advance(2);
                tokens.push(token(Kind::Arrow, "->".to_owned(), line, col));
            }
            _ if c.is_ascii_digit()
                || (c == '-' && scanner.peek(1).is_some_and(|d| d.is_ascii_digit())) =>
            {
                let skip = usize::from(c == '-').saturating_add(1);
                let number = scanner.run(skip, |c| c.is_ascii_digit() || c == '_');
                scanner.advance(number.chars().count());
                tokens.push(token(Kind::Number, number, line, col));
            }
            _ => match punctuation(c) {
                Some(kind) => {
                    scanner.advance(1);
                    tokens.push(token(kind, c.to_string(), line, col));
                }
                None => {
                    return Err(Diagnostics::one(Diagnostic::at(
                        format!("unexpected character '{c}'"),
                        line,
                        col,
                        1,
                    )));
                }
            },
        }
    }
    tokens.push(token(Kind::Eof, String::new(), scanner.line, scanner.col));
    Ok(tokens)
}

/// One token, with its span taken from the text it was read from.
fn token(kind: Kind, text: String, line: usize, col: usize) -> Token {
    let length = text.chars().count().max(1);
    Token {
        kind,
        text,
        line,
        col,
        length,
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, tokenize};

    /// The kinds `source` tokenizes to, end marker included.
    fn kinds(source: &str) -> Vec<Kind> {
        tokenize(source)
            .expect("tokenize")
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    /// [typediagram.model]: keywords, names, and punctuation as upstream reads
    /// them.
    #[test]
    fn reads_a_record_declaration() {
        assert_eq!(
            kinds("type User { id: Uuid }"),
            [
                Kind::Type,
                Kind::Ident,
                Kind::LBrace,
                Kind::Ident,
                Kind::Colon,
                Kind::Ident,
                Kind::RBrace,
                Kind::Eof,
            ]
        );
    }

    /// [typediagram.model]: a comment is not a token, and the newline after it
    /// still separates.
    #[test]
    fn comments_disappear_but_their_line_breaks_do_not() {
        assert_eq!(
            kinds("# a comment\ntype A { }"),
            [
                Kind::Newline,
                Kind::Type,
                Kind::Ident,
                Kind::LBrace,
                Kind::RBrace,
                Kind::Eof,
            ]
        );
        assert_eq!(
            kinds("type A { x: Int # trailing\n}"),
            [
                Kind::Type,
                Kind::Ident,
                Kind::LBrace,
                Kind::Ident,
                Kind::Colon,
                Kind::Ident,
                Kind::Newline,
                Kind::RBrace,
                Kind::Eof,
            ]
        );
    }

    /// [typediagram.model]: CRLF is one line break, and the position after it
    /// is the start of the next line.
    #[test]
    fn crlf_is_a_single_newline() {
        let tokens = tokenize("type A\r\ntype B").expect("tokenize");
        assert_eq!(tokens[2].kind, Kind::Newline);
        assert_eq!(tokens[3].kind, Kind::Type);
        assert_eq!((tokens[3].line, tokens[3].col), (2, 1));
    }

    /// [typediagram.model]: discriminants may be negative and `_`-grouped.
    #[test]
    fn numbers_carry_sign_and_grouping() {
        let tokens = tokenize("= -32_700").expect("tokenize");
        assert_eq!(tokens[1].kind, Kind::Number);
        assert_eq!(tokens[1].text, "-32_700");
        assert_eq!(tokens[1].length, 7);
    }

    /// `->` is one token; a bare `-` that starts nothing is a lexical error.
    #[test]
    fn the_arrow_is_one_token_and_a_stray_dash_is_not() {
        assert_eq!(kinds("-> Bytes"), [Kind::Arrow, Kind::Ident, Kind::Eof]);
        let error = tokenize("type A - B").expect_err("a stray dash is not typeDiagram");
        assert_eq!(error.0[0].col, 8);
        assert!(error.to_string().contains("unexpected character '-'"));
    }

    /// A name that merely starts with a keyword is still a name.
    #[test]
    fn keywords_are_whole_words() {
        assert_eq!(
            kinds("typeName aliasing"),
            [Kind::Ident, Kind::Ident, Kind::Eof]
        );
        assert_eq!(
            kinds("untagged union"),
            [Kind::Untagged, Kind::Union, Kind::Eof]
        );
    }

    /// Columns count characters, so a diagnostic points at the right glyph
    /// even after non-ASCII text in a comment.
    #[test]
    fn columns_count_characters_not_bytes() {
        let error = tokenize("# héllo →\ntype A { x: Int }\n%").expect_err("stray percent");
        assert_eq!((error.0[0].line, error.0[0].col), (3, 1));
    }
}
