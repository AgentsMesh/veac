use super::{
    Identifier, ItemStateDecl, MappingDecl, ModifierDecl, NumberLiteral, PlacementModeDecl,
    SourceDecl, Span, Spanned, StructureDecl, TemplateSlotDecl, TrackStateDecl,
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
    pub placement: Option<Spanned<PlacementModeDecl>>,
    pub state: Option<TrackStateDecl>,
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
    pub state: Option<ItemStateDecl>,
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
