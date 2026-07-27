use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use veac_ir::MediaIdentity;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, OwnedStagedFile};

mod project;
mod write;
pub(super) use project::verify_digest_file;
pub(crate) use project::write_project;

pub(super) fn safe_relative_path(value: &str) -> ArtifactResult<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || value.contains(':')
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "package path must be a portable relative path",
        ));
    }
    Ok(path.to_owned())
}

pub(crate) fn verify_file(path: &Path, identity: &MediaIdentity) -> ArtifactResult<()> {
    require_regular_file(path)?;
    let actual = hash_file(path)?;
    if actual == identity.digest {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "bound media does not match its pinned identity",
        ))
    }
}

pub(crate) fn copy_verified(
    source: &Path,
    root: &Path,
    packaged_path: &str,
    identity: &MediaIdentity,
) -> ArtifactResult<()> {
    let destination = prepare_output_file(root, packaged_path)?;
    if existing_regular_file(&destination)? {
        return verify_file(&destination, identity);
    }
    let mut staged = OwnedStagedFile::new_in(destination.parent().unwrap_or(Path::new(".")))?;
    let result = (|| {
        let mut input = File::open(source)?;
        let copied_size = std::io::copy(&mut input, staged.file_mut())?;
        staged.file_mut().flush()?;
        staged.file_mut().sync_all()?;
        let sealed = staged.seal()?;
        if sealed.sha256 != identity.digest || sealed.size_bytes != copied_size {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "packaged media does not match its pinned identity",
            ));
        }
        Ok(())
    })();
    if let Err(error) = result {
        return Err(staged.discard_after(error));
    }
    staged.persist_noclobber(&destination)?;
    Ok(())
}

pub(super) fn verified_package_file(root: &Path, packaged_path: &str) -> ArtifactResult<PathBuf> {
    let relative = safe_relative_path(packaged_path)?;
    verify_directory(root)?;
    let components: Vec<_> = relative.components().collect();
    let mut current = root.to_path_buf();
    for component in &components[..components.len() - 1] {
        current.push(component.as_os_str());
        verify_directory(&current)?;
    }
    current.push(components.last().unwrap().as_os_str());
    require_regular_file(&current)?;
    Ok(current)
}

pub(crate) fn write_manifest(root: &Path, bytes: &[u8]) -> ArtifactResult<()> {
    let destination = prepare_output_file(root, "package.json")?;
    existing_regular_file(&destination)?;
    write::bytes(&destination, bytes, write::Publish::Replace, |_| Ok(()))
}

fn prepare_output_file(root: &Path, packaged_path: &str) -> ArtifactResult<PathBuf> {
    let relative = safe_relative_path(packaged_path)?;
    create_root(root)?;
    let components: Vec<_> = relative.components().collect();
    let mut current = root.to_path_buf();
    for component in &components[..components.len() - 1] {
        current.push(component.as_os_str());
        create_or_verify_directory(&current)?;
    }
    current.push(components.last().unwrap().as_os_str());
    Ok(current)
}

fn create_root(root: &Path) -> ArtifactResult<()> {
    if let Err(error) = fs::create_dir_all(root) {
        return Err(error.into());
    }
    verify_directory(root)
}

fn create_or_verify_directory(path: &Path) -> ArtifactResult<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => verify_directory(path),
        Err(error) => Err(error.into()),
    }
}

fn verify_directory(path: &Path) -> ArtifactResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return unsafe_path("package path contains a symlink or non-directory component");
    }
    Ok(())
}

fn existing_regular_file(path: &Path) -> ArtifactResult<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            unsafe_path("package output must be a regular non-symlink file")
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn require_regular_file(path: &Path) -> ArtifactResult<()> {
    if existing_regular_file(path)? {
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "package file is missing").into())
    }
}

fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

fn hash_file(path: &Path) -> ArtifactResult<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex(digest.finalize()))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
