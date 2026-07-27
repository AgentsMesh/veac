use super::{Identifier, InterpolationDecl, NumberLiteral, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingDecl {
    Linear {
        from: NumberLiteral,
        to: NumberLiteral,
        outside: SourceOutOfRangeDecl,
        span: Span,
    },
    Curve {
        keys: Vec<MappingKey>,
        outside: SourceOutOfRangeDecl,
        span: Span,
    },
    Freeze {
        source: NumberLiteral,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceOutOfRangeDecl {
    #[default]
    Strict,
    HoldFirst,
    HoldLast,
    HoldBoth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingKey {
    pub id: Identifier,
    pub at: NumberLiteral,
    pub source: NumberLiteral,
    pub interpolation: InterpolationDecl,
    pub span: Span,
}
