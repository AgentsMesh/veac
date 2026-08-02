use crate::source_edit::{budget, SourceEditBatch, SourceEditOperation, SourcePrecondition};

pub(super) fn validate(batch: &SourceEditBatch) -> Result<(), super::SourceEditError> {
    let preconditions = batch.preconditions.iter().filter_map(|value| match value {
        SourcePrecondition::ExpressionEquals { expression, .. } => Some(expression.source.len()),
        SourcePrecondition::NodeExists { .. } | SourcePrecondition::NodeAbsent { .. } => None,
    });
    let operations = batch.operations.iter().map(|value| match value {
        SourceEditOperation::SetExpression { expression, .. } => expression.source.len(),
    });
    budget::validate_expression_payload(preconditions.chain(operations))
}
