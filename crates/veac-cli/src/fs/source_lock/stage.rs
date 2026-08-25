use std::fs::File;
use std::io::Write;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{fstat, Mode};

use super::path::{self, Identity};
use crate::error::{CliError, CliResult};

mod create;
mod publish;
mod rename;
mod verify;

pub(super) use publish::ExpectedTarget;
use verify::ExpectedContent;

pub(super) struct Staged<'a> {
    directory: &'a OwnedFd,
    file: File,
    identity: Identity,
    expected: ExpectedContent,
    mode: Mode,
    name: std::ffi::OsString,
    present: bool,
}

impl<'a> Staged<'a> {
    pub(super) fn create(
        directory: &'a OwnedFd,
        mode: Mode,
        value: &[u8],
        label: &Path,
    ) -> CliResult<Self> {
        let (name, descriptor) = create::unique(directory, label)?;
        let file = File::from(descriptor);
        let stat = cli_try!(fstat(&file), |error| write_error(
            label,
            "inspect stage",
            error
        ));
        let mut staged = Self {
            directory,
            identity: path::identity(&stat),
            expected: ExpectedContent::new(value),
            file,
            mode,
            name,
            present: true,
        };
        staged.write(value, label)?;
        Ok(staged)
    }

    fn write(&mut self, value: &[u8], label: &Path) -> CliResult {
        cli_try!(
            self.file
                .write_all(value)
                .and_then(|_| self.file.sync_all()),
            |error| io_error(label, "write stage", error)
        );
        verify::content(&mut self.file, &self.expected, label)
    }
}

#[cfg(test)]
#[path = "stage/tests.rs"]
mod tests;

impl Drop for Staged<'_> {
    fn drop(&mut self) {
        let _ = self.discard();
    }
}

fn write_error(label: &Path, operation: &str, error: rustix::io::Errno) -> CliError {
    CliError::new(
        "WRITE_FAILED",
        format!("cannot {operation} {}: {error}", label.display()),
    )
}

fn io_error(label: &Path, operation: &str, error: std::io::Error) -> CliError {
    CliError::new(
        "WRITE_FAILED",
        format!("cannot {operation} {}: {error}", label.display()),
    )
}
