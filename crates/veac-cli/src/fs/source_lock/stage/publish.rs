use std::fs::File;
use std::path::Path;

use rustix::fs::{fchmod, fstat, fsync, openat, statat, unlinkat, AtFlags, Mode, OFlags};

use super::super::path::{self, Identity, Parent, Target};
use super::verify::ExpectedContent;
use super::{rename, write_error, Staged};
use crate::error::{CliError, CliResult};

#[derive(Clone)]
pub(in crate::fs::source_lock) struct ExpectedTarget {
    identity: Identity,
    mode: Mode,
    content: ExpectedContent,
}

impl ExpectedTarget {
    pub(in crate::fs::source_lock) fn from_target(target: &Target) -> Self {
        Self {
            identity: target.identity,
            mode: target.mode,
            content: ExpectedContent::new(&target.bytes),
        }
    }
}

impl Staged<'_> {
    pub(in crate::fs::source_lock) fn expected_target(&self) -> ExpectedTarget {
        ExpectedTarget {
            identity: self.identity,
            mode: self.mode,
            content: self.expected.clone(),
        }
    }

    pub(in crate::fs::source_lock) fn publish(
        self,
        parent: &Parent,
        expected: &ExpectedTarget,
        label: &Path,
    ) -> CliResult {
        self.publish_with(parent, expected, label, rename::system)
    }

    pub(super) fn publish_with(
        mut self,
        parent: &Parent,
        expected: &ExpectedTarget,
        label: &Path,
        renamer: rename::Renamer,
    ) -> CliResult {
        self.prepare(label)?;
        if let Err(error) = renamer(self.directory, &self.name, &parent.descriptor, &parent.name) {
            let failure = write_error(label, "exchange source", error);
            if rename::replacement_visible(parent, self.identity) {
                self.present = false;
                return Err(committed(failure));
            }
            return Err(failure);
        }
        self.present = false;
        if !self.displaced_matches(expected, label) {
            return self.restore(parent, label, renamer);
        }
        if path::path_identity(parent, label).map_err(committed)? != self.identity {
            return Err(committed(changed(label, "published target changed")));
        }
        unlinkat(self.directory, &self.name, AtFlags::empty())
            .map_err(|error| committed(write_error(label, "remove displaced source", error)))?;
        if path::path_identity(parent, label).map_err(committed)? != self.identity {
            return Err(committed(changed(label, "published target changed")));
        }
        cli_try!(fsync(&parent.descriptor), |error| committed(write_error(
            label,
            "sync source directory",
            error
        )));
        Ok(())
    }

    fn prepare(&mut self, label: &Path) -> CliResult {
        self.require_path(label)?;
        super::verify::content(&mut self.file, &self.expected, label)?;
        cli_try!(fchmod(&self.file, self.mode), |error| write_error(
            label,
            "preserve source mode",
            error
        ));
        cli_try!(self.file.sync_all(), |error| super::io_error(
            label,
            "sync source mode",
            error
        ));
        self.require_path(label)?;
        super::verify::content(&mut self.file, &self.expected, label)?;
        self.require_path(label)
    }

    fn displaced_matches(&self, expected: &ExpectedTarget, label: &Path) -> bool {
        let Ok(descriptor) = openat(
            self.directory,
            &self.name,
            OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) else {
            return false;
        };
        let Ok(stat) = fstat(&descriptor) else {
            return false;
        };
        let mut file = File::from(descriptor);
        path::identity(&stat) == expected.identity
            && Mode::from_raw_mode(stat.st_mode) == expected.mode
            && super::verify::content(&mut file, &expected.content, label).is_ok()
            && matches!(
                statat(self.directory, &self.name, AtFlags::SYMLINK_NOFOLLOW),
                Ok(current) if path::identity(&current) == expected.identity
            )
    }

    fn restore(mut self, parent: &Parent, label: &Path, renamer: rename::Renamer) -> CliResult {
        if let Err(error) = renamer(self.directory, &self.name, &parent.descriptor, &parent.name) {
            return Err(committed(write_error(
                label,
                "restore changed source",
                error,
            )));
        }
        self.present = true;
        self.require_path(label)
            .map_err(committed)
            .and_then(|()| Err(changed(label, "target changed at publication boundary")))
    }

    pub(super) fn require_path(&self, label: &Path) -> CliResult {
        let opened = cli_try!(fstat(&self.file), |error| write_error(
            label,
            "inspect opened stage",
            error
        ));
        let stat = statat(self.directory, &self.name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| write_error(label, "inspect stage path", error))?;
        if path::identity(&stat) == self.identity
            && path::identity(&opened) == self.identity
            && rustix::fs::FileType::from_raw_mode(stat.st_mode)
                == rustix::fs::FileType::RegularFile
            && stat.st_nlink == 1
            && opened.st_nlink == 1
        {
            Ok(())
        } else {
            Err(CliError::new("WRITE_FAILED", "staged source path changed"))
        }
    }

    pub(super) fn discard(&mut self) -> CliResult {
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

fn changed(label: &Path, message: &str) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!(
            "source {} changed during publication: {message}",
            label.display()
        ),
    )
}

pub(super) fn committed(error: CliError) -> CliError {
    CliError::new(
        "WRITE_COMMIT_UNCERTAIN",
        format!("{error}; source replacement may be visible"),
    )
}
