use crate::source_edit::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedExpression {
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedStatement {
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedBody {
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedDeclaration {
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedImport {
    pub path: String,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedTopLevelDeclaration {
    pub source: String,
    pub range: TextRange,
}
