use crate::source_edit::{budget, SourceEditBatch, SourceEditOperation, SourcePrecondition};

pub(super) fn validate(batch: &SourceEditBatch) -> Result<(), super::SourceEditError> {
    let preconditions = batch.preconditions.iter().filter_map(|value| match value {
        SourcePrecondition::ExpressionEquals { expression, .. } => Some(expression.source.len()),
        SourcePrecondition::StatementEquals { statement, .. } => Some(statement.source.len()),
        SourcePrecondition::BodyEquals { body, .. } => Some(body.source.len()),
        SourcePrecondition::DeclarationEquals { declaration, .. } => Some(declaration.source.len()),
        SourcePrecondition::TopLevelDeclarationEquals { declaration, .. } => {
            Some(declaration.source.len())
        }
        SourcePrecondition::NodeExists { .. }
        | SourcePrecondition::NodeAbsent { .. }
        | SourcePrecondition::ImportExists { .. }
        | SourcePrecondition::ImportAbsent { .. } => None,
        SourcePrecondition::ImportEquals { import, .. } => {
            Some(import.path.len().saturating_add(import.alias.len()))
        }
    });
    let operations = batch.operations.iter().map(|value| match value {
        SourceEditOperation::SetExpression { expression, .. } => expression.source.len(),
        SourceEditOperation::SetStatement { statement, .. } => statement.source.len(),
        SourceEditOperation::SetBody { body, .. } => body.source.len(),
        SourceEditOperation::SetDeclaration { declaration, .. } => declaration.source.len(),
        SourceEditOperation::SetTopLevelDeclaration { declaration, .. } => declaration.source.len(),
        SourceEditOperation::InsertDeclaration { declaration, .. } => declaration.source.len(),
        SourceEditOperation::InsertImport { import, .. } => {
            import.path.len().saturating_add(import.alias.len())
        }
        SourceEditOperation::RemoveDeclaration { .. }
        | SourceEditOperation::RemoveImport { .. } => 0,
    });
    budget::validate_fragment_payload(preconditions.chain(operations))
}
