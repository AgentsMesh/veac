mod body;
mod declaration;
mod expression;
mod fragment;
mod payload;
mod precondition;
mod statement;
mod structural;

use super::{
    valid_sha256, validate_module_path, BodySite, DeclarationSite, ExpressionSite, SourceEditBatch,
    SourceEditError, SourceEditOperation, SourceNodeRef, SourceRevision, StatementSite,
    MAX_SOURCE_EDIT_OPERATIONS, MAX_SOURCE_EDIT_PRECONDITIONS, SOURCE_EDIT_SCHEMA,
    SOURCE_EDIT_SCHEMA_VERSION,
};

pub trait SourceSnapshot {
    fn node_exists(&self, target: &SourceNodeRef) -> bool;
    fn expression_source(&self, target: &SourceNodeRef, site: &ExpressionSite) -> Option<&str>;
    fn statement_source(&self, target: &SourceNodeRef, site: &StatementSite) -> Option<&str>;
    fn body_source(&self, target: &SourceNodeRef, site: BodySite) -> Option<&str>;
    fn declaration_source(&self, target: &SourceNodeRef, site: DeclarationSite) -> Option<&str>;
    fn top_level_declaration_source(&self, _target: &SourceNodeRef) -> Option<&str> {
        None
    }
    fn import_exists(&self, _target: &super::SourceImportRef) -> bool {
        false
    }
    fn import_path(&self, _target: &super::SourceImportRef) -> Option<&str> {
        None
    }
}

pub fn validate_source_edit_contract(batch: &SourceEditBatch) -> Result<(), SourceEditError> {
    if batch.schema != SOURCE_EDIT_SCHEMA {
        return Err(SourceEditError::InvalidSchema);
    }
    if batch.schema_version != SOURCE_EDIT_SCHEMA_VERSION {
        return Err(SourceEditError::UnsupportedSchemaVersion(
            batch.schema_version,
        ));
    }
    if veac_ir::OperationId::new(batch.operation_id.as_str()).is_err() {
        return Err(SourceEditError::InvalidOperationId(
            batch.operation_id.to_string(),
        ));
    }
    if !batch.atomic {
        return Err(SourceEditError::AtomicRequired);
    }
    if batch.operations.is_empty() {
        return Err(SourceEditError::EmptyOperations);
    }
    if batch.preconditions.len() > MAX_SOURCE_EDIT_PRECONDITIONS {
        return Err(SourceEditError::TooManyPreconditions {
            limit: MAX_SOURCE_EDIT_PRECONDITIONS,
        });
    }
    if batch.operations.len() > MAX_SOURCE_EDIT_OPERATIONS {
        return Err(SourceEditError::TooManyOperations {
            limit: MAX_SOURCE_EDIT_OPERATIONS,
        });
    }
    validate_digest(&batch.base_revision)?;
    for precondition in &batch.preconditions {
        precondition::validate(precondition)?;
    }
    for operation in &batch.operations {
        validate_operation(operation)?;
    }
    payload::validate(batch)?;
    Ok(())
}

pub fn validate_source_edit_batch(
    batch: &SourceEditBatch,
    current: &SourceRevision,
    snapshot: &impl SourceSnapshot,
) -> Result<(), SourceEditError> {
    validate_source_edit_contract(batch)?;
    validate_digest(current)?;
    if batch.base_revision != *current {
        return Err(SourceEditError::StaleRevision {
            expected: describe_revision(&batch.base_revision),
            actual: describe_revision(current),
        });
    }
    precondition::require_satisfied(batch, snapshot)
}

pub(super) fn validate_operation(value: &SourceEditOperation) -> Result<(), SourceEditError> {
    match value {
        SourceEditOperation::SetExpression {
            target,
            site,
            expression,
        } => expression::validate(target, site, &expression.source),
        SourceEditOperation::SetStatement {
            target,
            site,
            statement,
        } => statement::validate(target, site, &statement.source),
        SourceEditOperation::SetBody { target, site, body } => {
            body::validate(target, *site, &body.source)
        }
        SourceEditOperation::SetDeclaration {
            target,
            site,
            declaration,
        } => declaration::validate(target, *site, &declaration.source),
        SourceEditOperation::SetTopLevelDeclaration {
            target,
            declaration,
        } => structural::validate_declaration_target(target)
            .and_then(|_| structural::validate_declaration_source(&declaration.source)),
        SourceEditOperation::InsertDeclaration {
            module,
            anchor,
            declaration,
        } => structural::validate_module_anchor(module, anchor)
            .and_then(|_| structural::validate_declaration_source(&declaration.source)),
        SourceEditOperation::RemoveDeclaration { target } => {
            structural::validate_declaration_target(target)
        }
        SourceEditOperation::InsertImport {
            module,
            anchor,
            import,
        } => structural::validate_module_anchor(module, anchor)
            .and_then(|_| structural::validate_import_source(import)),
        SourceEditOperation::RemoveImport { target } => structural::validate_import_target(target),
    }
}

pub(super) fn validate_target(target: &SourceNodeRef) -> Result<(), SourceEditError> {
    validate_module_path(&target.module)?;
    if let Some(value) = target
        .path
        .identifiers()
        .into_iter()
        .find(|value| !crate::name::is_name(value))
    {
        return Err(SourceEditError::InvalidNodeId(value.to_owned()));
    }
    Ok(())
}

fn validate_digest(value: &SourceRevision) -> Result<(), SourceEditError> {
    for digest in [
        &value.authored_source_graph_sha256,
        &value.complete_source_graph_sha256,
    ] {
        if !valid_sha256(digest) {
            return Err(SourceEditError::InvalidDigest(digest.clone()));
        }
    }
    Ok(())
}

fn describe_revision(value: &SourceRevision) -> String {
    format!(
        "authored={}, complete={}",
        value.authored_source_graph_sha256, value.complete_source_graph_sha256
    )
}
