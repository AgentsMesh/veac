use std::path::Path;

use crate::RuntimeError;

use super::io_result;
#[cfg(not(unix))]
use super::tool_error;

#[cfg(unix)]
pub(super) fn executable_permissions(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;
    io_result(
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o500)),
        "protect executable snapshot",
    )
}

#[cfg(unix)]
pub(super) fn master_permissions(path: &Path, directory: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;
    let result = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o400))
        .and_then(|()| std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o500)));
    io_result(result, "protect executable master")
}

#[cfg(not(unix))]
pub(super) fn master_permissions(path: &Path, _directory: &Path) -> Result<(), RuntimeError> {
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| tool_error("inspect executable master", error))?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| tool_error("protect executable master", error))
}

#[cfg(unix)]
pub(super) fn writable_directory(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
pub(super) fn writable_directory(_path: &Path) {}

#[cfg(not(unix))]
pub(super) fn executable_permissions(path: &Path) -> Result<(), RuntimeError> {
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| tool_error("inspect executable snapshot", error))?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| tool_error("protect executable snapshot", error))
}
