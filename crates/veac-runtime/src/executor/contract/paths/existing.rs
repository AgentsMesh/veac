use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::invalid;
use crate::executor::output;
use crate::RuntimeError;

pub(super) fn validate_static(path: &Path) -> Result<(), RuntimeError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            invalid("existing backend output must be a regular non-symlink file")
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(RuntimeError::new(format!(
            "cannot inspect existing backend output {}: {error}",
            path.display()
        ))),
    }
}

pub(super) fn validate_pattern(pattern: &Path) -> Result<(), RuntimeError> {
    output::enumerate_pattern(pattern).map(|_| ())
}
