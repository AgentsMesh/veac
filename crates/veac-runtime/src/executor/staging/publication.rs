use std::path::Path;
use std::time::Instant;

use tempfile::TempDir;

use super::directory::{Directory, EntryState};
use super::{commit, journal, ownership, recovery, stale, StagedOutput, StagedTask};
use crate::executor::deadline;
use crate::executor::locking::OutputLocks;
use crate::RuntimeError;

pub(in crate::executor) struct Publication {
    directory: TempDir,
    descriptor: Directory,
    outputs: Vec<StagedOutput>,
    stale: Vec<super::StaleFamily>,
    next_source: usize,
}

pub(in crate::executor) struct PublicationFailure {
    error: RuntimeError,
    committed: bool,
}

impl Publication {
    pub(in crate::executor) fn new(parent: &Path) -> Result<Self, RuntimeError> {
        let (directory, descriptor) = ownership::create(parent, "bundle publication area")?;
        Ok(Self {
            directory,
            descriptor,
            outputs: Vec::new(),
            stale: Vec::new(),
            next_source: 0,
        })
    }

    pub(in crate::executor) fn adopt(&mut self, task: StagedTask) -> Result<(), RuntimeError> {
        let StagedTask {
            directory,
            descriptor,
            outputs,
            stale,
        } = task;
        for mut output in outputs {
            let source = component(output.source())?;
            let identity = match (&output, descriptor.state(source)?) {
                (StagedOutput::File(file), EntryState::Regular(value))
                    if file.allow_empty || value.size_bytes().is_some_and(|size| size > 0) =>
                {
                    value
                }
                (StagedOutput::Package(package), EntryState::Directory(value)) => {
                    package.inventory.validate().map_err(|error| {
                        RuntimeError::new(format!("cannot adopt invalid package: {error}"))
                    })?;
                    value
                }
                (_, EntryState::Missing) => {
                    return Err(RuntimeError::new(
                        "atomic render output commit failed: staged output is missing",
                    ))
                }
                _ => return Err(RuntimeError::new("cannot adopt unsafe staged output")),
            };
            let destination = format!("entry-{:06}", self.next_source);
            self.next_source += 1;
            descriptor
                .rename_bound_to(source, identity, &self.descriptor, &destination)
                .map_err(|failure| failure.error)?;
            output.rebase_source(self.directory.path().join(destination));
            self.outputs.push(output);
        }
        self.stale.extend(stale);
        drop(descriptor);
        drop(directory);
        Ok(())
    }

    pub(in crate::executor) fn commit(
        self,
        locks: &OutputLocks,
        deadline: Instant,
    ) -> Result<(), PublicationFailure> {
        deadline::ensure(deadline).map_err(PublicationFailure::before_commit)?;
        let stale = stale::enumerate(&self.stale, &self.outputs)
            .map_err(PublicationFailure::before_commit)?;
        let parent = super::common_parent_from_outputs(&self.outputs)
            .map_err(PublicationFailure::before_commit)?;
        let output = locks
            .directory_until(parent, deadline)
            .map_err(PublicationFailure::before_commit)?;
        let result = commit::apply_locked(
            commit::CommitContext::new(
                self.directory.path(),
                &self.descriptor,
                &output,
                &self.outputs,
                &stale,
            ),
            deadline,
        );
        self.finish(result, &output, deadline)
    }

    fn finish(
        self,
        result: Result<(), commit::CommitFailure>,
        output: &Directory,
        deadline: Instant,
    ) -> Result<(), PublicationFailure> {
        let (primary, committed) = match result {
            Ok(()) => (None, true),
            Err(failure) if failure.preserve_staging() => {
                let committed = failure.crossed_commit();
                let error = failure.into_error();
                let path = self.directory.keep();
                return Err(PublicationFailure {
                    error: error
                        .context(&format!("recovery staging preserved at {}", path.display())),
                    committed,
                });
            }
            Err(failure) => (Some(failure.into_error()), false),
        };
        self.cleanup(primary, committed, output, deadline)
    }

    fn cleanup(
        self,
        primary: Option<RuntimeError>,
        committed: bool,
        output: &Directory,
        deadline: Instant,
    ) -> Result<(), PublicationFailure> {
        let staging = self.directory.keep();
        let result = cleanup_stage(&staging, self.descriptor, output, deadline);
        match (primary, result) {
            (None, Ok(())) => Ok(()),
            (Some(error), Ok(())) => Err(PublicationFailure { error, committed }),
            (None, Err(error)) => Err(PublicationFailure { error, committed }),
            (Some(error), Err(cleanup)) => Err(PublicationFailure {
                error: RuntimeError {
                    kind: error.kind,
                    message: format!("{error}; staging cleanup failed: {cleanup}"),
                },
                committed,
            }),
        }
    }
}

impl PublicationFailure {
    pub(in crate::executor) fn before_commit(error: RuntimeError) -> Self {
        Self {
            error,
            committed: false,
        }
    }

    pub(in crate::executor) fn committed(&self) -> bool {
        self.committed
    }

    pub(in crate::executor) fn into_error(self) -> RuntimeError {
        self.error
    }
}

fn cleanup_stage(
    staging: &Path,
    descriptor: Directory,
    output: &Directory,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let name = staging
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RuntimeError::new("publication staging name must be valid UTF-8"))?;
    match descriptor.state(journal::JOURNAL_NAME)? {
        EntryState::Regular(_) => recovery::recover_bound_stage(output, name, descriptor, deadline),
        EntryState::Missing => recovery::discard_bound_stage(output, name, &descriptor, deadline),
        EntryState::Directory(_) => Err(RuntimeError::new(
            "publication journal must be a regular file",
        )),
    }
}

fn component(path: &Path) -> Result<&str, RuntimeError> {
    path.file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RuntimeError::new("staged output name must be valid UTF-8"))
}

#[cfg(test)]
#[path = "publication/tests.rs"]
mod tests;
