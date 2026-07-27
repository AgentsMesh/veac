use super::{Directory, EntryIdentity, Journal, JOURNAL_NAME, TEMP_NAME};
use crate::RuntimeError;

const MAX_BYTES: u64 = 1024 * 1024;

pub(super) fn load(stage: &Directory) -> Result<(Vec<u8>, EntryIdentity), RuntimeError> {
    stage
        .read_bounded(JOURNAL_NAME, MAX_BYTES)
        .map_err(|_| bounded_error())
}

pub(super) fn persist(stage: &Directory, journal: &Journal) -> Result<(), RuntimeError> {
    let bytes = serde_json_canonicalizer::to_vec(journal).map_err(|error| {
        RuntimeError::new(format!("cannot encode render commit journal: {error}"))
    })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(bounded_error());
    }
    let identity = stage
        .write_all_sync(TEMP_NAME, &bytes)
        .map_err(journal_error)?;
    stage
        .rename_bound_to(TEMP_NAME, identity, stage, JOURNAL_NAME)
        .map_err(|failure| journal_error(failure.error))?;
    stage.sync().map_err(journal_error)
}

fn bounded_error() -> RuntimeError {
    RuntimeError::new(
        "invalid render commit journal: commit journal must be a bounded regular non-symlink file",
    )
}

fn journal_error(error: impl std::fmt::Display) -> RuntimeError {
    RuntimeError::new(format!("render commit journal I/O failed: {error}"))
}
