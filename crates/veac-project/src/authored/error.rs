use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDecodeError {
    pub path: String,
    pub message: String,
}

impl ProjectDecodeError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ProjectDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ProjectDecodeError {}

#[derive(Debug)]
pub enum ProjectAuthoringError {
    Load(String),
    Language(veac_lang::program::Diagnostics),
    Decode(ProjectDecodeError),
    Validation(crate::ProjectIssues),
    Serialization(serde_json::Error),
}

impl fmt::Display for ProjectAuthoringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(value) => write!(formatter, "project source load failed: {value}"),
            Self::Language(value) => write!(formatter, "project language failed: {value}"),
            Self::Decode(value) => write!(formatter, "project value decode failed: {value}"),
            Self::Validation(value) => value.fmt(formatter),
            Self::Serialization(value) => {
                write!(formatter, "project serialization failed: {value}")
            }
        }
    }
}

impl std::error::Error for ProjectAuthoringError {}

impl From<veac_lang::program::Diagnostics> for ProjectAuthoringError {
    fn from(value: veac_lang::program::Diagnostics) -> Self {
        Self::Language(value)
    }
}

impl From<ProjectDecodeError> for ProjectAuthoringError {
    fn from(value: ProjectDecodeError) -> Self {
        Self::Decode(value)
    }
}

impl From<crate::ProjectIssues> for ProjectAuthoringError {
    fn from(value: crate::ProjectIssues) -> Self {
        Self::Validation(value)
    }
}

impl From<serde_json::Error> for ProjectAuthoringError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}
