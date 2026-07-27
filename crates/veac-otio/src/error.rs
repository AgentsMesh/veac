use std::fmt;

#[derive(Debug)]
pub enum OtioError {
    Json(serde_json::Error),
    Contract(String),
    Time(String),
    Loss(OtioLossReport),
    Edit(String),
}

impl OtioError {
    pub(crate) fn contract(message: impl Into<String>) -> Self {
        Self::Contract(message.into())
    }

    pub(crate) fn time(message: impl fmt::Display) -> Self {
        Self::Time(message.to_string())
    }
}

impl fmt::Display for OtioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "OTIO JSON failed: {error}"),
            Self::Contract(message) => write!(formatter, "OTIO contract failed: {message}"),
            Self::Time(message) => write!(formatter, "OTIO time failed: {message}"),
            Self::Loss(report) => {
                write!(formatter, "OTIO conversion has {} loss(es)", report.len())
            }
            Self::Edit(message) => write!(formatter, "OTIO edit proposal failed: {message}"),
        }
    }
}

impl std::error::Error for OtioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for OtioError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

use crate::OtioLossReport;
