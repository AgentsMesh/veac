use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use rustix::fs::{fstat, unlinkat, AtFlags, Dir, FileType};

use super::{failure, Directory};
use crate::RuntimeError;

impl Directory {
    pub(in crate::executor) fn entries(
        &self,
        maximum: usize,
    ) -> Result<Vec<OsString>, RuntimeError> {
        let mut names = Vec::new();
        let entries = match Dir::read_from(&self.descriptor) {
            Ok(entries) => entries,
            Err(error) => return Err(failure(&self.path, "read transaction directory", error)),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Err(failure(&self.path, "read transaction directory", error)),
            };
            let name = entry.file_name().to_bytes();
            if name == b"." || name == b".." {
                continue;
            }
            if names.len() == maximum {
                return Err(RuntimeError::resource_limit(
                    "transaction directory entry count exceeds its limit",
                ));
            }
            names.push(OsString::from_vec(name.to_vec()));
        }
        names.sort();
        Ok(names)
    }

    pub(in crate::executor) fn remove_child(
        &self,
        name: &str,
        child: &Directory,
    ) -> Result<(), RuntimeError> {
        let expected = match fstat(&child.descriptor) {
            Ok(expected) => expected,
            Err(error) => return Err(failure(&child.path, "inspect transaction directory", error)),
        };
        let current = self.child(name)?;
        let actual = match fstat(&current.descriptor) {
            Ok(actual) => actual,
            Err(error) => {
                return Err(failure(
                    &current.path,
                    "inspect transaction directory",
                    error,
                ))
            }
        };
        if FileType::from_raw_mode(expected.st_mode) != FileType::Directory
            || expected.st_dev != actual.st_dev
            || expected.st_ino != actual.st_ino
        {
            return Err(RuntimeError::new(
                "transaction directory changed identity before removal",
            ));
        }
        match unlinkat(&self.descriptor, name, AtFlags::REMOVEDIR) {
            Ok(()) => Ok(()),
            Err(error) => Err(failure(&self.path, "remove transaction directory", error)),
        }
    }
}

#[cfg(test)]
#[path = "list/tests.rs"]
mod tests;
