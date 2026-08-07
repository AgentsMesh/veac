use std::ops::Range;
use std::sync::Arc;

use veac_ir::{
    AuthoredDefinition, AuthoredDefinitionId, AuthoredDefinitionKind, AuthoredFunctionName,
    AuthoredSite, AuthoredSourceId, AuthoredSpan, AuthorshipEvent, AuthorshipEventKind,
    DomainOpcode,
};

use super::{
    CompiledFunction, CoreDigest, ExpressionLoopFrame, FunctionOrigin, MAX_FUNCTION_CALL_DEPTH,
};
use crate::program::DomainOperationId;

mod identity;
mod iteration;
mod temporal;
pub(crate) use identity::json_bytes;
pub(crate) use temporal::{TemporalDefinitionView, TemporalSiteView};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionDefinition {
    kind: AuthoredDefinitionKind,
    identity: Arc<str>,
    name: Arc<str>,
    origin: FunctionOrigin,
    definition_span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionFrame {
    definition: ExecutionDefinition,
    call_site: Option<CapturedSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedSite {
    definition: Arc<str>,
    function: Arc<str>,
    source: Arc<str>,
    span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainOrigin {
    site: CapturedSite,
    definition: ExecutionDefinition,
    call_stack: Arc<[CapturedSite]>,
    iterations: Arc<[ExpressionLoopFrame]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProgramIdentity {
    pub(crate) core_version: u16,
    pub(crate) domain_opset: u16,
    pub(crate) domain_registry_sha256: String,
    pub(crate) main_content_sha256: String,
    pub(crate) source_graph_sha256: String,
    pub(crate) declared_inputs_sha256: String,
}

impl ExecutionDefinition {
    pub(crate) fn function(value: &CompiledFunction) -> Option<Self> {
        let origin = value.origin()?.clone();
        Some(Self {
            kind: AuthoredDefinitionKind::Function,
            identity: value.id().to_string().into(),
            name: value.name().into(),
            definition_span: origin.body_span(),
            origin,
        })
    }

    pub(crate) fn closure(&self, digest: CoreDigest, span: Range<usize>) -> Self {
        Self {
            kind: AuthoredDefinitionKind::Closure,
            identity: digest.to_string().into(),
            name: "<closure>".into(),
            origin: self.origin.clone(),
            definition_span: self.origin.absolute_span(span),
        }
    }

    fn site(&self, span: Range<usize>) -> CapturedSite {
        CapturedSite {
            definition: Arc::clone(&self.identity),
            function: Arc::clone(&self.name),
            source: self.origin.source_id().into(),
            span: self.origin.absolute_span(span),
        }
    }

    fn authored(&self) -> AuthoredDefinition {
        AuthoredDefinition {
            identity: AuthoredDefinitionId::new(self.identity.as_ref()),
            kind: self.kind,
            name: AuthoredFunctionName::new(self.name.as_ref()),
            source: AuthoredSourceId::new(self.origin.source_id()),
            span: authored_span(&self.definition_span),
        }
    }
}

impl ExecutionFrame {
    pub(crate) fn new(definition: ExecutionDefinition, call_site: Option<DomainOrigin>) -> Self {
        Self {
            definition,
            call_site: call_site.map(|origin| origin.site),
        }
    }

    pub(crate) fn definition(&self) -> &ExecutionDefinition {
        &self.definition
    }
}

impl DomainOrigin {
    pub(crate) fn capture(
        frames: &[ExecutionFrame],
        iterations: &[ExpressionLoopFrame],
        span: Range<usize>,
    ) -> Option<Self> {
        let current = frames.last()?;
        let call_stack = frames
            .iter()
            .filter_map(|frame| frame.call_site.clone())
            .take(MAX_FUNCTION_CALL_DEPTH)
            .collect::<Vec<_>>();
        Some(Self {
            site: current.definition.site(span),
            definition: current.definition.clone(),
            call_stack: call_stack.into(),
            iterations: iterations.into(),
        })
    }

    pub(crate) fn event(
        &self,
        operation: DomainOperationId,
        kind: AuthorshipEventKind,
    ) -> AuthorshipEvent {
        AuthorshipEvent {
            call_stack: self.call_stack.iter().map(CapturedSite::authored).collect(),
            definition: self.definition.authored(),
            kind,
            iterations: self.iterations.iter().map(iteration::authored).collect(),
            operation: DomainOpcode(operation.opcode()),
            origin: self.site.authored(),
        }
    }
}

impl CapturedSite {
    fn authored(&self) -> AuthoredSite {
        AuthoredSite {
            definition: AuthoredDefinitionId::new(self.definition.as_ref()),
            function: AuthoredFunctionName::new(self.function.as_ref()),
            source: AuthoredSourceId::new(self.source.as_ref()),
            span: authored_span(&self.span),
        }
    }
}

fn authored_span(span: &Range<usize>) -> AuthoredSpan {
    AuthoredSpan {
        start: span.start as u64,
        end: span.end as u64,
    }
}
