use crate::authoring::Span;

use super::FunctionBodyBinding;

mod property;
pub(crate) use property::TemporalProperty;
mod target;
pub(crate) use target::{TemporalApplyPath, TemporalItemPath, TemporalTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalResourcePath {
    pub project: String,
    pub resource: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TemporalDecl {
    pub property: TemporalProperty,
    pub target: TemporalTarget,
    pub source: Option<TemporalResourcePath>,
    pub body: FunctionBodyBinding,
    pub span: Span,
}

impl TemporalResourcePath {
    pub(crate) fn segments(&self) -> [&str; 2] {
        [&self.project, &self.resource]
    }
}
