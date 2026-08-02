use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::Path;

use rustix::fs::{openat, renameat, renameat_with, unlinkat, AtFlags, Mode, RenameFlags};

use super::{
    file_flags, io_error, publish_error, unsafe_io, unsafe_path, verify_identity,
    verify_owned_regular, PublishMode, Stage, PAYLOAD,
};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, StagedContent};

impl Stage {
    pub(in crate::staged) fn publish_while(
        &mut self,
        destination: &Path,
        mode: PublishMode,
        expected: &StagedContent,
        after_verify: impl FnOnce(&Path),
        after_publish: impl FnOnce(&Path),
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        checked(&mut guard)?;
        let name = self.destination_name(destination)?;
        checked(&mut guard)?;
        step(&mut guard, || self.verify_payload())?;
        self.verify_content_while(expected, &mut guard)?;
        after_verify(&self.path);
        checked(&mut guard)?;
        step(&mut guard, || self.verify_payload())?;
        self.verify_content_while(expected, &mut guard)?;
        checked(&mut guard)?;
        let result = match mode {
            PublishMode::Replace => renameat(&self.directory, PAYLOAD, &self.parent, &name),
            PublishMode::NoClobber => renameat_with(
                &self.directory,
                PAYLOAD,
                &self.parent,
                &name,
                RenameFlags::NOREPLACE,
            ),
        };
        match result {
            Ok(()) => self.payload_present = false,
            Err(error) => {
                checked(&mut guard)?;
                return Err(publish_error(mode, error));
            }
        }
        checked(&mut guard).map_err(ArtifactError::committed)?;
        after_publish(destination);
        checked(&mut guard).map_err(ArtifactError::committed)?;
        committed_step(&mut guard, || self.verify_published(&name))?;
        self.verify_content_while(expected, &mut guard)
            .map_err(ArtifactError::committed)?;
        committed_step(&mut guard, || self.verify_destination_parent(destination))
    }

    pub(in crate::staged) fn finish(&mut self) -> ArtifactResult<()> {
        self.cleanup_directory()
    }

    pub(in crate::staged) fn discard(&mut self) -> ArtifactResult<()> {
        if self.payload_present {
            self.verify_payload_identity()?;
            unlinkat(&self.directory, PAYLOAD, AtFlags::empty())
                .map_err(|error| io_error("remove staged payload", error))?;
            self.payload_present = false;
        }
        self.cleanup_directory()
    }

    pub(in crate::staged) fn into_file(mut self) -> File {
        self.file.take().expect("staged payload is live")
    }

    fn verify_content_while(
        &self,
        expected: &StagedContent,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        if &self.content_while(guard)? != expected {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "staged payload content changed after it was sealed",
            ));
        }
        Ok(())
    }

    fn destination_name(&self, destination: &Path) -> ArtifactResult<OsString> {
        let name = destination
            .file_name()
            .ok_or_else(|| unsafe_path("staged destination has no file name"))?;
        self.verify_destination_parent(destination)?;
        Ok(name.to_owned())
    }

    fn verify_published(&self, name: &OsStr) -> ArtifactResult<()> {
        let current = File::from(
            openat(&self.parent, name, file_flags(), Mode::empty())
                .map_err(|error| unsafe_io("reopen published output", error))?,
        );
        verify_owned_regular(self.file(), &current)
    }

    fn verify_payload_identity(&self) -> ArtifactResult<()> {
        let current = File::from(
            openat(&self.directory, PAYLOAD, file_flags(), Mode::empty())
                .map_err(|error| unsafe_io("reopen staged payload for cleanup", error))?,
        );
        verify_identity(self.file(), &current, rustix::fs::FileType::RegularFile)
    }

    fn cleanup_directory(&mut self) -> ArtifactResult<()> {
        if self.directory_present {
            self.verify_directory()?;
            unlinkat(&self.parent, &self.directory_name, AtFlags::REMOVEDIR)
                .map_err(|error| io_error("remove private directory", error))?;
            self.directory_present = false;
        }
        Ok(())
    }
}

fn step<T>(
    guard: &mut impl FnMut() -> bool,
    operation: impl FnOnce() -> ArtifactResult<T>,
) -> ArtifactResult<T> {
    checked(guard)?;
    let value = operation()?;
    checked(guard)?;
    Ok(value)
}

fn committed_step<T>(
    guard: &mut impl FnMut() -> bool,
    operation: impl FnOnce() -> ArtifactResult<T>,
) -> ArtifactResult<T> {
    step(guard, operation).map_err(ArtifactError::committed)
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::staged::check_guard(guard)
}

impl Drop for Stage {
    fn drop(&mut self) {
        // Destructors cannot surface cleanup errors; explicit callers use `discard`.
        let _ = self.discard();
    }
}

#[cfg(test)]
#[path = "publication/tests.rs"]
mod tests;
