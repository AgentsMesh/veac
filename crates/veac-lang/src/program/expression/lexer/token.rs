use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Number { number: String, unit: String },
    Text(String),
    Color(String),
    Symbol(String),
    Bool(bool),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    Comma,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}
