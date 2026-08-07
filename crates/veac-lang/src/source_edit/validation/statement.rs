use crate::source_edit::{SourceEditError, SourceNodeRef, StatementSite};

pub(super) fn validate(
    target: &SourceNodeRef,
    site: &StatementSite,
    source: &str,
) -> Result<(), SourceEditError> {
    super::validate_target(target)?;
    if !site.is_valid() {
        return Err(SourceEditError::InvalidStatementPath);
    }
    if !site.accepts_target(target) {
        return Err(SourceEditError::IncompatibleStatementSite);
    }
    super::fragment::validate(source)
        .map_err(|message| SourceEditError::InvalidStatement(message.to_owned()))?;
    crate::program::expression::validate_function_statement(source)
        .map_err(|error| SourceEditError::InvalidStatement(error.to_string()))
}
