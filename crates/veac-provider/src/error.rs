use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
    InvalidContract,
    UnsupportedCapability,
    VersionMismatch,
    NondeterministicProvider,
    UnsupportedApplication,
    Serialization,
    Artifact,
}

#[derive(Debug)]
pub struct ProviderError {
    pub kind: ProviderErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ProviderError {
    pub(crate) fn new(kind: ProviderErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub(crate) fn with_source(
        kind: ProviderErrorKind,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(value: serde_json::Error) -> Self {
        Self::with_source(
            ProviderErrorKind::Serialization,
            "provider contract cannot be canonicalized",
            value,
        )
    }
}

impl From<veac_artifact::ArtifactError> for ProviderError {
    fn from(value: veac_artifact::ArtifactError) -> Self {
        Self::with_source(
            ProviderErrorKind::Artifact,
            "provider artifact identity is invalid",
            value,
        )
    }
}

pub type ProviderResult<T> = Result<T, ProviderError>;

pub trait Validate {
    fn validate(&self) -> ProviderResult<()>;
}
