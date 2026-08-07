use crate::source_edit::{SourceEditBatch, SourceEditError, SourcePrecondition};

use super::SourceSnapshot;

pub(super) fn validate(value: &SourcePrecondition) -> Result<(), SourceEditError> {
    match value {
        SourcePrecondition::NodeExists { target } | SourcePrecondition::NodeAbsent { target } => {
            super::validate_target(target)
        }
        SourcePrecondition::ExpressionEquals {
            target,
            site,
            expression,
        } => super::expression::validate(target, site, &expression.source),
        SourcePrecondition::StatementEquals {
            target,
            site,
            statement,
        } => super::statement::validate(target, site, &statement.source),
        SourcePrecondition::BodyEquals { target, site, body } => {
            super::body::validate(target, *site, &body.source)
        }
        SourcePrecondition::DeclarationEquals {
            target,
            site,
            declaration,
        } => super::declaration::validate(target, *site, &declaration.source),
        SourcePrecondition::TopLevelDeclarationEquals {
            target,
            declaration,
        } => super::structural::validate_declaration_target(target)
            .and_then(|_| super::structural::validate_declaration_source(&declaration.source)),
        SourcePrecondition::ImportExists { target }
        | SourcePrecondition::ImportAbsent { target } => {
            super::structural::validate_import_target(target)
        }
        SourcePrecondition::ImportEquals { target, import } => {
            super::structural::validate_import_equals(target, import)
        }
    }
}

pub(super) fn require_satisfied(
    batch: &SourceEditBatch,
    snapshot: &impl SourceSnapshot,
) -> Result<(), SourceEditError> {
    for (index, value) in batch.preconditions.iter().enumerate() {
        if !is_satisfied(value, snapshot) {
            return Err(SourceEditError::PreconditionFailed { index });
        }
    }
    Ok(())
}

fn is_satisfied(value: &SourcePrecondition, snapshot: &impl SourceSnapshot) -> bool {
    match value {
        SourcePrecondition::NodeExists { target } => snapshot.node_exists(target),
        SourcePrecondition::NodeAbsent { target } => !snapshot.node_exists(target),
        SourcePrecondition::ExpressionEquals {
            target,
            site,
            expression,
        } => snapshot.expression_source(target, site) == Some(expression.source.as_str()),
        SourcePrecondition::StatementEquals {
            target,
            site,
            statement,
        } => snapshot.statement_source(target, site) == Some(statement.source.as_str()),
        SourcePrecondition::BodyEquals { target, site, body } => {
            snapshot.body_source(target, *site) == Some(body.source.as_str())
        }
        SourcePrecondition::DeclarationEquals {
            target,
            site,
            declaration,
        } => snapshot.declaration_source(target, *site) == Some(declaration.source.as_str()),
        SourcePrecondition::TopLevelDeclarationEquals {
            target,
            declaration,
        } => snapshot.top_level_declaration_source(target) == Some(declaration.source.as_str()),
        SourcePrecondition::ImportExists { target } => snapshot.import_exists(target),
        SourcePrecondition::ImportAbsent { target } => !snapshot.import_exists(target),
        SourcePrecondition::ImportEquals { target, import } => {
            snapshot.import_path(target) == Some(import.path.as_str())
        }
    }
}
