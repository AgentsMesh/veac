use super::{Identifier, NumberLiteral, Span, TypedReference};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticamDecl {
    pub id: Identifier,
    pub sync: MulticamSyncDecl,
    pub angles: Vec<MulticamAngleDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticamSyncDecl {
    pub basis: MulticamSyncBasisDecl,
    pub reference: TypedReference,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulticamSyncBasisDecl {
    Timecode,
    Audio,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticamAngleDecl {
    pub id: Identifier,
    pub resource: TypedReference,
    pub source_offset: NumberLiteral,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulticamSwitchDecl {
    pub angle: TypedReference,
    pub at: NumberLiteral,
    pub duration: NumberLiteral,
    pub span: Span,
}
