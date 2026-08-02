use std::ffi::OsString;
use std::time::Instant;

use super::{active, invalid, Directory, EntryState, Journal, JOURNAL_NAME};
use crate::executor::staging::directory::EntryIdentity;
use crate::executor::staging::journal::TEMP_NAME;
use crate::executor::staging::ownership;
use crate::RuntimeError;

const MAX_STAGE_ENTRIES: usize = 65_536;

pub(super) fn transaction(
    output: &Directory,
    stage_name: &str,
    stage: &Directory,
    backup: &Directory,
    journal: &Journal,
    journal_identity: EntryIdentity,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    for entry in &journal.entries {
        active(deadline)?;
        if let (Some(name), Some(expected)) = (&entry.source, entry.source_identity) {
            match stage.state(name)? {
                EntryState::Regular(actual) | EntryState::Directory(actual)
                    if actual == expected =>
                {
                    remove(stage, name, expected, deadline)?;
                }
                EntryState::Missing => {}
                _ => return invalid("staged recovery source changed identity"),
            }
        }
    }
    stage.remove_if_regular(TEMP_NAME)?;
    ownership::remove(stage, stage_name)?;
    if !backup.entries(MAX_STAGE_ENTRIES)?.is_empty() {
        return invalid("recovery backup directory contains unexpected entries");
    }
    stage.remove_child("backups", backup)?;
    let entries = stage.entries(MAX_STAGE_ENTRIES)?;
    if entries.as_slice() != [OsString::from(JOURNAL_NAME)] {
        return invalid("recovery staging directory contains unexpected entries");
    }
    stage.remove_bound(JOURNAL_NAME, journal_identity)?;
    output.remove_child(stage_name, stage)?;
    output.sync()?;
    active(deadline)
}

pub(super) fn uncommitted(
    output: &Directory,
    stage_name: &str,
    stage: &Directory,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    for name in stage.entries(MAX_STAGE_ENTRIES)? {
        active(deadline)?;
        let name = match name.to_str() {
            Some(name) => name,
            None => return Err(RuntimeError::new("staging entry name must be valid UTF-8")),
        };
        if name == "backups" {
            discard_child(stage, name, deadline)?;
            continue;
        }
        match stage.state(name)? {
            EntryState::Regular(identity) | EntryState::Directory(identity) => {
                remove(stage, name, identity, deadline)?
            }
            EntryState::Missing => {}
        }
    }
    output.remove_child(stage_name, stage)?;
    output.sync()?;
    active(deadline)
}

fn discard_child(parent: &Directory, name: &str, deadline: Instant) -> Result<(), RuntimeError> {
    let child = parent.child(name)?;
    for entry in child.entries(MAX_STAGE_ENTRIES)? {
        let entry = match entry.to_str() {
            Some(entry) => entry,
            None => return Err(RuntimeError::new("staging entry name must be valid UTF-8")),
        };
        match child.state(entry)? {
            EntryState::Regular(identity) | EntryState::Directory(identity) => {
                remove(&child, entry, identity, deadline)?
            }
            EntryState::Missing => {}
        }
    }
    parent.remove_child(name, &child)
}

fn remove(
    directory: &Directory,
    name: &str,
    identity: EntryIdentity,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    directory.remove_tree_bound(name, identity, MAX_STAGE_ENTRIES, || {
        Instant::now() < deadline
    })
}
