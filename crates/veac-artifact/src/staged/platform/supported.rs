use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};

use rustix::fs::{fstat, openat, FileType, Mode, OFlags};

use crate::{ArtifactResult, StagedContent};

mod checks;
mod content;
mod publication;

use checks::*;

const PAYLOAD: &str = "payload";

#[derive(Debug, Clone, Copy)]
pub(in crate::staged) enum PublishMode {
    Replace,
    NoClobber,
}

#[derive(Debug)]
pub(in crate::staged) struct Stage {
    file: Option<File>,
    parent: File,
    directory: File,
    directory_name: OsString,
    path: PathBuf,
    payload_present: bool,
    directory_present: bool,
}

impl Stage {
    pub(in crate::staged) fn new(parent: &Path) -> ArtifactResult<Self> {
        let parent_fd = open_directory(parent)?;
        let (directory_name, directory) = create_private_directory(&parent_fd)?;
        let path = parent.join(&directory_name).join(PAYLOAD);
        let mut stage = Self {
            file: None,
            parent: parent_fd,
            directory,
            directory_name,
            path,
            payload_present: false,
            directory_present: true,
        };
        stage.verify_directory()?;
        let payload = openat(
            &stage.directory,
            PAYLOAD,
            file_flags() | OFlags::CREATE | OFlags::EXCL | OFlags::RDWR,
            Mode::RUSR | Mode::WUSR,
        )
        .map_err(|error| io_error("create payload", error))?;
        stage.file = Some(File::from(payload));
        stage.payload_present = true;
        Ok(stage)
    }

    pub(in crate::staged) fn path(&self) -> &Path {
        &self.path
    }

    pub(in crate::staged) fn file_mut(&mut self) -> &mut File {
        self.file.as_mut().expect("staged payload is live")
    }

    pub(in crate::staged) fn verify_payload(&self) -> ArtifactResult<()> {
        let current = File::from(
            openat(&self.directory, PAYLOAD, file_flags(), Mode::empty())
                .map_err(|error| unsafe_io("reopen staged payload", error))?,
        );
        verify_owned_regular(self.file(), &current)
    }

    pub(in crate::staged) fn content_while(
        &self,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<StagedContent> {
        content::fingerprint_while(self.file(), guard)
    }

    fn file(&self) -> &File {
        self.file.as_ref().expect("staged payload is live")
    }

    fn verify_destination_parent(&self, destination: &Path) -> ArtifactResult<()> {
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let current = open_directory(parent)?;
        verify_identity(&self.parent, &current, FileType::Directory)
    }

    fn verify_directory(&self) -> ArtifactResult<()> {
        let current = File::from(
            openat(
                &self.parent,
                &self.directory_name,
                directory_flags(),
                Mode::empty(),
            )
            .map_err(|error| unsafe_io("reopen private directory", error))?,
        );
        verify_identity(&self.directory, &current, FileType::Directory)?;
        let stat = fstat(&current).map_err(|error| io_error("inspect private directory", error))?;
        if stat.st_mode & 0o077 != 0 {
            return Err(unsafe_path("staging directory is not owner-private"));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "supported/tests.rs"]
mod tests;
