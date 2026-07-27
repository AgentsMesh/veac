use std::path::PathBuf;
use std::time::Instant;

use super::directory::{Directory, EntryState};
use super::journal::{self, Journal, JournalEntry, JournalState, JOURNAL_NAME};
use crate::executor::locking::OutputLocks;
use crate::RuntimeError;

const STAGE_PREFIX: &str = ".veac-stage-";
const MAX_PARENT_ENTRIES: usize = 65_536;

mod cleanup;

pub(super) fn recover_until(
    locks: &OutputLocks,
    parents: &[PathBuf],
    deadline: Instant,
) -> Result<(), RuntimeError> {
    for parent in parents {
        active(deadline)?;
        let output = locks.setup_directory_until(parent, deadline)?;
        recover_parent(&output, deadline)?;
    }
    Ok(())
}

fn recover_parent(output: &Directory, deadline: Instant) -> Result<(), RuntimeError> {
    for name in output.entries(MAX_PARENT_ENTRIES)? {
        active(deadline)?;
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(STAGE_PREFIX) {
            continue;
        }
        let stage = output
            .child(name)
            .map_err(|_| invalid_error("reserved staging path must be a non-symlink directory"))?;
        match stage.state(JOURNAL_NAME)? {
            EntryState::Regular(_) => recover_bound_stage(output, name, stage, deadline)?,
            EntryState::Missing => {}
        }
    }
    Ok(())
}

pub(super) fn recover_bound_stage(
    output: &Directory,
    stage_name: &str,
    stage: Directory,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    active(deadline)?;
    let (transaction, journal_identity) = journal::load_bound(&stage)?;
    let backup = stage.child("backups")?;
    match transaction.state {
        JournalState::Prepared => rollback(output, &backup, &transaction, deadline)?,
        JournalState::Committed => finalize(output, &backup, &transaction, deadline)?,
    }
    cleanup::transaction(
        output,
        stage_name,
        &stage,
        &backup,
        &transaction,
        journal_identity,
        deadline,
    )
}

pub(super) fn discard_bound_stage(
    output: &Directory,
    stage_name: &str,
    stage: &Directory,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    cleanup::uncommitted(output, stage_name, stage, deadline)
}

fn rollback(
    output: &Directory,
    backup: &Directory,
    journal: &Journal,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    for (index, entry) in journal.entries.iter().enumerate() {
        active(deadline)?;
        let backup_name = index.to_string();
        match (entry.original, backup.state(&backup_name)?) {
            (Some(original), EntryState::Regular(actual)) if actual == original => {
                remove_installed(output, entry)?;
                backup
                    .rename_bound_to(&backup_name, original, output, &entry.target)
                    .map_err(|failure| failure.error)?;
            }
            (Some(original), EntryState::Missing) => output.require(&entry.target, original)?,
            (Some(_), EntryState::Regular(_)) => {
                return invalid("recovery backup changed identity")
            }
            (None, EntryState::Missing) => remove_installed(output, entry)?,
            (None, EntryState::Regular(_)) => {
                return invalid("new output unexpectedly has a recovery backup")
            }
        }
    }
    Ok(())
}

fn finalize(
    output: &Directory,
    backup: &Directory,
    journal: &Journal,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    for (index, entry) in journal.entries.iter().enumerate() {
        active(deadline)?;
        match entry.source_identity {
            Some(expected) => output.require(&entry.target, expected)?,
            None if !output.missing(&entry.target)? => {
                return invalid("committed stale output still exists")
            }
            None => {}
        }
        let backup_name = index.to_string();
        match (entry.original, backup.state(&backup_name)?) {
            (Some(expected), EntryState::Regular(actual)) if actual == expected => {
                backup.remove_bound(&backup_name, expected)?;
            }
            (_, EntryState::Missing) => {}
            _ => return invalid("committed recovery backup changed identity"),
        }
    }
    Ok(())
}

fn remove_installed(output: &Directory, entry: &JournalEntry) -> Result<(), RuntimeError> {
    match (entry.source_identity, output.state(&entry.target)?) {
        (Some(expected), EntryState::Regular(actual)) if actual == expected => {
            output.remove_bound(&entry.target, expected)
        }
        (_, EntryState::Missing) => Ok(()),
        (None, EntryState::Regular(_)) => invalid("stale output reappeared during rollback"),
        _ => invalid("installed recovery output changed identity"),
    }
}

pub(super) fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(invalid_error(message))
}

fn invalid_error(message: &str) -> RuntimeError {
    RuntimeError::new(format!("render commit recovery failed: {message}"))
}

fn active(deadline: Instant) -> Result<(), RuntimeError> {
    crate::executor::deadline::ensure_setup(deadline)
}
