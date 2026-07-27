use crate::authoring::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::authoring) struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::authoring) enum TokenKind {
    Word(String),
    Number(String),
    String(String),
    Color(String),
    LeftBrace,
    RightBrace,
    Semicolon,
    Eof,
}
