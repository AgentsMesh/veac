use super::{
    Identifier, MappingDecl, ModifierDecl, NumberLiteral, SourceDecl, Span, StructureDecl,
    TemplateSlotDecl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceDecl {
    pub id: Identifier,
    pub layers: Vec<LayerDecl>,
    pub structures: Vec<StructureDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Video,
    Audio,
    Visual,
    Caption,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerDecl {
    pub kind: LayerKind,
    pub id: Identifier,
    pub order: Option<NumberLiteral>,
    pub route_bus: Option<Identifier>,
    pub items: Vec<ItemDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDecl {
    pub id: Identifier,
    pub source: SourceDecl,
    pub record: RecordSpan,
    pub mapping: Option<MappingDecl>,
    pub template_slot: Option<TemplateSlotDecl>,
    pub modifiers: Vec<ModifierDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSpan {
    pub at: NumberLiteral,
    pub duration: NumberLiteral,
    pub span: Span,
}
