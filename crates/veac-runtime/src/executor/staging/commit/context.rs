use std::path::{Path, PathBuf};

use super::super::directory::Directory;
use super::super::StagedOutput;

pub(in crate::executor) struct CommitContext<'a> {
    pub(super) staging: &'a Path,
    pub(super) stage: &'a Directory,
    pub(super) output: &'a Directory,
    pub(super) outputs: &'a [StagedOutput],
    pub(super) stale: &'a [PathBuf],
}

impl<'a> CommitContext<'a> {
    pub(in crate::executor) fn new(
        staging: &'a Path,
        stage: &'a Directory,
        output: &'a Directory,
        outputs: &'a [StagedOutput],
        stale: &'a [PathBuf],
    ) -> Self {
        Self {
            staging,
            stage,
            output,
            outputs,
            stale,
        }
    }
}
