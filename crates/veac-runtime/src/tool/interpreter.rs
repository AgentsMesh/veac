use std::path::{Path, PathBuf};

use crate::RuntimeError;

pub(super) fn validate(bytes: &[u8]) -> Result<(), RuntimeError> {
    if !bytes.starts_with(b"#!") {
        return Ok(());
    }
    let line = bytes[2..]
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    let line = std::str::from_utf8(line)
        .map_err(|_| RuntimeError::new("executable shebang must be UTF-8"))?;
    let interpreter = line
        .split_whitespace()
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| RuntimeError::new("executable shebang has no interpreter"))?;
    if !interpreter.is_absolute() || interpreter == Path::new("/usr/bin/env") {
        return Err(RuntimeError::new(
            "executable shebang must use a fixed absolute interpreter",
        ));
    }
    fixed_path(&interpreter)
}

#[cfg(unix)]
fn fixed_path(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;

    let target = std::fs::canonicalize(path).map_err(|error| {
        RuntimeError::new(format!("cannot resolve executable interpreter: {error}"))
    })?;
    let metadata = std::fs::symlink_metadata(&target).map_err(interpreter_error)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(RuntimeError::new(
            "executable interpreter must resolve to an executable regular file",
        ));
    }
    reject_writable(&target, &metadata)?;
    let mut ancestor = path.parent();
    while let Some(directory) = ancestor {
        let resolved = std::fs::canonicalize(directory).map_err(interpreter_error)?;
        let metadata = std::fs::metadata(&resolved).map_err(interpreter_error)?;
        if !metadata.is_dir() {
            return Err(RuntimeError::new(
                "executable interpreter parent is not a directory",
            ));
        }
        reject_writable(&resolved, &metadata)?;
        ancestor = directory.parent();
    }
    Ok(())
}

#[cfg(unix)]
fn reject_writable(path: &Path, metadata: &std::fs::Metadata) -> Result<(), RuntimeError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let mode = metadata.permissions().mode();
    let euid = rustix::process::geteuid().as_raw();
    let owned = euid != 0 && metadata.uid() == euid;
    if mode & 0o022 != 0 || (owned && mode & 0o200 != 0) {
        return Err(RuntimeError::new(format!(
            "executable interpreter path is writable: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn fixed_path(_path: &Path) -> Result<(), RuntimeError> {
    Err(RuntimeError::new(
        "script executables are unsupported on this platform",
    ))
}

pub(super) fn interpreter_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot validate executable interpreter: {error}"))
}
