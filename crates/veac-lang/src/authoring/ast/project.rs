use super::{
    AnnotationDecl, Identifier, MulticamDecl, NumberLiteral, OutputDecl, SequenceDecl, Spanned,
    TypedReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub project: ProjectDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDecl {
    pub id: Identifier,
    pub entry: TypedReference,
    pub settings: ProjectSettings,
    pub resources: Vec<ResourceDecl>,
    pub multicams: Vec<MulticamDecl>,
    pub sequences: Vec<SequenceDecl>,
    pub outputs: Vec<OutputDecl>,
    pub annotations: Vec<AnnotationDecl>,
    pub span: super::Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectSettings {
    pub timebase: Option<NumberLiteral>,
    pub canvas: Option<(NumberLiteral, NumberLiteral)>,
    pub frame_rate: Option<NumberLiteral>,
    pub sample_rate: Option<NumberLiteral>,
    pub span: super::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Video,
    Audio,
    Image,
    Font,
    Lut1d,
    Lut3d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDecl {
    pub kind: ResourceKind,
    pub id: Identifier,
    pub locator: ResourceLocator,
    pub streams: Option<ResourceStreams>,
    pub span: super::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceLocator {
    Local {
        path: Spanned<String>,
    },
    Remote {
        uri: Spanned<String>,
        identity: ResourceIdentity,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceIdentity {
    Sha256(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceStreams {
    pub video: StreamSelection,
    pub audio: StreamSelection,
    pub span: super::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamSelection {
    Auto,
    Disabled,
}
