//! Source-anchored diagnostics for the typeDiagram front end
//! [typediagram.diagnostics].
//!
//! A definition lives inside a Markdown fence, so a bare message is useless:
//! the author needs the line and column *inside the fence* and, at the
//! boundary, the document line the fence starts on. Everything here carries
//! the position; [`Diagnostic::in_document`] is what turns fence-relative
//! positions into document-relative ones once the binder knows where the fence
//! began.

use std::fmt;

/// One problem in a typeDiagram definition, anchored in its own source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// What went wrong, in the author's terms.
    pub message: String,
    /// One-based line, relative to the text the diagnostic was produced from.
    pub line: usize,
    /// One-based column within that line.
    pub col: usize,
    /// How many characters the offending token spans; at least one.
    pub length: usize,
}

impl Diagnostic {
    /// A diagnostic at `line`/`col` spanning `length` characters.
    #[must_use]
    pub fn at(message: impl Into<String>, line: usize, col: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            line,
            col,
            length: length.max(1),
        }
    }

    /// The same diagnostic with its line rebased onto the enclosing document.
    ///
    /// `fence_line` is the one-based document line of the fence's *opening*
    /// marker, so the definition's own first line is the one after it.
    #[must_use]
    pub fn in_document(&self, fence_line: usize) -> Self {
        Self {
            message: self.message.clone(),
            line: fence_line.saturating_add(self.line),
            col: self.col,
            length: self.length,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.col, self.message
        )
    }
}

/// Every diagnostic one parse or validation produced, in source order.
///
/// This is a newtype rather than a bare `Vec` so that the whole set formats as
/// one block: a definition with three unknown type names is one failure with
/// three lines, not three failures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    /// A set holding exactly one diagnostic.
    #[must_use]
    pub fn one(diagnostic: Diagnostic) -> Self {
        Self(vec![diagnostic])
    }

    /// Whether anything was reported.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The same set rebased onto the enclosing document [`Diagnostic::in_document`].
    #[must_use]
    pub fn in_document(&self, fence_line: usize) -> Self {
        Self(self.0.iter().map(|d| d.in_document(fence_line)).collect())
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for diagnostic in &self.0 {
            if !first {
                writeln!(f)?;
            }
            first = false;
            write!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Diagnostics};

    /// [typediagram.diagnostics]: a fence-relative position becomes a document
    /// position, and the fence's own opening line is not part of the source.
    #[test]
    fn rebasing_counts_from_the_line_after_the_fence_marker() {
        let inner = Diagnostic::at("unknown type 'Timestamp'", 3, 9, 9);
        let outer = inner.in_document(12);
        assert_eq!((outer.line, outer.col, outer.length), (15, 9, 9));
        assert_eq!(outer.message, inner.message);
    }

    /// A zero-length token still underlines one character.
    #[test]
    fn a_span_is_never_empty() {
        assert_eq!(Diagnostic::at("end of input", 1, 1, 0).length, 1);
    }

    /// [typediagram.diagnostics]: several problems format as one block.
    #[test]
    fn a_set_formats_one_line_per_diagnostic() {
        let set = Diagnostics(vec![
            Diagnostic::at("first", 1, 2, 3),
            Diagnostic::at("second", 4, 5, 6),
        ]);
        assert_eq!(
            set.to_string(),
            "line 1, column 2: first\nline 4, column 5: second"
        );
        assert!(!set.is_empty());
        assert!(Diagnostics::default().is_empty());
        assert_eq!(Diagnostics::one(Diagnostic::at("only", 1, 1, 1)).0.len(), 1);
    }
}
