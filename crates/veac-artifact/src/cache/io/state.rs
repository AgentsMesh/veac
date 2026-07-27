use std::ffi::{OsStr, OsString};

use super::common::corrupt;
use super::directory::BoundDirectory;
use super::entry::BoundFile;
use crate::ArtifactResult;

pub(super) const DESCRIPTOR: &str = "descriptor.json";
pub(super) const PAYLOAD: &str = "payload.bin";
pub(super) const RECORD: &str = "record.json";
pub(super) const SEALED: &str = "sealed";

pub(super) fn marker(directory: &BoundDirectory) -> ArtifactResult<Option<BoundFile>> {
    directory.verify()?;
    let entries = directory.entries(5)?;
    let sealed = entries.iter().any(|name| name == OsStr::new(SEALED));
    if !sealed {
        if entries.iter().all(|name| is_data_name(name)) {
            directory.verify()?;
            return Ok(None);
        }
        return corrupt("unsealed cache directory contains unexpected entries");
    }
    if entries != committed_entries() {
        return corrupt("sealed cache directory is incomplete or contains unexpected entries");
    }
    let marker = BoundFile::open(directory.file(), SEALED.as_ref())?;
    if marker.size()? != 0 {
        return corrupt("cache commit marker must be empty");
    }
    directory.verify()?;
    marker.verify_while(directory.file(), || true)?;
    Ok(Some(marker))
}

pub(super) fn data_entries() -> [OsString; 3] {
    [DESCRIPTOR.into(), PAYLOAD.into(), RECORD.into()]
}

pub(super) fn committed_entries() -> [OsString; 4] {
    [
        DESCRIPTOR.into(),
        PAYLOAD.into(),
        RECORD.into(),
        SEALED.into(),
    ]
}

pub(super) fn is_data_name(name: &OsStr) -> bool {
    [DESCRIPTOR, PAYLOAD, RECORD]
        .iter()
        .any(|expected| name == OsStr::new(expected))
}
