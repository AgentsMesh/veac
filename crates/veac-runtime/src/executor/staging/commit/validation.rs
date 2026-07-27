use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::directory::{Directory, EntryIdentity, EntryState};
use super::super::StagedFile;
use crate::RuntimeError;

pub(super) fn validate(
    staging: &Path,
    stage: &Directory,
    files: &[StagedFile],
    stale: &[PathBuf],
    allow_empty: bool,
) -> Result<Vec<EntryIdentity>, RuntimeError> {
    if files.is_empty() {
        return Err(RuntimeError::new("staged task contains no output files"));
    }
    let mut targets = BTreeSet::new();
    let mut identities = Vec::with_capacity(files.len());
    for file in files {
        if !targets.insert(&file.target) {
            return Err(RuntimeError::new(
                "staged task contains duplicate output targets",
            ));
        }
        if file.source.parent() != Some(staging) {
            return Err(RuntimeError::new(
                "staged output escaped its transaction directory",
            ));
        }
        match stage.state(name(&file.source)?) {
            Ok(EntryState::Regular(identity)) if allow_empty || identity.size_bytes > 0 => {
                identities.push(identity);
            }
            Ok(EntryState::Missing) => {
                return Err(RuntimeError::new(
                    "atomic render output commit failed: staged output is missing",
                ))
            }
            _ => {
                return Err(RuntimeError::new(
                    "staged render output is not a regular non-empty file",
                ))
            }
        }
    }
    if stale.iter().any(|path| targets.contains(path)) {
        return Err(RuntimeError::new(
            "stale and current render outputs overlap",
        ));
    }
    Ok(identities)
}

fn name(path: &Path) -> Result<&str, RuntimeError> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RuntimeError::new("transaction file name must be valid UTF-8"))
}
