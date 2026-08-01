use std::path::PathBuf;
use std::time::Instant;

use crate::RuntimeError;

mod commit;
pub(super) mod directory;
mod ffmpeg;
#[cfg(test)]
#[path = "staging/tests/finish.rs"]
mod finish;
mod journal;
mod model;
mod ownership;
mod package;
mod perform;
mod publication;
mod recovery;
mod stale;
mod target;
mod write;

pub(super) use model::{StagedFile, StagedOutput, StagedPackage, StagedTask};
pub(in crate::executor) use package::inspect_hls as inspect_package;
pub(super) use perform::perform;
pub(super) use publication::{Publication, PublicationFailure};
pub(super) use stale::StaleFamily;
pub(super) use target::{common_parent, common_parent_from_outputs};

#[cfg(test)]
mod tests;

pub(super) fn recover(
    locks: &super::locking::OutputLocks,
    parents: &[PathBuf],
    deadline: Instant,
) -> Result<(), RuntimeError> {
    recovery::recover_until(locks, parents, deadline)
}
