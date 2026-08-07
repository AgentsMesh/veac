use std::ops::Range;
use std::sync::Arc;

use veac_ir::{
    AuthoredDefinitionId, AuthoredFunctionName, AuthoredIteration, AuthoredSourceId, AuthoredSpan,
    LoopLogicalKey,
};

use super::{ExecutionDefinition, ExecutionFrame};
use crate::program::expression::ExpressionLoopFrame;

pub(super) fn frame(
    definition: &ExecutionDefinition,
    loop_span: Range<usize>,
    binding_span: Range<usize>,
    index: usize,
) -> ExpressionLoopFrame {
    ExpressionLoopFrame::new(
        Arc::clone(&definition.identity),
        Arc::clone(&definition.name),
        definition.origin.source_id().into(),
        definition.origin.absolute_span(loop_span),
        definition.origin.absolute_span(binding_span),
        index,
    )
}

pub(super) fn authored(frame: &ExpressionLoopFrame) -> AuthoredIteration {
    AuthoredIteration {
        binding_span: span(frame.binding_span()),
        definition: AuthoredDefinitionId::new(frame.definition()),
        function: AuthoredFunctionName::new(frame.function()),
        index: frame.index() as u64,
        logical_key: LoopLogicalKey::new(frame.logical_key().as_str()),
        loop_span: span(frame.loop_span()),
        source: AuthoredSourceId::new(frame.source()),
    }
}

fn span(value: Range<usize>) -> AuthoredSpan {
    AuthoredSpan {
        start: value.start as u64,
        end: value.end as u64,
    }
}

impl ExecutionFrame {
    pub(crate) fn iteration(
        &self,
        loop_span: Range<usize>,
        binding_span: Range<usize>,
        index: usize,
    ) -> ExpressionLoopFrame {
        frame(&self.definition, loop_span, binding_span, index)
    }
}
