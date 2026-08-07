use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    apply_borrowed_text_edits, text_edit::ensure_edit_count, BorrowedTextEdit, SourceEditError,
    SourceEditOperation, SourceNodeRef, TextEdit, TextRange,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTextReplacement {
    pub operation_index: usize,
    pub target: SourceNodeRef,
    pub edit: TextEdit,
}

pub fn resolve_source_edit_text(
    operation_index: usize,
    operation: &SourceEditOperation,
    range: TextRange,
) -> Result<ResolvedTextReplacement, SourceEditError> {
    super::validation::validate_operation(operation)?;
    let (target, replacement) = match operation {
        SourceEditOperation::SetExpression {
            target, expression, ..
        } => (target, &expression.source),
        SourceEditOperation::SetStatement {
            target, statement, ..
        } => (target, &statement.source),
        SourceEditOperation::SetBody { target, body, .. } => (target, &body.source),
        SourceEditOperation::SetDeclaration {
            target,
            declaration,
            ..
        } => (target, &declaration.source),
        SourceEditOperation::SetTopLevelDeclaration {
            target,
            declaration,
        } => (target, &declaration.source),
        SourceEditOperation::InsertDeclaration { .. }
        | SourceEditOperation::RemoveDeclaration { .. }
        | SourceEditOperation::InsertImport { .. }
        | SourceEditOperation::RemoveImport { .. } => {
            return Err(SourceEditError::StructuralOperationRequiresIndex)
        }
    };
    Ok(ResolvedTextReplacement {
        operation_index,
        target: target.clone(),
        edit: TextEdit {
            range,
            replacement: replacement.clone(),
        },
    })
}

pub fn apply_resolved_text_replacements(
    module: &str,
    source: &str,
    replacements: &[ResolvedTextReplacement],
) -> Result<String, SourceEditError> {
    for replacement in replacements {
        if replacement.target.module != module {
            return Err(SourceEditError::ResolvedModuleMismatch {
                expected: module.to_owned(),
                actual: replacement.target.module.clone(),
            });
        }
    }
    ensure_edit_count(replacements.len())?;
    let edits = replacements
        .iter()
        .map(|replacement| BorrowedTextEdit {
            range: replacement.edit.range,
            replacement: &replacement.edit.replacement,
        })
        .collect();
    apply_borrowed_text_edits(source, edits)
}
