use std::fmt;

use crate::{CaptionFormat, ValidationErrors};

#[derive(Debug)]
pub enum CaptionError {
    Parse {
        format: CaptionFormat,
        message: String,
    },
    Validation(ValidationErrors),
    Json(serde_json::Error),
    Time(String),
    Ir(String),
}

impl CaptionError {
    pub(crate) fn parse(format: CaptionFormat, error: impl fmt::Display) -> Self {
        Self::Parse {
            format,
            message: error.to_string(),
        }
    }

    pub(crate) fn time(message: impl Into<String>) -> Self {
        Self::Time(message.into())
    }

    pub(crate) fn ir(message: impl Into<String>) -> Self {
        Self::Ir(message.into())
    }
}

impl fmt::Display for CaptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { format, message } => {
                write!(formatter, "{format} parse failed: {message}")
            }
            Self::Validation(errors) => write!(formatter, "caption validation failed: {errors}"),
            Self::Json(error) => write!(formatter, "caption JSON failed: {error}"),
            Self::Time(message) => write!(formatter, "caption time conversion failed: {message}"),
            Self::Ir(message) => write!(formatter, "caption IR conversion failed: {message}"),
        }
    }
}

impl std::error::Error for CaptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ValidationErrors> for CaptionError {
    fn from(value: ValidationErrors) -> Self {
        Self::Validation(value)
    }
}

impl From<serde_json::Error> for CaptionError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
