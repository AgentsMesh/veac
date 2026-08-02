use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactResult,
    ContentDigest,
};

mod catalog;
mod file;
mod guard;
mod io;
mod memory;
mod verified;

pub use catalog::ArtifactCatalogQuery;
pub use verified::VerifiedArtifact;

#[cfg(test)]
#[path = "cache/guard_tests.rs"]
mod guard_tests;
#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRecord {
    pub key: ContentDigest,
    pub content: ContentDigest,
    #[schemars(range(max = 9007199254740991u64))]
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CachedArtifact {
    pub record: ArtifactRecord,
    pub descriptor: ArtifactDescriptor,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn get(&self, key: &ContentDigest) -> ArtifactResult<Option<CachedArtifact>> {
        self.get_while(key, || true)
    }

    pub fn get_while(
        &self,
        key: &ContentDigest,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Option<CachedArtifact>> {
        guard::check(&mut guard)?;
        key.validate()?;
        guard::check(&mut guard)?;
        let directory = self.directory(key);
        let Some((descriptor, record, payload)) =
            io::read_while(&self.root, &directory, &mut guard)?
        else {
            return Ok(None);
        };
        guard::check(&mut guard)?;
        let descriptor_key = artifact_key(&descriptor)?;
        guard::check(&mut guard)?;
        if descriptor_key != *key || record.key != *key {
            return Err(ArtifactError::new(
                ArtifactErrorKind::CorruptCache,
                "cached artifact failed identity verification",
            ));
        }
        Ok(Some(CachedArtifact {
            record,
            descriptor,
            payload,
        }))
    }

    pub fn remove(&self, key: &ContentDigest) -> ArtifactResult<bool> {
        self.remove_while(key, || true)
    }

    pub fn remove_while(
        &self,
        key: &ContentDigest,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<bool> {
        guard::check(&mut guard)?;
        key.validate()?;
        guard::check(&mut guard)?;
        let directory = self.directory(key);
        io::remove_while(&self.root, &directory, guard)
    }

    fn directory(&self, key: &ContentDigest) -> PathBuf {
        self.root
            .join("sha256")
            .join(&key.value[..2])
            .join(&key.value[2..])
    }
}
