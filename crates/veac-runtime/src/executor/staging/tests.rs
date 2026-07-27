mod commit_contracts;
mod commit_identity_contracts;
mod deadline_contracts;
mod directory_behavior;
mod directory_contracts;
mod ffmpeg_contracts;
mod filter_script_contracts;
mod io_failure_tests;
mod journal_contracts;
mod journal_safety_contracts;
mod recovery_safety_contracts;
mod rollback_contracts;
mod validation_contracts;

use std::path::{Path, PathBuf};

use super::directory::{Directory, EntryIdentity, EntryState};
use super::{StagedFile, StagedTask};

#[derive(Clone, Copy)]
pub(super) enum RollbackFault {
    None,
    Remove,
    Restore,
}

impl super::commit::RollbackOperations for RollbackFault {
    fn remove(
        &self,
        output: &Directory,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), crate::RuntimeError> {
        if matches!(self, Self::Remove) {
            Err(crate::RuntimeError::new("injected rollback remove failure"))
        } else {
            output.remove_bound(name, expected)
        }
    }

    fn restore(
        &self,
        backup: &Directory,
        source: &str,
        output: &Directory,
        target: &str,
        expected: EntryIdentity,
    ) -> Result<(), crate::RuntimeError> {
        if matches!(self, Self::Restore) {
            Err(crate::RuntimeError::new(
                "injected rollback restore failure",
            ))
        } else {
            backup
                .rename_bound_to(source, expected, output, target)
                .map_err(|failure| failure.error)
        }
    }
}

pub(super) fn apply(
    staging: &Path,
    files: &[StagedFile],
    stale: &[PathBuf],
    allow_empty: bool,
) -> Result<(), crate::RuntimeError> {
    let stage = Directory::open(staging)?;
    let parent = files
        .first()
        .and_then(|file| file.target.parent())
        .unwrap_or(Path::new("."));
    let output = Directory::open(parent)?;
    super::commit::apply_locked(
        super::commit::CommitContext::new(staging, &stage, &output, files, stale, allow_empty),
        deadline(),
    )
    .map_err(super::commit::CommitFailure::into_error)
}

impl StagedTask {
    pub(super) fn commit_with_fault(
        self,
        locks: &crate::executor::locking::OutputLocks,
        fault: RollbackFault,
    ) -> Result<(), crate::RuntimeError> {
        let parent = super::common_parent_from_files(&self.files)?;
        let output = locks.directory(parent)?;
        let result = super::commit::apply_with(
            super::commit::CommitContext::new(
                self.directory.path(),
                &self.descriptor,
                &output,
                &self.files,
                &self.stale,
                self.allow_empty,
            ),
            || true,
            &fault,
        );
        self.finish_commit(result, &output, deadline())
    }
}

pub(super) fn recover(parent: &Path) -> Result<(), crate::RuntimeError> {
    let parents = vec![std::fs::canonicalize(parent).unwrap()];
    let deadline = deadline();
    let locks = crate::executor::locking::acquire_until(&parents, deadline)?;
    super::recovery::recover_until(&locks, &parents, deadline)
}

pub(super) fn prepare_journal(
    staging: &Path,
    files: &[StagedFile],
    stale: &[PathBuf],
) -> Result<super::journal::Journal, crate::RuntimeError> {
    let stage = Directory::open(staging)?;
    let parent = files[0].target.parent().unwrap_or(Path::new("."));
    let output = Directory::open(parent)?;
    let identities = files
        .iter()
        .map(
            |file| match stage.state(file.source.file_name().unwrap().to_str().unwrap())? {
                EntryState::Regular(identity) => Ok(identity),
                EntryState::Missing => {
                    Err(crate::RuntimeError::new("test stage source is missing"))
                }
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    super::journal::prepare(&stage, &output, files, &identities, stale)
}

pub(super) fn load_journal(staging: &Path) -> Result<super::journal::Journal, crate::RuntimeError> {
    let stage = Directory::open(staging)?;
    super::journal::load_bound(&stage).map(|(journal, _)| journal)
}

pub(super) fn commit_journal(
    staging: &Path,
    journal: &mut super::journal::Journal,
) -> Result<(), crate::RuntimeError> {
    let stage = Directory::open(staging)?;
    super::journal::commit(&stage, journal)
}

pub(super) fn task(directory: tempfile::TempDir, files: Vec<StagedFile>) -> StagedTask {
    let descriptor = Directory::open(directory.path()).unwrap();
    StagedTask {
        directory,
        descriptor,
        files,
        stale: Vec::new(),
        allow_empty: false,
    }
}

pub(super) fn deadline() -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_secs(10)
}
