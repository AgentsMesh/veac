use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use rustix::fs::fstat;
use veac_artifact::ContentDigest;

use crate::error::{CliError, CliResult};

#[derive(Clone)]
pub(super) struct ExpectedContent {
    digest: String,
    size: usize,
}

impl ExpectedContent {
    pub(super) fn new(value: &[u8]) -> Self {
        Self {
            digest: ContentDigest::sha256(value).value,
            size: value.len(),
        }
    }
}

pub(super) fn content(file: &mut File, expected: &ExpectedContent, label: &Path) -> CliResult {
    let before = cli_try!(fstat(&*file), |error| failure(label, error.to_string()));
    file.seek(SeekFrom::Start(0))
        .map_err(|error| failure(label, error.to_string()))?;
    let mut actual = Vec::with_capacity(expected.size);
    file.take(expected.size.saturating_add(1) as u64)
        .read_to_end(&mut actual)
        .map_err(|error| failure(label, error.to_string()))?;
    let after = cli_try!(fstat(&*file), |error| failure(label, error.to_string()));
    let stable = before.st_dev == after.st_dev
        && before.st_ino == after.st_ino
        && before.st_mode == after.st_mode
        && before.st_nlink == after.st_nlink
        && before.st_size == after.st_size
        && before.st_mtime == after.st_mtime
        && before.st_mtime_nsec == after.st_mtime_nsec
        && before.st_ctime == after.st_ctime
        && before.st_ctime_nsec == after.st_ctime_nsec;
    let digest = ContentDigest::sha256(&actual);
    if stable && actual.len() == expected.size && digest.value == expected.digest {
        Ok(())
    } else {
        Err(CliError::new(
            "WRITE_FAILED",
            format!(
                "staged source {} changed before publication",
                label.display()
            ),
        ))
    }
}

fn failure(label: &Path, message: String) -> CliError {
    CliError::new(
        "WRITE_FAILED",
        format!("cannot verify staged source {}: {message}", label.display()),
    )
}

#[cfg(test)]
#[path = "verify/tests.rs"]
mod tests;
