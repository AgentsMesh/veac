use std::ffi::OsString;

use super::authority::BoundChain;
use super::directory::BoundDirectory;
use super::entry::BoundFile;
use super::state;
use crate::ArtifactResult;

pub(super) fn remove_incomplete(
    directory: BoundDirectory,
) -> ArtifactResult<(BoundChain, OsString)> {
    if state::marker(&directory)?.is_some() {
        return super::common::corrupt("refusing to recover a sealed cache directory");
    }
    let names = directory.entries(4)?;
    let mut files = Vec::with_capacity(names.len());
    for name in names {
        if !state::is_data_name(&name) {
            return super::common::corrupt("unsealed cache directory changed during recovery");
        }
        let file = BoundFile::open(directory.file(), &name)?;
        files.push(file);
    }
    directory.verify()?;
    for file in files {
        directory.verify()?;
        file.unlink(directory.file())?;
    }
    directory.remove_empty_into_parent()
}
