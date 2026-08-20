//! The typeDiagram parser [typediagram.model].
//!
//! The published grammar is LL(1) with six productions, so this is a cursor
//! over the token stream and one function per production — no table, no
//! backtracking, no regular expression anywhere near the source.
//!
//! Unlike the upstream parser this one stops at the first error rather than
//! recovering to the next declaration. A definition that does not parse
//! generates nothing either way, and one precise position beats a cascade of
//! consequences [typediagram.diagnostics].

use super::ast::{
    Alias, Decl, Diagram, Field, Function, Record, Signature, Span, Targeting, TypeRef, Union,
    Variant,
};
use super::diagnostic::{Diagnostic, Diagnostics};
use super::lexer::{Kind, Token, tokenize};

/// Parses one typeDiagram definition.
///
/// # Errors
///
/// Fails on the first token the grammar has no production for, naming what was
/// expected and what was found.
pub fn parse(source: &str) -> Result<Diagram, Diagnostics> {
    Cursor::new(tokenize(source)?).diagram()
}

/// The token stream and the position the parser has reached in it.
struct Cursor {
    /// Every token, ending with [`Kind::Eof`].
    tokens: Vec<Token>,
    /// The index of the next token to read.
    next: usize,
}

impl Cursor {
    /// A cursor at the start of `tokens`, which always end with an EOF token.
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, next: 0 }
    }

    /// The token at the cursor. The stream always ends with EOF, so the
    /// fallback is unreachable in practice and still says the honest thing.
    fn peek(&self) -> &Token {
        self.at(self.next)
    }

    /// The token at `index`, clamped to the end marker.
    fn at(&self, index: usize) -> &Token {
        match self.tokens.get(index).or_else(|| self.tokens.last()) {
            Some(token) => token,
            None => &END,
        }
    }

    /// Consumes and returns the token at the cursor.
    fn take(&mut self) -> Token {
        let token = self.peek().clone();
        if token.kind != Kind::Eof {
            self.next = self.next.saturating_add(1);
        }
        token
    }

    /// Consumes the token at the cursor when it is `kind`.
    fn eat(&mut self, kind: Kind) -> Option<Token> {
        (self.peek().kind == kind).then(|| self.take())
    }

    /// Consumes every line break at the cursor.
    fn eat_newlines(&mut self) {
        while self.eat(Kind::Newline).is_some() {}
    }

    /// Consumes the token at the cursor, or says what was expected instead.
    fn expect(&mut self, kind: Kind) -> Result<Token, Diagnostics> {
        if self.peek().kind == kind {
            return Ok(self.take());
        }
        Err(self.unexpected(kind.describe()))
    }

    /// The diagnostic for "expected `what`, found this".
    fn unexpected(&self, what: &str) -> Diagnostics {
        let token = self.peek();
        Diagnostics::one(Diagnostic::at(
            format!("expected {what}, found {}", token.describe()),
            token.line,
            token.col,
            token.length,
        ))
    }

    /// Where the token at the cursor begins.
    fn span(&self) -> Span {
        let token = self.peek();
        Span {
            line: token.line,
            col: token.col,
            length: token.length,
        }
    }

    /// `Diagram = ("typeDiagram")? Declaration*`.
    fn diagram(&mut self) -> Result<Diagram, Diagnostics> {
        self.eat_newlines();
        if self.eat(Kind::Header).is_some() {
            self.eat_newlines();
        }
        let mut decls = Vec::new();
        loop {
            self.eat_newlines();
            if self.peek().kind == Kind::Eof {
                return Ok(Diagram { decls });
            }
            decls.push(self.declaration()?);
        }
    }

    /// `Declaration = Record | Union | Alias | Function`, with any targeting
    /// annotations that precede it.
    fn declaration(&mut self) -> Result<Decl, Diagnostics> {
        let targeting = self.targeting()?;
        let span = self.span();
        match self.peek().kind {
            Kind::Type => self.record(targeting, span).map(Decl::Record),
            Kind::Union | Kind::Untagged => self.union(targeting, span).map(Decl::Union),
            Kind::Alias => self.alias(targeting, span).map(Decl::Alias),
            Kind::Function | Kind::Async => self.function(targeting, span).map(Decl::Function),
            _ => Err(self.unexpected("'type', 'union', 'untagged union', 'alias', or 'function'")),
        }
    }

    /// `("@targets" | "@skipTargets") "(" Name ("," Name)* ")"`, repeated.
    fn targeting(&mut self) -> Result<Option<Targeting>, Diagnostics> {
        let mut targeting: Option<Targeting> = None;
        while self.eat(Kind::At).is_some() {
            let name = self.expect(Kind::Ident)?;
            let _ = self.expect(Kind::LParen)?;
            let mut values = Vec::new();
            while self.peek().kind != Kind::RParen && self.peek().kind != Kind::Eof {
                values.push(self.expect(Kind::Ident)?.text);
                if self.eat(Kind::Comma).is_none() {
                    break;
                }
                self.eat_newlines();
            }
            let _ = self.expect(Kind::RParen)?;
            let entry = targeting.get_or_insert_with(Targeting::default);
            match name.text.as_str() {
                "targets" => entry.targets = Some(values),
                "skipTargets" => entry.skip_targets = Some(values),
                other => {
                    return Err(Diagnostics::one(Diagnostic::at(
                        format!("unknown annotation '@{other}'"),
                        name.line,
                        name.col,
                        name.length,
                    )));
                }
            }
            self.eat_newlines();
        }
        Ok(targeting)
    }

    /// `Record = "type" Name Generics? "{" Field* "}"`.
    fn record(&mut self, targeting: Option<Targeting>, span: Span) -> Result<Record, Diagnostics> {
        let _ = self.take();
        let name = self.expect(Kind::Ident)?.text;
        let generics = self.generic_params()?;
        let _ = self.expect(Kind::LBrace)?;
        let fields = self.brace_list(Self::field)?;
        let _ = self.expect(Kind::RBrace)?;
        Ok(Record {
            name,
            generics,
            fields,
            targeting,
            span,
        })
    }

    /// `Union = "untagged"? "union" Name Generics? "{" Variant* "}"`.
    fn union(&mut self, targeting: Option<Targeting>, span: Span) -> Result<Union, Diagnostics> {
        let untagged = self.eat(Kind::Untagged).is_some();
        let _ = self.expect(Kind::Union)?;
        let name = self.expect(Kind::Ident)?.text;
        let generics = self.generic_params()?;
        let _ = self.expect(Kind::LBrace)?;
        let variants = self.brace_list(Self::variant)?;
        let _ = self.expect(Kind::RBrace)?;
        Ok(Union {
            name,
            generics,
            untagged,
            variants,
            targeting,
            span,
        })
    }

    /// `Alias = "alias" Name Generics? "=" TypeRef`.
    fn alias(&mut self, targeting: Option<Targeting>, span: Span) -> Result<Alias, Diagnostics> {
        let _ = self.take();
        let name = self.expect(Kind::Ident)?.text;
        let generics = self.generic_params()?;
        let _ = self.expect(Kind::Equals)?;
        let target = self.type_ref()?;
        Ok(Alias {
            name,
            generics,
            target,
            targeting,
            span,
        })
    }

    /// `Function = "async"? "function" Name Generics? (Signature | "{" Signature* "}")`.
    ///
    /// A head `async` describes the *bare* form's one signature. An overload
    /// block spells `async` per signature, and upstream discards the head's
    /// flag there — matched exactly, because the model JSON must agree
    /// [typediagram.delivery.baseline].
    fn function(
        &mut self,
        targeting: Option<Targeting>,
        span: Span,
    ) -> Result<Function, Diagnostics> {
        let is_async = self.eat(Kind::Async).is_some();
        let _ = self.expect(Kind::Function)?;
        let name = self.expect(Kind::Ident)?.text;
        let generics = self.generic_params()?;
        let signatures = match self.eat(Kind::LBrace) {
            Some(_) => {
                let signatures = self.brace_list(Self::overload)?;
                let _ = self.expect(Kind::RBrace)?;
                signatures
            }
            None => vec![self.signature(is_async)?],
        };
        Ok(Function {
            name,
            generics,
            signatures,
            targeting,
            span,
        })
    }

    /// One signature inside an overload block, with its own `async` flag.
    fn overload(&mut self) -> Result<Signature, Diagnostics> {
        let is_async = self.eat(Kind::Async).is_some();
        self.signature(is_async)
    }

    /// `Signature = "(" Parameter* ")" "->" TypeRef`.
    fn signature(&mut self, is_async: bool) -> Result<Signature, Diagnostics> {
        let span = self.span();
        let _ = self.expect(Kind::LParen)?;
        let mut params = Vec::new();
        while self.peek().kind != Kind::RParen && self.peek().kind != Kind::Eof {
            params.push(self.field()?);
            if self.eat(Kind::Comma).is_none() {
                break;
            }
            self.eat_newlines();
        }
        let _ = self.expect(Kind::RParen)?;
        let _ = self.expect(Kind::Arrow)?;
        let returns = self.type_ref()?;
        Ok(Signature {
            params,
            returns,
            is_async,
            span,
        })
    }

    /// `Field = Name ":" TypeRef`, which is also a parameter.
    fn field(&mut self) -> Result<Field, Diagnostics> {
        let span = self.span();
        let name = self.expect(Kind::Ident)?.text;
        let _ = self.expect(Kind::Colon)?;
        let ty = self.type_ref()?;
        Ok(Field { name, ty, span })
    }

    /// `Variant = Name ("=" Number)? ("{" Field* "}" | "(" TypeRef* ")")?`.
    fn variant(&mut self) -> Result<Variant, Diagnostics> {
        let span = self.span();
        let name = self.expect(Kind::Ident)?.text;
        let discriminant = match self.eat(Kind::Equals) {
            Some(_) => Some(self.expect(Kind::Number)?.text),
            None => None,
        };
        let fields = match self.peek().kind {
            Kind::LBrace => {
                let _ = self.take();
                let fields = self.brace_list(Self::field)?;
                let _ = self.expect(Kind::RBrace)?;
                fields
            }
            Kind::LParen => {
                let _ = self.take();
                let fields = self.tuple_fields()?;
                let _ = self.expect(Kind::RParen)?;
                fields
            }
            _ => Vec::new(),
        };
        Ok(Variant {
            name,
            discriminant,
            fields,
            span,
        })
    }

    /// The positional payload of a tuple variant, named `_0`, `_1`, … exactly
    /// as upstream names it.
    fn tuple_fields(&mut self) -> Result<Vec<Field>, Diagnostics> {
        let mut fields: Vec<Field> = Vec::new();
        loop {
            self.eat_newlines();
            if matches!(self.peek().kind, Kind::RParen | Kind::Eof) {
                return Ok(fields);
            }
            let ty = self.type_ref()?;
            fields.push(Field {
                name: format!("_{}", fields.len()),
                span: ty.span,
                ty,
            });
            self.eat_newlines();
            if self.eat(Kind::Comma).is_none() {
                return Ok(fields);
            }
        }
    }

    /// Items inside `{ … }`, separated by a comma, a line break, or both.
    fn brace_list<T>(
        &mut self,
        item: fn(&mut Self) -> Result<T, Diagnostics>,
    ) -> Result<Vec<T>, Diagnostics> {
        let mut items = Vec::new();
        loop {
            self.eat_newlines();
            if matches!(self.peek().kind, Kind::RBrace | Kind::Eof) {
                return Ok(items);
            }
            items.push(item(self)?);
            let _ = self.eat(Kind::Comma);
            self.eat_newlines();
        }
    }

    /// `Generics = "<" Name ("," Name)* ">"`, absent when there is no `<`.
    fn generic_params(&mut self) -> Result<Vec<String>, Diagnostics> {
        if self.eat(Kind::LAngle).is_none() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        while self.peek().kind != Kind::RAngle && self.peek().kind != Kind::Eof {
            names.push(self.expect(Kind::Ident)?.text);
            if self.eat(Kind::Comma).is_none() {
                break;
            }
            self.eat_newlines();
        }
        let _ = self.expect(Kind::RAngle)?;
        Ok(names)
    }

    /// `TypeRef = Name ("<" TypeRef ("," TypeRef)* ">")?`.
    fn type_ref(&mut self) -> Result<TypeRef, Diagnostics> {
        let span = self.span();
        let name = self.expect(Kind::Ident)?.text;
        let mut args = Vec::new();
        if self.eat(Kind::LAngle).is_some() {
            while self.peek().kind != Kind::RAngle && self.peek().kind != Kind::Eof {
                args.push(self.type_ref()?);
                if self.eat(Kind::Comma).is_none() {
                    break;
                }
                self.eat_newlines();
            }
            let _ = self.expect(Kind::RAngle)?;
        }
        Ok(TypeRef { name, args, span })
    }
}

/// The token a cursor reports when its stream is empty, which construction
/// prevents — [`tokenize`] always appends an end marker.
static END: Token = Token {
    kind: Kind::Eof,
    text: String::new(),
    line: 1,
    col: 1,
    length: 1,
};

// A separate file only because parser.rs is at the 500-line ceiling.
#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
