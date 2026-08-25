use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageErrorKind {
    Io,
    Json,
    Contract,
    RootEscape,
    MissingLockedDependency,
    DigestMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageError {
    kind: PackageErrorKind,
    message: String,
}

impl PackageError {
    pub(crate) fn new(kind: PackageErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn kind(&self) -> PackageErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for PackageError {}
