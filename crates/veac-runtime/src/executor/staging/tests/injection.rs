use std::time::Instant;

use super::super::commit::{
    self, CommitContext, CommitFailure, CommitObserver, RollbackOperations,
};
use super::super::{common_parent_from_outputs, Publication, PublicationFailure, StagedTask};
use crate::RuntimeError;

struct PassiveObserver;

impl CommitObserver for PassiveObserver {}

pub(super) fn apply_with<O: RollbackOperations>(
    context: CommitContext<'_>,
    guard: impl FnMut() -> bool,
    rollback: &O,
) -> Result<(), CommitFailure> {
    commit::apply_observed_until(context, guard, rollback, &PassiveObserver, None)
}

pub(super) fn apply_observed<R: RollbackOperations, O: CommitObserver>(
    context: CommitContext<'_>,
    guard: impl FnMut() -> bool,
    rollback: &R,
    observer: &O,
) -> Result<(), CommitFailure> {
    commit::apply_observed_until(context, guard, rollback, observer, None)
}

impl StagedTask {
    pub(super) fn commit(
        self,
        locks: &crate::executor::locking::OutputLocks,
        deadline: Instant,
    ) -> Result<(), RuntimeError> {
        let parent = common_parent_from_outputs(&self.outputs)?.to_owned();
        let mut publication = Publication::new(&parent)?;
        publication.adopt(self)?;
        publication
            .commit(locks, deadline)
            .map_err(PublicationFailure::into_error)
    }
}
