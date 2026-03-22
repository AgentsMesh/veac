use std::fmt;

/// Source location for error reporting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }
}

/// All token types in the VEAC language.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Project,
    Asset,
    Timeline,
    Track,
    Clip,
    Text,
    Transition,
    Let,
    Include,

    // Asset type functions
    Video,
    Audio,
    Image,

    // Identifiers and literals
    Ident(String),
    StringLit(String),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),

    // Time literals: value stored, suffix determines kind
    TimeSec(f64),    // e.g. 3.5s
    TimeMs(f64),     // e.g. 500ms
    TimeFrames(u64), // e.g. 84f
    Smpte(String),   // e.g. "00:01:30:12" (parsed from string)

    // Color literal
    ColorLit(String), // e.g. #FFFFFF or #FF000080

    // Punctuation
    LBrace, // {
    RBrace, // }
    LParen, // (
    RParen, // )
    Equals, // =
    Comma,  // ,
    Dot,    // .

    // Special
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Project => write!(f, "project"),
            TokenKind::Asset => write!(f, "asset"),
            TokenKind::Timeline => write!(f, "timeline"),
            TokenKind::Track => write!(f, "track"),
            TokenKind::Clip => write!(f, "clip"),
            TokenKind::Text => write!(f, "text"),
            TokenKind::Transition => write!(f, "transition"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Include => write!(f, "include"),
            TokenKind::Video => write!(f, "video"),
            TokenKind::Audio => write!(f, "audio"),
            TokenKind::Image => write!(f, "image"),
            TokenKind::Ident(s) => write!(f, "identifier `{s}`"),
            TokenKind::StringLit(s) => write!(f, "string \"{s}\""),
            TokenKind::IntLit(n) => write!(f, "integer {n}"),
            TokenKind::FloatLit(n) => write!(f, "float {n}"),
            TokenKind::BoolLit(b) => write!(f, "bool {b}"),
            TokenKind::TimeSec(v) => write!(f, "{v}s"),
            TokenKind::TimeMs(v) => write!(f, "{v}ms"),
            TokenKind::TimeFrames(n) => write!(f, "{n}f"),
            TokenKind::Smpte(s) => write!(f, "SMPTE \"{s}\""),
            TokenKind::ColorLit(c) => write!(f, "color #{c}"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Eof => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
