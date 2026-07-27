use std::path::{Path, PathBuf};

use super::super::directory::Directory;
use super::super::StagedFile;

pub(in crate::executor) struct CommitContext<'a> {
    pub(super) staging: &'a Path,
    pub(super) stage: &'a Directory,
    pub(super) output: &'a Directory,
    pub(super) files: &'a [StagedFile],
    pub(super) stale: &'a [PathBuf],
    pub(super) allow_empty: bool,
}

impl<'a> CommitContext<'a> {
    pub(in crate::executor) fn new(
        staging: &'a Path,
        stage: &'a Directory,
        output: &'a Directory,
        files: &'a [StagedFile],
        stale: &'a [PathBuf],
        allow_empty: bool,
    ) -> Self {
        Self {
            staging,
            stage,
            output,
            files,
            stale,
            allow_empty,
        }
    }
}
