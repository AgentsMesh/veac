use super::{
    AudioModifierDecl, ColorModifierDecl, Identifier, MaskModifierDecl, NumberLiteral,
    ParameterDecl, PointDecl, RectDecl, Span, Spanned, SurfaceModifierDecl, VectorDecl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierDecl {
    Layout(LayoutModifierDecl),
    Transform(TransformModifierDecl),
    Composite(CompositeModifierDecl),
    Surface(SurfaceModifierDecl),
    Mask(MaskModifierDecl),
    Audio(AudioModifierDecl),
    Color(ColorModifierDecl),
    Effect(EffectModifierDecl),
}

impl ModifierDecl {
    pub fn id(&self) -> &Identifier {
        match self {
            Self::Layout(value) => &value.id,
            Self::Transform(value) => &value.id,
            Self::Composite(value) => &value.id,
            Self::Surface(value) => &value.id,
            Self::Mask(value) => &value.id,
            Self::Audio(value) => &value.id,
            Self::Color(value) => &value.id,
            Self::Effect(value) => &value.id,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Layout(value) => value.span,
            Self::Transform(value) => value.span,
            Self::Composite(value) => value.span,
            Self::Surface(value) => value.span,
            Self::Mask(value) => value.span,
            Self::Audio(value) => value.span,
            Self::Color(value) => value.span,
            Self::Effect(value) => value.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutModifierDecl {
    pub id: Identifier,
    pub placement: Option<PlacementDecl>,
    pub frame: Option<FrameDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementDecl {
    Anchor {
        anchor: Identifier,
        inset: VectorDecl,
        span: Span,
    },
    Absolute {
        position: PointDecl,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameDecl {
    pub width: NumberLiteral,
    pub height: NumberLiteral,
    pub fit: Identifier,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformModifierDecl {
    pub id: Identifier,
    pub position: Option<ParameterDecl<PointDecl>>,
    pub scale: Option<ParameterDecl<VectorDecl>>,
    pub shear: Option<Box<VectorDecl>>,
    pub rotation: Option<ParameterDecl<NumberLiteral>>,
    pub anchor: Option<VectorDecl>,
    pub crop: Option<ParameterDecl<RectDecl>>,
    pub flip_horizontal: Option<Spanned<bool>>,
    pub flip_vertical: Option<Spanned<bool>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeModifierDecl {
    pub id: Identifier,
    pub opacity: Option<ParameterDecl<NumberLiteral>>,
    pub z_index: Option<NumberLiteral>,
    pub blend: Option<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectModifierDecl {
    pub id: Identifier,
    pub effect_type: Identifier,
    pub enabled: Option<Spanned<bool>>,
    pub record: Option<super::RecordSpan>,
    pub parameters: Vec<EffectParameterDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectParameterDecl {
    pub name: Identifier,
    pub value: EffectParameterValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectParameterValue {
    Number(ParameterDecl<NumberLiteral>),
    Boolean(Spanned<bool>),
    Color(Spanned<String>),
}

impl EffectParameterValue {
    pub fn kind(&self) -> veac_ir::ParameterValueKind {
        match self {
            Self::Number(ParameterDecl::Constant(_)) => veac_ir::ParameterValueKind::Number,
            Self::Number(ParameterDecl::Curve { .. }) => veac_ir::ParameterValueKind::NumberCurve,
            Self::Boolean(_) => veac_ir::ParameterValueKind::Boolean,
            Self::Color(_) => veac_ir::ParameterValueKind::Color,
        }
    }
}
