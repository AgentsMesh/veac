use std::ops::Range;

use super::{Expression, PathSegment};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NominalField {
    pub name: String,
    pub name_span: Range<usize>,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatternField {
    pub field: String,
    pub field_span: Range<usize>,
    pub binding: String,
    pub binding_span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchPattern {
    Variant {
        path: Vec<PathSegment>,
        fields: Vec<PatternField>,
    },
    Wildcard {
        span: Range<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Expression,
    pub span: Range<usize>,
}
