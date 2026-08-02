mod payload;

use super::{
    valid_sha256, validate_module_path, ExpressionSite, SourceEditBatch, SourceEditError,
    SourceEditOperation, SourceNodeRef, SourcePrecondition, SourceRevision,
    MAX_SOURCE_EDIT_OPERATIONS, MAX_SOURCE_EDIT_PRECONDITIONS,
    MAX_SOURCE_EDIT_SINGLE_EXPRESSION_BYTES, SOURCE_EDIT_SCHEMA, SOURCE_EDIT_SCHEMA_VERSION,
};

pub trait SourceSnapshot {
    fn node_exists(&self, target: &SourceNodeRef) -> bool;
    fn expression_source(&self, target: &SourceNodeRef, site: &ExpressionSite) -> Option<&str>;
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
        validate_precondition(precondition)?;
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
            expected: batch.base_revision.source_graph_sha256.clone(),
            actual: current.source_graph_sha256.clone(),
        });
    }
    for (index, precondition) in batch.preconditions.iter().enumerate() {
        let satisfied = match precondition {
            SourcePrecondition::NodeExists { target } => snapshot.node_exists(target),
            SourcePrecondition::NodeAbsent { target } => !snapshot.node_exists(target),
            SourcePrecondition::ExpressionEquals {
                target,
                site,
                expression,
            } => snapshot.expression_source(target, site) == Some(expression.source.as_str()),
        };
        if !satisfied {
            return Err(SourceEditError::PreconditionFailed { index });
        }
    }
    Ok(())
}

fn validate_precondition(value: &SourcePrecondition) -> Result<(), SourceEditError> {
    match value {
        SourcePrecondition::NodeExists { target } | SourcePrecondition::NodeAbsent { target } => {
            validate_target(target)
        }
        SourcePrecondition::ExpressionEquals {
            target,
            site,
            expression,
        } => validate_expression_target(target, site, &expression.source),
    }
}

pub(super) fn validate_operation(value: &SourceEditOperation) -> Result<(), SourceEditError> {
    match value {
        SourceEditOperation::SetExpression {
            target,
            site,
            expression,
        } => validate_expression_target(target, site, &expression.source),
    }
}

fn validate_expression_target(
    target: &SourceNodeRef,
    site: &ExpressionSite,
    expression: &str,
) -> Result<(), SourceEditError> {
    validate_target(target)?;
    if !site.accepts_target(target) {
        return Err(SourceEditError::IncompatibleExpressionSite);
    }
    if site
        .name()
        .is_some_and(|value| !crate::name::is_name(value))
    {
        return Err(SourceEditError::InvalidNodeId(
            site.name().unwrap_or_default().to_owned(),
        ));
    }
    if expression.trim().is_empty() || expression.len() > MAX_SOURCE_EDIT_SINGLE_EXPRESSION_BYTES {
        return Err(SourceEditError::InvalidExpression(
            "source must contain 1..65536 bytes".to_owned(),
        ));
    }
    if expression.contains('\0') {
        return Err(SourceEditError::InvalidExpression(
            "source must not contain NUL".to_owned(),
        ));
    }
    if matches!(
        site,
        ExpressionSite::PresetTextStyleField { .. }
            | ExpressionSite::PresetTextLayoutField { .. }
            | ExpressionSite::PresetColorField { .. }
            | ExpressionSite::PresetAudioProcessorField { .. }
            | ExpressionSite::PresetAudioEqBandField { .. }
            | ExpressionSite::PresetDeliveryField { .. }
    ) {
        crate::program::validate_expression_fragment(expression)
            .map_err(SourceEditError::InvalidExpression)?;
    } else {
        validate_pure_expression(expression)?;
    }
    Ok(())
}

fn validate_pure_expression(source: &str) -> Result<(), SourceEditError> {
    let source = source.trim();
    let expression = source
        .strip_prefix("${")
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or(source)
        .trim();
    crate::program::expression::referenced_symbols(expression)
        .map(|_| ())
        .map_err(|error| SourceEditError::InvalidExpression(error.to_string()))
}

fn validate_target(target: &SourceNodeRef) -> Result<(), SourceEditError> {
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
    if valid_sha256(&value.source_graph_sha256) {
        Ok(())
    } else {
        Err(SourceEditError::InvalidDigest(
            value.source_graph_sha256.clone(),
        ))
    }
}
