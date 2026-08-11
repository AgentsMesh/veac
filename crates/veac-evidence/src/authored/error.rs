use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceDecodeError {
    pub path: String,
    pub message: String,
}

impl EvidenceDecodeError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for EvidenceDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for EvidenceDecodeError {}

#[derive(Debug)]
pub enum EvidenceAuthoringError {
    Load(String),
    Language(veac_lang::program::Diagnostics),
    Decode(EvidenceDecodeError),
    Validation(crate::ValidationError),
    Identity(crate::BundleError),
}

impl fmt::Display for EvidenceAuthoringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(value) => write!(formatter, "evidence source load failed: {value}"),
            Self::Language(value) => write!(formatter, "evidence language failed: {value}"),
            Self::Decode(value) => write!(formatter, "evidence value decode failed: {value}"),
            Self::Validation(value) => value.fmt(formatter),
            Self::Identity(value) => write!(formatter, "evidence identity failed: {value}"),
        }
    }
}

impl std::error::Error for EvidenceAuthoringError {}

impl From<veac_lang::program::Diagnostics> for EvidenceAuthoringError {
    fn from(value: veac_lang::program::Diagnostics) -> Self {
        Self::Language(value)
    }
}

impl From<EvidenceDecodeError> for EvidenceAuthoringError {
    fn from(value: EvidenceDecodeError) -> Self {
        Self::Decode(value)
    }
}

impl From<crate::ValidationError> for EvidenceAuthoringError {
    fn from(value: crate::ValidationError) -> Self {
        Self::Validation(value)
    }
}

impl From<crate::BundleError> for EvidenceAuthoringError {
    fn from(value: crate::BundleError) -> Self {
        Self::Identity(value)
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
