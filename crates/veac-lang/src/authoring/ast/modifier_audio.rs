use super::{AudioProcessorDecl, Identifier, NumberLiteral, ParameterDecl, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioModifierDecl {
    pub id: Identifier,
    pub gain: Option<ParameterDecl<NumberLiteral>>,
    pub pan: Option<ParameterDecl<NumberLiteral>>,
    pub muted: Option<Spanned<bool>>,
    pub normalize: Option<Spanned<bool>>,
    pub pitch: Option<Spanned<PitchPolicyDecl>>,
    pub processors: Vec<AudioProcessorDecl>,
    pub crossfade: Option<AudioCrossfadeDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchPolicyDecl {
    Preserve,
    FollowSpeed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCrossfadeDecl {
    pub fade_in: NumberLiteral,
    pub fade_out: NumberLiteral,
    pub curve: Spanned<AudioFadeCurveDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFadeCurveDecl {
    Linear,
    EqualPower,
    Exponential,
}
