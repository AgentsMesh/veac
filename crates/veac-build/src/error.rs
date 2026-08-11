use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErrorKind {
    InvalidContract,
    ResourceLimit,
    Cache,
    Cancelled,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    kind: BuildErrorKind,
    message: String,
}

impl BuildError {
    pub fn new(kind: BuildErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(BuildErrorKind::InvalidContract, message)
    }

    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self::new(BuildErrorKind::ResourceLimit, message)
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::new(BuildErrorKind::Cache, message)
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(BuildErrorKind::Cancelled, message)
    }

    pub fn kind(&self) -> BuildErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for BuildError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BuildError {}

pub type BuildResult<T> = Result<T, BuildError>;
