use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    apply_borrowed_text_edits, text_edit::ensure_edit_count, BorrowedTextEdit, ExpressionSite,
    SourceEditError, SourceEditOperation, SourceNodeRef, TextEdit, TextRange,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTextReplacement {
    pub operation_index: usize,
    pub target: SourceNodeRef,
    pub site: ExpressionSite,
    pub edit: TextEdit,
}

pub fn resolve_set_expression_text(
    operation_index: usize,
    operation: &SourceEditOperation,
    range: TextRange,
) -> Result<ResolvedTextReplacement, SourceEditError> {
    super::validation::validate_operation(operation)?;
    Ok(match operation {
        SourceEditOperation::SetExpression {
            target,
            site,
            expression,
        } => ResolvedTextReplacement {
            operation_index,
            target: target.clone(),
            site: site.clone(),
            edit: TextEdit {
                range,
                replacement: expression.source.clone(),
            },
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
