use std::fs::File;
use std::io::Write;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{fchmod, fstat, fsync, statat, unlinkat, AtFlags, FileType, Mode};

use super::path::{self, Identity, Parent};
use crate::error::{CliError, CliResult};

mod create;
mod rename;
mod verify;

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

    pub(super) fn publish(self, parent: &Parent, label: &Path) -> CliResult {
        self.publish_with(parent, label, rename::system)
    }

    fn publish_with(
        mut self,
        parent: &Parent,
        label: &Path,
        renamer: rename::Renamer,
    ) -> CliResult {
        self.require_path(label)?;
        verify::content(&mut self.file, &self.expected, label)?;
        cli_try!(fchmod(&self.file, self.mode), |error| write_error(
            label,
            "preserve source mode",
            error
        ));
        cli_try!(self.file.sync_all(), |error| io_error(
            label,
            "sync source mode",
            error
        ));
        self.require_path(label)?;
        verify::content(&mut self.file, &self.expected, label)?;
        self.require_path(label)?;
        if let Err(error) = renamer(self.directory, &self.name, &parent.descriptor, &parent.name) {
            let failure = write_error(label, "replace source", error);
            if rename::looks_committed(self.directory, &self.name, parent, self.identity, label) {
                self.present = false;
                return Err(committed(failure));
            }
            return Err(failure);
        }
        self.present = false;
        if path::path_identity(parent, label).map_err(committed)? != self.identity {
            return Err(committed(CliError::new(
                "SOURCE_CHANGED",
                "published source identity changed",
            )));
        }
        cli_try!(fsync(&parent.descriptor), |error| committed(write_error(
            label,
            "sync source directory",
            error
        )));
        Ok(())
    }

    fn require_path(&self, label: &Path) -> CliResult {
        let opened = cli_try!(fstat(&self.file), |error| write_error(
            label,
            "inspect opened stage",
            error
        ));
        let stat = statat(self.directory, &self.name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| write_error(label, "inspect stage path", error))?;
        if path::identity(&stat) == self.identity
            && path::identity(&opened) == self.identity
            && FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile
            && stat.st_nlink == 1
            && opened.st_nlink == 1
        {
            Ok(())
        } else {
            Err(CliError::new("WRITE_FAILED", "staged source path changed"))
        }
    }

    fn discard(&mut self) -> CliResult {
        if !self.present {
            return Ok(());
        }
        let label = Path::new("source edit staging");
        self.require_path(label)?;
        unlinkat(self.directory, &self.name, AtFlags::empty())
            .map_err(|error| write_error(label, "remove stage", error))?;
        self.present = false;
        Ok(())
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

fn committed(error: CliError) -> CliError {
    CliError::new(
        "WRITE_COMMIT_UNCERTAIN",
        format!("{error}; source replacement committed"),
    )
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
