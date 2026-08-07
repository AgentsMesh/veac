use crate::authoring::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Word(String),
    Number(String),
    String(String),
    Color(String),
    LocalId(String),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Arrow,
    DotDot,
    Semicolon,
    Comma,
    Equals,
    EqualsEquals,
    Bang,
    BangEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
    AndAnd,
    OrOr,
    Plus,
    Minus,
    Star,
    Slash,
    DollarLeftBrace,
    Eof,
}

impl Token {
    pub(crate) fn word(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Word(value) => Some(value),
            _ => None,
        }
    }
}

pub(crate) fn is_word_start(value: char) -> bool {
    value.is_alphabetic() || value == '_'
}

pub(crate) fn is_word_continue(value: char) -> bool {
    value.is_alphanumeric() || matches!(value, '_' | '-' | '.')
}
