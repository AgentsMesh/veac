use super::{NumberLiteral, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSlotDecl {
    Media {
        accepts: Spanned<TemplateMediaKindDecl>,
        fill: Spanned<TemplateFillDecl>,
        label: Spanned<String>,
        minimum_source_duration: Option<NumberLiteral>,
        span: Span,
    },
    Text {
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateMediaKindDecl {
    Video,
    Image,
    VideoOrImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateFillDecl {
    FitDuration,
    TakeHead,
    TakeCenter,
}
