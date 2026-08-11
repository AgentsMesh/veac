use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{
    canonical_json, publish_evidence_bundle, sha256_hex, BundleError, EvidenceBundleManifestV1,
    EvidenceOutcome, PreparedEvidenceBundle,
};

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_BUNDLE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct EvidenceCache {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedEvidenceBundle {
    pub root: PathBuf,
    pub manifest: EvidenceBundleManifestV1,
}

impl EvidenceCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn lookup(&self, key: &str) -> Result<Option<CachedEvidenceBundle>, BundleError> {
        validate_key(key)?;
        let root = self.path(key);
        let manifest_path = root.join("bundle.json");
        if !manifest_path.exists() {
            return Ok(None);
        }
        let bytes = bounded_read(&manifest_path, MAX_MANIFEST_BYTES)?;
        let manifest: EvidenceBundleManifestV1 = serde_json::from_slice(&bytes)
            .map_err(|error| BundleError::Encode(error.to_string()))?;
        if canonical_json(&manifest)? != bytes || manifest.cache_key != key {
            return Err(BundleError::InvalidContract(
                "cached evidence manifest is non-canonical or misaddressed".into(),
            ));
        }
        verify_artifacts(&root, &manifest)?;
        Ok(Some(CachedEvidenceBundle { root, manifest }))
    }

    pub fn publish(
        &self,
        bundle: &PreparedEvidenceBundle,
    ) -> Result<CachedEvidenceBundle, BundleError> {
        if bundle.manifest.outcome == EvidenceOutcome::Error {
            return Err(BundleError::InvalidContract(
                "evidence with observation errors is not cacheable".into(),
            ));
        }
        let key = &bundle.manifest.cache_key;
        if let Some(found) = self.lookup(key)? {
            return same_manifest(found, &bundle.manifest);
        }
        let destination = self.path(key);
        match publish_evidence_bundle(bundle, &destination) {
            Ok(_) => self.lookup(key)?.ok_or_else(|| {
                BundleError::InvalidContract("published evidence cache entry is missing".into())
            }),
            Err(error) if destination.exists() => {
                let found = self.lookup(key)?.ok_or(error)?;
                same_manifest(found, &bundle.manifest)
            }
            Err(error) => Err(error),
        }
    }

    fn path(&self, key: &str) -> PathBuf {
        self.root.join("sha256").join(&key[..2]).join(&key[2..])
    }
}

fn same_manifest(
    found: CachedEvidenceBundle,
    expected: &EvidenceBundleManifestV1,
) -> Result<CachedEvidenceBundle, BundleError> {
    if found.manifest == *expected {
        Ok(found)
    } else {
        Err(BundleError::InvalidContract(
            "evidence cache key resolved to different contents".into(),
        ))
    }
}

fn verify_artifacts(root: &Path, manifest: &EvidenceBundleManifestV1) -> Result<(), BundleError> {
    let canonical_root = fs::canonicalize(root)?;
    let mut total = 0_u64;
    let mut previous = None;
    for record in &manifest.artifacts {
        safe_relative(&record.path)?;
        if previous.is_some_and(|value: &str| value >= record.path.as_str()) {
            return Err(BundleError::InvalidContract(
                "cached evidence artifact records are not strictly sorted".into(),
            ));
        }
        previous = Some(&record.path);
        total = total
            .checked_add(record.size_bytes)
            .filter(|value| *value <= MAX_BUNDLE_BYTES)
            .ok_or_else(|| BundleError::InvalidContract("evidence cache exceeds budget".into()))?;
        let path = root.join(&record.path);
        let canonical = fs::canonicalize(&path)?;
        if !canonical.starts_with(&canonical_root) {
            return Err(BundleError::InvalidContract(
                "cached evidence artifact escapes its entry".into(),
            ));
        }
        let bytes = bounded_read(&path, record.size_bytes)?;
        if bytes.len() as u64 != record.size_bytes || sha256_hex(&bytes) != record.sha256 {
            return Err(BundleError::InvalidContract(
                "cached evidence artifact failed content verification".into(),
            ));
        }
    }
    Ok(())
}

fn bounded_read(path: &Path, maximum: u64) -> Result<Vec<u8>, BundleError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(BundleError::InvalidContract(
            "evidence cache file exceeds its declared budget".into(),
        ));
    }
    Ok(fs::read(path)?)
}

fn safe_relative(value: &str) -> Result<(), BundleError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || !path
            .components()
            .all(|value| matches!(value, Component::Normal(_)))
    {
        return Err(BundleError::UnsafePath(value.into()));
    }
    Ok(())
}

fn validate_key(value: &str) -> Result<(), BundleError> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    valid
        .then_some(())
        .ok_or_else(|| BundleError::InvalidContract("evidence cache key is not SHA-256".into()))
}
