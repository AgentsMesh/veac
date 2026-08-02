use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    DeliveryPackageInventory, DeliveryPackageMember, DeliveryPackageNodeType,
    MAX_DELIVERY_PACKAGE_MEMBERS, MAX_DELIVERY_PACKAGE_PATH_BYTES, MAX_RENDER_TASK_OUTPUT_BYTES,
};

use super::super::directory::{Directory, EntryState};
use crate::executor::deadline;
use crate::RuntimeError;

struct Scan {
    members: Vec<DeliveryPackageMember>,
    bytes: u64,
    deadline: Instant,
}

pub(super) fn inventory(
    root: &Directory,
    entrypoint: &Path,
    deadline: Instant,
) -> Result<DeliveryPackageInventory, RuntimeError> {
    let entrypoint = entrypoint
        .to_str()
        .ok_or_else(|| RuntimeError::new("package entrypoint must be valid UTF-8"))?;
    let mut scan = Scan {
        members: Vec::new(),
        bytes: 0,
        deadline,
    };
    walk(root, "", &mut scan)?;
    scan.members
        .sort_by(|left, right| left.path.cmp(&right.path));
    DeliveryPackageInventory::new(entrypoint, scan.members).map_err(artifact_error)
}

fn walk(directory: &Directory, prefix: &str, scan: &mut Scan) -> Result<(), RuntimeError> {
    deadline::ensure(scan.deadline)?;
    let remaining = MAX_DELIVERY_PACKAGE_MEMBERS.saturating_sub(scan.members.len());
    let names = directory.entries(remaining)?;
    if names.is_empty() {
        return Err(RuntimeError::new(
            "delivery package may not contain an empty directory",
        ));
    }
    for name in names {
        deadline::ensure(scan.deadline)?;
        let name = name
            .to_str()
            .ok_or_else(|| RuntimeError::new("package member name must be valid UTF-8"))?;
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        if path.len() > MAX_DELIVERY_PACKAGE_PATH_BYTES
            || scan.members.len() == MAX_DELIVERY_PACKAGE_MEMBERS
        {
            return Err(RuntimeError::resource_limit(
                "delivery package member shape exceeds its budget",
            ));
        }
        match directory.state(name)? {
            EntryState::Regular(identity) => regular(directory, name, path, identity, scan)?,
            EntryState::Directory(_) => {
                scan.members.push(DeliveryPackageMember {
                    path: path.clone(),
                    node_type: DeliveryPackageNodeType::Directory,
                    size_bytes: 0,
                    content: None,
                });
                let child = directory.child(name)?;
                walk(&child, &path, scan)?;
            }
            EntryState::Missing => {
                return Err(RuntimeError::new(
                    "delivery package member disappeared during inventory",
                ))
            }
        }
    }
    Ok(())
}

fn regular(
    directory: &Directory,
    name: &str,
    path: String,
    identity: super::super::directory::EntryIdentity,
    scan: &mut Scan,
) -> Result<(), RuntimeError> {
    let remaining = MAX_RENDER_TASK_OUTPUT_BYTES.saturating_sub(scan.bytes);
    let (content, size_bytes) = directory.hash_regular(name, identity, remaining, scan.deadline)?;
    if size_bytes == 0 {
        return Err(RuntimeError::new(
            "delivery package regular files must be non-empty",
        ));
    }
    scan.bytes += size_bytes;
    scan.members.push(DeliveryPackageMember {
        path,
        node_type: DeliveryPackageNodeType::RegularFile,
        size_bytes,
        content: Some(content),
    });
    Ok(())
}

fn artifact_error(error: veac_artifact::ArtifactError) -> RuntimeError {
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(format!("invalid delivery package: {error}"))
    } else {
        RuntimeError::new(format!("invalid delivery package: {error}"))
    }
}

#[cfg(test)]
#[path = "scanner/tests.rs"]
mod tests;
