use crate::source_edit::{BodySite, SourceEditError, SourceNodeRef};

pub(super) fn validate(
    target: &SourceNodeRef,
    site: BodySite,
    source: &str,
) -> Result<(), SourceEditError> {
    super::validate_target(target)?;
    if !site.accepts_target(target) {
        return Err(SourceEditError::IncompatibleBodySite);
    }
    if let Err(message) = super::fragment::validate(source) {
        return Err(SourceEditError::InvalidBody(message.to_owned()));
    }
    match crate::program::expression::validate_function_body(source) {
        Ok(()) => Ok(()),
        Err(error) => Err(SourceEditError::InvalidBody(error.to_string())),
    }
}
