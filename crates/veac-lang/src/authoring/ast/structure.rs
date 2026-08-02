use super::{
    ColorModifierDecl, EffectModifierDecl, Identifier, MaskModifierDecl, NumberLiteral,
    ParameterDecl, RecordSpan, RelationDecl, Span,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureDecl {
    Relation(RelationDecl),
    Apply(ApplyDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyDecl {
    pub id: Identifier,
    pub scope: ApplyScope,
    pub record: RecordSpan,
    pub pipeline: Vec<ApplyStageDecl>,
    pub mix: ApplyMixDecl,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyScope {
    CompositeBand {
        from: Identifier,
        through: Identifier,
        span: Span,
    },
    Layer {
        layer: Identifier,
        span: Span,
    },
    Items {
        targets: Vec<ApplyItemTarget>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyItemTarget {
    Item(Identifier),
    Group(Identifier),
}

impl ApplyItemTarget {
    pub fn id(&self) -> &Identifier {
        match self {
            Self::Item(value) | Self::Group(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyStageDecl {
    Color(ColorModifierDecl),
    Effect(EffectModifierDecl),
}

impl ApplyStageDecl {
    pub fn id(&self) -> &Identifier {
        match self {
            Self::Color(value) => &value.id,
            Self::Effect(value) => &value.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyMixDecl {
    pub opacity: Option<ParameterDecl<NumberLiteral>>,
    pub blend: Option<Identifier>,
    pub masks: Vec<MaskModifierDecl>,
    pub span: Span,
}
