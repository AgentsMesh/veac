use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::super::directory::{Directory, EntryIdentity, EntryState};
use super::super::StagedOutput;
use crate::RuntimeError;

pub(super) fn validate(
    staging: &Path,
    stage: &Directory,
    outputs: &[StagedOutput],
    stale: &[PathBuf],
    deadline: Instant,
) -> Result<Vec<EntryIdentity>, RuntimeError> {
    if outputs.is_empty() {
        return Err(RuntimeError::new("staged task contains no output files"));
    }
    let mut targets = BTreeSet::new();
    let mut identities = Vec::with_capacity(outputs.len());
    for output in outputs {
        if !targets.insert(output.target()) {
            return Err(RuntimeError::new(
                "staged task contains duplicate output targets",
            ));
        }
        if output.source().parent() != Some(staging) {
            return Err(RuntimeError::new(
                "staged output escaped its transaction directory",
            ));
        }
        match (output, stage.state(name(output.source())?)) {
            (StagedOutput::File(file), Ok(EntryState::Regular(identity)))
                if file.allow_empty || identity.size_bytes().is_some_and(|value| value > 0) =>
            {
                identities.push(identity);
            }
            (_, Ok(EntryState::Missing)) => {
                return Err(RuntimeError::new(
                    "atomic render output commit failed: staged output is missing",
                ))
            }
            (StagedOutput::Package(package), Ok(EntryState::Directory(identity))) => {
                let inventory = super::super::package::inspect_hls(
                    &package.source,
                    &package.entrypoint,
                    deadline,
                )?;
                if inventory != package.inventory {
                    return Err(RuntimeError::new(
                        "staged package inventory changed before commit",
                    ));
                }
                identities.push(identity);
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
