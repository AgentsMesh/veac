use super::directory::{Directory, EntryIdentity};
use crate::RuntimeError;

pub(super) const JOURNAL_NAME: &str = ".veac-commit.json";
pub(super) const TEMP_NAME: &str = ".veac-commit.tmp";
const SCHEMA_VERSION: u32 = 3;

mod model;
mod prepare;
mod storage;
mod validation;

pub(super) use model::{Journal, JournalEntry, JournalState};
pub(super) use prepare::prepare;

pub(super) fn commit_with(
    stage: &Directory,
    journal: &mut Journal,
    sync: impl FnOnce() -> Result<(), RuntimeError>,
) -> Result<(), storage::PersistFailure> {
    journal.state = JournalState::Committed;
    storage::persist_with(stage, journal, sync)
}

pub(super) fn load_bound(stage: &Directory) -> Result<(Journal, EntryIdentity), RuntimeError> {
    let (bytes, identity) = storage::load(stage)?;
    let value: Journal = serde_json::from_slice(&bytes)
        .map_err(|error| RuntimeError::new(format!("invalid render commit journal: {error}")))?;
    validation::validate(&value)?;
    Ok((value, identity))
}
