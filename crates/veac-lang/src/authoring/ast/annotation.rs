use super::{AnnotationPayloadDecl, Identifier, NumberLiteral, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationDecl {
    pub id: Identifier,
    pub target: AnnotationTargetDecl,
    pub timing: AnnotationTimingDecl,
    pub payload: AnnotationPayloadDecl,
    pub provenance: AnnotationProvenanceDecl,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationTargetDecl {
    Project { span: Span },
    Sequence(Identifier),
    Layer(Identifier),
    Item(Identifier),
    Resource(Identifier),
    Multicam(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationTimingDecl {
    Untimed {
        span: Span,
    },
    Point {
        at: NumberLiteral,
        span: Span,
    },
    Range {
        at: NumberLiteral,
        duration: NumberLiteral,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationProvenanceDecl {
    pub producer: Spanned<String>,
    pub request_sha256: Spanned<String>,
    pub response_sha256: Spanned<String>,
    pub span: Span,
}

impl AnnotationTargetDecl {
    pub fn span(&self) -> Span {
        match self {
            Self::Project { span } => *span,
            Self::Sequence(value)
            | Self::Layer(value)
            | Self::Item(value)
            | Self::Resource(value)
            | Self::Multicam(value) => value.span,
        }
    }
}

impl AnnotationTimingDecl {
    pub fn span(&self) -> Span {
        match self {
            Self::Untimed { span } | Self::Point { span, .. } | Self::Range { span, .. } => *span,
        }
    }
}
