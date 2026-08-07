use std::collections::BTreeMap;

use crate::source_edit::{
    apply_text_edits, validate_source_edit_batch, SourceEditBatch, SourceEditOperation,
    SourceModuleAnchor, SourceRevision, TextEdit, TextRange,
};

use super::super::SourceIndex;
use super::model::{SourceModuleChange, SourceTransactionError};

pub(super) struct Selection {
    pub(super) previous_revision: SourceRevision,
    pub(super) previous_modules: Vec<String>,
    pub(super) changes: Vec<SourceModuleChange>,
    pub(super) sources: BTreeMap<String, String>,
}

pub(super) fn apply(
    sources: &BTreeMap<String, String>,
    index: &SourceIndex,
    batch: &SourceEditBatch,
) -> Result<Selection, SourceTransactionError> {
    validate_source_edit_batch(batch, index.revision(), index)
        .map_err(SourceTransactionError::Contract)?;
    let mut grouped = BTreeMap::<String, Vec<TextEdit>>::new();
    for (operation_index, operation) in batch.operations.iter().enumerate() {
        let (module, edit) = replacement(index, operation, operation_index)?;
        grouped.entry(module).or_default().push(edit);
    }
    let mut next = sources.clone();
    let mut changes = Vec::with_capacity(grouped.len());
    for (module, edits) in grouped {
        let current = sources
            .get(&module)
            .ok_or(SourceTransactionError::TargetNotFound { operation: 0 })?;
        let source = apply_text_edits(current, &edits).map_err(SourceTransactionError::Contract)?;
        next.insert(module.clone(), source.clone());
        changes.push(SourceModuleChange::new(module, current.clone(), source));
    }
    Ok(Selection {
        previous_revision: index.revision().clone(),
        previous_modules: sources.keys().cloned().collect(),
        changes,
        sources: next,
    })
}

fn replacement(
    index: &SourceIndex,
    operation: &SourceEditOperation,
    operation_index: usize,
) -> Result<(String, TextEdit), SourceTransactionError> {
    let missing = || SourceTransactionError::TargetNotFound {
        operation: operation_index,
    };
    let (module, range, replacement) = match operation {
        SourceEditOperation::SetExpression {
            target,
            site,
            expression,
        } => (
            &target.module,
            index.expression(target, site).ok_or_else(missing)?.range,
            expression.source.clone(),
        ),
        SourceEditOperation::SetStatement {
            target,
            site,
            statement,
        } => (
            &target.module,
            index.statement(target, site).ok_or_else(missing)?.range,
            statement.source.clone(),
        ),
        SourceEditOperation::SetBody { target, site, body } => (
            &target.module,
            index.body(target, *site).ok_or_else(missing)?.range,
            body.source.clone(),
        ),
        SourceEditOperation::SetDeclaration {
            target,
            site,
            declaration,
        } => (
            &target.module,
            index.declaration(target, *site).ok_or_else(missing)?.range,
            declaration.source.clone(),
        ),
        SourceEditOperation::SetTopLevelDeclaration {
            target,
            declaration,
        } => (
            &target.module,
            index.top_level(target).ok_or_else(missing)?.range,
            declaration.source.clone(),
        ),
        SourceEditOperation::InsertDeclaration {
            module,
            anchor,
            declaration,
        } => (
            module,
            insertion(index, module, anchor, operation_index)?,
            inserted(&declaration.source),
        ),
        SourceEditOperation::RemoveDeclaration { target } => (
            &target.module,
            index.top_level(target).ok_or_else(missing)?.range,
            String::new(),
        ),
        SourceEditOperation::InsertImport {
            module,
            anchor,
            import,
        } => (
            module,
            insertion(index, module, anchor, operation_index)?,
            inserted(&import.render()),
        ),
        SourceEditOperation::RemoveImport { target } => (
            &target.module,
            index.import(target).ok_or_else(missing)?.range,
            String::new(),
        ),
    };
    Ok((module.clone(), TextEdit { range, replacement }))
}

fn insertion(
    index: &SourceIndex,
    module: &str,
    anchor: &SourceModuleAnchor,
    operation: usize,
) -> Result<TextRange, SourceTransactionError> {
    let missing = || SourceTransactionError::TargetNotFound { operation };
    let offset = match anchor {
        SourceModuleAnchor::ModuleStart => index.module_range(module).ok_or_else(missing)?.start,
        SourceModuleAnchor::ModuleEnd => index.module_range(module).ok_or_else(missing)?.end,
        SourceModuleAnchor::BeforeDeclaration { target } => {
            index.top_level(target).ok_or_else(missing)?.range.start
        }
        SourceModuleAnchor::AfterDeclaration { target } => {
            index.top_level(target).ok_or_else(missing)?.range.end
        }
        SourceModuleAnchor::BeforeImport { target } => {
            index.import(target).ok_or_else(missing)?.range.start
        }
        SourceModuleAnchor::AfterImport { target } => {
            index.import(target).ok_or_else(missing)?.range.end
        }
    };
    Ok(TextRange {
        start: offset,
        end: offset,
    })
}

fn inserted(source: &str) -> String {
    format!("\n{source}\n")
}
