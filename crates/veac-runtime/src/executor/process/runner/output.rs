use std::path::Path;

use crate::RuntimeError;

const MAX_OUTPUT_ENTRIES: u64 = veac_ir::MAX_VIDEO_FRAMES_PER_DELIVERABLE as u64 + 4_096;

pub(super) fn tree_size(root: &Path) -> Result<u64, RuntimeError> {
    let mut total = 0_u64;
    let mut count = 0_u64;
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error(error)),
        };
        for entry in entries {
            let entry = entry.map_err(io_error)?;
            count = count.saturating_add(1);
            if count > MAX_OUTPUT_ENTRIES {
                return Err(RuntimeError::new(
                    "FFmpeg output file count exceeded its limit",
                ));
            }
            let metadata = match entry.path().symlink_metadata() {
                Ok(value) => value,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(io_error(error)),
            };
            if metadata.file_type().is_symlink() {
                return Err(RuntimeError::new(
                    "FFmpeg created a symlink in its staging area",
                ));
            }
            if metadata.is_dir() {
                directories.push(entry.path());
            } else if metadata.is_file() {
                total = total
                    .checked_add(metadata.len())
                    .ok_or_else(|| RuntimeError::new("FFmpeg output byte count overflowed"))?;
            } else {
                return Err(RuntimeError::new(
                    "FFmpeg created a non-regular staging entry",
                ));
            }
        }
    }
    Ok(total)
}

fn io_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(error.to_string())
}

#[cfg(test)]
mod tests;
