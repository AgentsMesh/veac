use super::{Directory, EntryIdentity, Journal, JOURNAL_NAME, TEMP_NAME};
use crate::RuntimeError;

const MAX_BYTES: u64 = 1024 * 1024;

pub(super) fn load(stage: &Directory) -> Result<(Vec<u8>, EntryIdentity), RuntimeError> {
    stage
        .read_bounded(JOURNAL_NAME, MAX_BYTES)
        .map_err(|_| bounded_error())
}

pub(in crate::executor::staging) struct PersistFailure {
    pub error: RuntimeError,
    pub crossed_commit: bool,
}

pub(super) fn persist(stage: &Directory, journal: &Journal) -> Result<(), PersistFailure> {
    persist_with(stage, journal, || stage.sync())
}

pub(super) fn persist_with(
    stage: &Directory,
    journal: &Journal,
    sync: impl FnOnce() -> Result<(), RuntimeError>,
) -> Result<(), PersistFailure> {
    let bytes = serde_json_canonicalizer::to_vec(journal).map_err(|error| {
        before(RuntimeError::new(format!(
            "cannot encode render commit journal: {error}"
        )))
    })?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(before(bounded_error()));
    }
    let identity = stage
        .write_all_sync(TEMP_NAME, &bytes)
        .map_err(|error| before(journal_error(error)))?;
    if let Err(failure) = stage.rename_bound_to(TEMP_NAME, identity, stage, JOURNAL_NAME) {
        return Err(PersistFailure {
            error: journal_error(failure.error),
            crossed_commit: failure.crossed_commit,
        });
    }
    sync().map_err(|error| PersistFailure {
        error: journal_error(error),
        crossed_commit: true,
    })
}

fn before(error: RuntimeError) -> PersistFailure {
    PersistFailure {
        error,
        crossed_commit: false,
    }
}

fn bounded_error() -> RuntimeError {
    RuntimeError::new(
        "invalid render commit journal: commit journal must be a bounded regular non-symlink file",
    )
}

fn journal_error(error: impl std::fmt::Display) -> RuntimeError {
    RuntimeError::new(format!("render commit journal I/O failed: {error}"))
}
