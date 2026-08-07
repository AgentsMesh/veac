use crate::source_edit::{ExpressionSite, SourceEditError, SourceNodeRef};

pub(super) fn validate(
    target: &SourceNodeRef,
    site: &ExpressionSite,
    source: &str,
) -> Result<(), SourceEditError> {
    super::validate_target(target)?;
    if !site.is_valid() {
        return Err(SourceEditError::InvalidExpressionPath);
    }
    if !site.accepts_target(target) {
        return Err(SourceEditError::IncompatibleExpressionSite);
    }
    super::fragment::validate(source)
        .map_err(|message| SourceEditError::InvalidExpression(message.to_owned()))?;
    validate_pure(source)
}

fn validate_pure(source: &str) -> Result<(), SourceEditError> {
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
