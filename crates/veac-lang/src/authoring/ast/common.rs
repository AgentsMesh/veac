#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

pub type Identifier = Spanned<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberLiteral {
    pub raw: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedReference {
    pub kind: Identifier,
    pub id: Identifier,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticValue {
    String(Spanned<String>),
    Color(Spanned<String>),
    Number(NumberLiteral),
    Boolean(Spanned<bool>),
    Identifier(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticBlock {
    pub entries: Vec<SemanticEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEntry {
    pub name: Identifier,
    pub values: Vec<SemanticValue>,
    pub block: Option<SemanticBlock>,
    pub span: Span,
}
