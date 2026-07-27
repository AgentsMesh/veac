use super::{
    CaptionSourceDecl, GeneratorDecl, MulticamSwitchDecl, Span, TextSourceDecl, TypedReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDecl {
    Media {
        resource: TypedReference,
        span: Span,
    },
    Text {
        text: TextSourceDecl,
        span: Span,
    },
    Caption {
        caption: CaptionSourceDecl,
        span: Span,
    },
    Generated {
        generator: GeneratorDecl,
        span: Span,
    },
    Sequence {
        sequence: TypedReference,
        span: Span,
    },
    Multicam {
        group: TypedReference,
        switches: Vec<MulticamSwitchDecl>,
        span: Span,
    },
}
