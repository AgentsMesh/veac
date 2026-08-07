use std::ops::Range;

use super::{CapturedSite, DomainOrigin, ExecutionDefinition};

#[derive(Debug, Clone)]
pub(crate) struct TemporalDefinitionView {
    pub(crate) identity: String,
    pub(crate) closure: bool,
    pub(crate) name: String,
    pub(crate) source: String,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct TemporalSiteView {
    pub(crate) definition: String,
    pub(crate) function: String,
    pub(crate) source: String,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct TemporalOriginView {
    pub(crate) origin: TemporalSiteView,
    pub(crate) call_stack: Vec<TemporalSiteView>,
}

impl ExecutionDefinition {
    pub(crate) fn temporal_view(&self) -> TemporalDefinitionView {
        TemporalDefinitionView {
            identity: self.identity.to_string(),
            closure: matches!(self.kind, veac_ir::AuthoredDefinitionKind::Closure),
            name: self.name.to_string(),
            source: self.origin.source_id().to_owned(),
            span: self.definition_span.clone(),
        }
    }
}

impl DomainOrigin {
    pub(crate) fn temporal_view(&self) -> TemporalOriginView {
        TemporalOriginView {
            origin: site(&self.site),
            call_stack: self.call_stack.iter().map(site).collect(),
        }
    }
}

fn site(value: &CapturedSite) -> TemporalSiteView {
    TemporalSiteView {
        definition: value.definition.to_string(),
        function: value.function.to_string(),
        source: value.source.to_string(),
        span: value.span.clone(),
    }
}
