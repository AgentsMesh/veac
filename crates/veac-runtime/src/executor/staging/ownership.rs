use std::path::Path;

use tempfile::{Builder, TempDir};

use super::directory::{Directory, EntryIdentity, EntryState};
use crate::RuntimeError;

pub(super) const STAGE_PREFIX: &str = ".veac-stage-";
pub(super) const MARKER_NAME: &str = ".veac-stage-owner";
const MARKER_SCHEMA: &str = "veac-stage-owner-v1";
const MAX_MARKER_BYTES: u64 = 512;

pub(super) fn create(parent: &Path, label: &str) -> Result<(TempDir, Directory), RuntimeError> {
    let directory = Builder::new()
        .prefix(STAGE_PREFIX)
        .tempdir_in(parent)
        .map_err(|error| RuntimeError::new(format!("cannot create {label}: {error}")))?;
    let descriptor = Directory::open(directory.path())?;
    let name = directory
        .path()
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RuntimeError::new("render staging name must be valid UTF-8"))?;
    mark(&descriptor, name)?;
    Ok((directory, descriptor))
}

pub(super) fn mark(stage: &Directory, stage_name: &str) -> Result<(), RuntimeError> {
    if !stage_name.starts_with(STAGE_PREFIX) || stage_name.contains('/') {
        return Err(RuntimeError::new("invalid owned render staging name"));
    }
    stage.write_all_sync(MARKER_NAME, &marker(stage_name))?;
    stage.sync()
}

pub(super) fn identity(stage: &Directory, stage_name: &str) -> Option<EntryIdentity> {
    let EntryState::Regular(expected) = stage.state(MARKER_NAME).ok()? else {
        return None;
    };
    if expected.size_bytes()? > MAX_MARKER_BYTES {
        return None;
    }
    let (bytes, actual) = stage.read_bounded(MARKER_NAME, MAX_MARKER_BYTES).ok()?;
    (actual == expected && bytes == marker(stage_name)).then_some(actual)
}

pub(super) fn remove(stage: &Directory, stage_name: &str) -> Result<(), RuntimeError> {
    if let Some(identity) = identity(stage, stage_name) {
        stage.remove_bound(MARKER_NAME, identity)?;
    }
    Ok(())
}

fn marker(stage_name: &str) -> Vec<u8> {
    format!("{MARKER_SCHEMA}\nstage={stage_name}\n").into_bytes()
}
