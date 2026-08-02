use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(super) fn corrupt<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::CorruptCache, message))
}

pub(super) fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

pub(super) fn resource_limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}

pub(super) fn io_error(operation: &str, error: rustix::io::Errno) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::Io,
        format!("cache cannot {operation}"),
        std::io::Error::from_raw_os_error(error.raw_os_error()),
    )
}

pub(super) fn corrupt_io(operation: &str, error: rustix::io::Errno) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::CorruptCache,
        format!("cache cannot safely {operation}"),
        std::io::Error::from_raw_os_error(error.raw_os_error()),
    )
}

pub(super) fn source_open_error(error: rustix::io::Errno) -> ArtifactError {
    if error == rustix::io::Errno::LOOP {
        ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "artifact source must not be a symbolic link",
        )
    } else {
        std::io::Error::from_raw_os_error(error.raw_os_error()).into()
    }
}

pub(super) fn serialization_error(error: serde_json::Error) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::CorruptCache,
        "cached artifact metadata is invalid",
        error,
    )
}
