use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{canonical_json, sha256_hex, BundleError, PreparedEvidenceBundle};

pub fn publish_evidence_bundle(
    bundle: &PreparedEvidenceBundle,
    destination: &Path,
) -> Result<PathBuf, BundleError> {
    verify_prepared(bundle)?;
    if destination.exists() {
        return Err(BundleError::InvalidContract(
            "evidence bundle destination already exists".into(),
        ));
    }
    let parent = destination.parent().ok_or_else(|| {
        BundleError::InvalidContract("evidence bundle destination has no parent".into())
    })?;
    fs::create_dir_all(parent)?;
    let temporary = tempfile::Builder::new()
        .prefix(".veac-evidence-")
        .tempdir_in(parent)?;
    for artifact in &bundle.artifacts {
        write(temporary.path(), &artifact.path, &artifact.bytes)?;
    }
    write(
        temporary.path(),
        "bundle.json",
        &canonical_json(&bundle.manifest)?,
    )?;
    sync_directory(temporary.path())?;
    fs::rename(temporary.path(), destination)?;
    sync_directory(parent)?;
    Ok(destination.to_path_buf())
}

fn verify_prepared(bundle: &PreparedEvidenceBundle) -> Result<(), BundleError> {
    if bundle.manifest.artifacts.len() != bundle.artifacts.len() {
        return Err(BundleError::InvalidContract(
            "bundle artifact manifest length differs from its payloads".into(),
        ));
    }
    for (record, artifact) in bundle.manifest.artifacts.iter().zip(&bundle.artifacts) {
        if record.path != artifact.path
            || record.size_bytes != artifact.bytes.len() as u64
            || record.sha256 != sha256_hex(&artifact.bytes)
        {
            return Err(BundleError::InvalidContract(
                "bundle artifact payload failed manifest verification".into(),
            ));
        }
    }
    Ok(())
}

fn write(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), BundleError> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), BundleError> {
    OpenOptions::new().read(true).open(path)?.sync_all()?;
    Ok(())
}
