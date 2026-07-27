use super::{NumberLiteral, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationPayloadDecl {
    Marker(MarkerPayloadDecl),
    Language(LanguagePayloadDecl),
    SceneBoundary(SceneBoundaryPayloadDecl),
    Scene,
    Beat(BeatPayloadDecl),
    Silence(SilencePayloadDecl),
    Filler(FillerPayloadDecl),
    Highlight(HighlightPayloadDecl),
    Review(ReviewPayloadDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerPayloadDecl {
    pub label: Spanned<String>,
    pub color: Option<Spanned<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguagePayloadDecl {
    pub candidates: Vec<LanguageCandidateDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCandidateDecl {
    pub language: Spanned<String>,
    pub confidence: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneBoundaryPayloadDecl {
    pub confidence: NumberLiteral,
    pub hard_cut: Spanned<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeatPayloadDecl {
    pub confidence: NumberLiteral,
    pub bar: NumberLiteral,
    pub beat_in_bar: NumberLiteral,
    pub tempo_bpm: NumberLiteral,
    pub meter: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilencePayloadDecl {
    pub mean_db: NumberLiteral,
    pub confidence: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillerPayloadDecl {
    pub token: Spanned<String>,
    pub confidence: NumberLiteral,
    pub suggestion: Spanned<FillerSuggestionDecl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillerSuggestionDecl {
    Keep,
    Delete,
    Tighten,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightPayloadDecl {
    pub score: NumberLiteral,
    pub rationale: Spanned<String>,
    pub evidence: Vec<Spanned<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewPayloadDecl {
    pub action: Spanned<ReviewActionDecl>,
    pub rationale: Spanned<String>,
    pub confidence: NumberLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewActionDecl {
    Keep,
    Remove,
}
