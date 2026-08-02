use super::{NumberLiteral, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioProcessorDecl {
    ParametricEq { bands: Vec<EqBandDecl>, span: Span },
    HighPass(FilterDecl),
    LowPass(FilterDecl),
    Compressor(CompressorDecl),
    Limiter(LimiterDecl),
    Gate(GateDecl),
    Loudness(LoudnessDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqBandDecl {
    pub frequency: NumberLiteral,
    pub gain: NumberLiteral,
    pub q: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterDecl {
    pub frequency: NumberLiteral,
    pub q: NumberLiteral,
    pub poles: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressorDecl {
    pub threshold: NumberLiteral,
    pub ratio: NumberLiteral,
    pub attack: NumberLiteral,
    pub release: NumberLiteral,
    pub knee: NumberLiteral,
    pub makeup_gain: NumberLiteral,
    pub mix: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimiterDecl {
    pub ceiling: NumberLiteral,
    pub attack: NumberLiteral,
    pub release: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateDecl {
    pub threshold: NumberLiteral,
    pub ratio: NumberLiteral,
    pub attack: NumberLiteral,
    pub release: NumberLiteral,
    pub range: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoudnessDecl {
    pub integrated: NumberLiteral,
    pub true_peak: NumberLiteral,
    pub range: NumberLiteral,
    pub span: Span,
}
