use std::time::Instant;

use super::{commit, directory, journal, recovery, StagedTask};
use crate::RuntimeError;

impl StagedTask {
    pub(super) fn finish_commit(
        self,
        result: Result<(), commit::CommitFailure>,
        output: &directory::Directory,
        deadline: Instant,
    ) -> Result<(), RuntimeError> {
        let primary = match result {
            Ok(()) => None,
            Err(failure) if failure.preserve_staging() => {
                let error = failure.into_error();
                let path = self.directory.keep();
                return Err(
                    error.context(&format!("recovery staging preserved at {}", path.display()))
                );
            }
            Err(failure) => Some(failure.into_error()),
        };
        let StagedTask {
            directory,
            descriptor,
            ..
        } = self;
        let staging = directory.keep();
        let cleanup = (|| {
            let name = staging
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| RuntimeError::new("render staging name must be valid UTF-8"))?;
            match descriptor.state(journal::JOURNAL_NAME)? {
                directory::EntryState::Regular(_) => {
                    recovery::recover_bound_stage(output, name, descriptor, deadline)
                }
                directory::EntryState::Missing => {
                    recovery::discard_bound_stage(output, name, &descriptor, deadline)
                }
                directory::EntryState::Directory(_) => {
                    Err(RuntimeError::new("render journal must be a regular file"))
                }
            }
        })();
        match (primary, cleanup) {
            (None, Ok(())) => Ok(()),
            (Some(error), Ok(())) => Err(error),
            (None, Err(cleanup)) => Err(cleanup
                .context("render outputs crossed the commit point but staging cleanup failed")),
            (Some(error), Err(cleanup)) => Err(RuntimeError {
                kind: error.kind,
                message: format!(
                    "{error}; staging cleanup failed and was preserved at {}: {cleanup}",
                    staging.display()
                ),
            }),
        }
    }
}
