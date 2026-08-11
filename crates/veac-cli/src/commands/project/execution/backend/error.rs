use std::fmt::Display;

use veac_build::ProjectBackendError;

pub(super) trait ProjectResultExt<T> {
    fn project_context(self, context: &str) -> Result<T, ProjectBackendError>;
}

impl<T, E: Display> ProjectResultExt<T> for Result<T, E> {
    fn project_context(self, context: &str) -> Result<T, ProjectBackendError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(ProjectBackendError::failed(format!("{context}: {error}"))),
        }
    }
}

pub(super) trait ProjectOptionExt<T> {
    fn project_required(self, message: &str) -> Result<T, ProjectBackendError>;
}

impl<T> ProjectOptionExt<T> for Option<T> {
    fn project_required(self, message: &str) -> Result<T, ProjectBackendError> {
        match self {
            Some(value) => Ok(value),
            None => Err(ProjectBackendError::failed(message)),
        }
    }
}

#[cfg(test)]
#[path = "error/tests.rs"]
mod tests;
