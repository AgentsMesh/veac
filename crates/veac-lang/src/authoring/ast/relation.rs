use super::{Identifier, NumberLiteral, RecordSpan, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationDecl {
    pub id: Identifier,
    pub kind: RelationKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind {
    Transition(TransitionRelation),
    Matte(MatteRelation),
    Sidechain(SidechainRelation),
    Group(GroupRelation),
    AvLink(AvLinkRelation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEndpoint {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalEndpoint {
    Track { id: Identifier },
    Bus { id: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionRelation {
    pub endpoints: TransitionEndpoints,
    pub timing: TransitionTiming,
    pub style: TransitionStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionEndpoints {
    pub from: ItemEndpoint,
    pub to: ItemEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionTiming {
    pub duration: NumberLiteral,
    pub alignment: Spanned<TransitionAlignment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionAlignment {
    BeforeCut,
    Centered,
    AfterCut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionStyle {
    Dissolve,
    Fade {
        color: Spanned<FadeColor>,
    },
    Wipe {
        direction: Spanned<CardinalDirection>,
        angle: NumberLiteral,
        softness: NumberLiteral,
    },
    Slide {
        direction: Spanned<CardinalDirection>,
        amount: NumberLiteral,
    },
    Zoom {
        direction: Spanned<ZoomDirection>,
        amount: NumberLiteral,
    },
    Circle {
        direction: Spanned<CircleDirection>,
        softness: NumberLiteral,
    },
    Pixelize {
        amount: NumberLiteral,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeColor {
    Transparent,
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardinalDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomDirection {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleDirection {
    Open,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatteRelation {
    pub endpoints: MatteEndpoints,
    pub style: MatteStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatteEndpoints {
    pub producer: ItemEndpoint,
    pub consumer: ItemEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatteStyle {
    pub mode: Spanned<MatteMode>,
    pub invert: Spanned<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatteMode {
    Alpha,
    Luma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidechainRelation {
    pub endpoints: SidechainEndpoints,
    pub dynamics: SidechainDynamics,
    pub timing: SidechainTiming,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidechainEndpoints {
    pub key: SignalEndpoint,
    pub target: ItemEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidechainDynamics {
    pub threshold: NumberLiteral,
    pub ratio: NumberLiteral,
    pub attack: NumberLiteral,
    pub release: NumberLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidechainTiming {
    pub active: Option<RecordSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupRelation {
    pub members: Vec<ItemEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvLinkRelation {
    pub video: ItemEndpoint,
    pub audio: Vec<ItemEndpoint>,
}
