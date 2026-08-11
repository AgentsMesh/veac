use std::{collections::BTreeMap, sync::Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::{
    artifact_key, ArtifactDescriptor, ArtifactParameters, ContentDigest, ProducerFingerprint,
    RenderCheckpointParameters, RenderOutputParameters,
};

use crate::{ArtifactOutputs, BuildError, BuildResult, BUILD_GRAPH_CONTRACT_VERSION};

mod disk;
mod reservation;

pub use disk::*;
pub use reservation::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NodeCacheKey {
    digest: ContentDigest,
    descriptor: ArtifactDescriptor,
}

impl NodeCacheKey {
    pub fn computation(
        action_kind: &str,
        action_version: u32,
        configuration: ContentDigest,
    ) -> BuildResult<Self> {
        let descriptor = ArtifactDescriptor::new(
            ProducerFingerprint {
                name: format!("veac-build:{action_kind}"),
                version: format!("{BUILD_GRAPH_CONTRACT_VERSION}.{action_version}"),
                configuration,
            },
            Vec::new(),
            ArtifactParameters::RenderCheckpoint(RenderCheckpointParameters::Output(
                RenderOutputParameters::new(0, "veac-build-computation-record-v1"),
            )),
        );
        let digest = artifact_key(&descriptor)
            .map_err(|error| BuildError::invalid(format!("invalid computation key: {error}")))?;
        Ok(Self { digest, descriptor })
    }

    pub fn digest(&self) -> &ContentDigest {
        &self.digest
    }

    pub fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
    }
}

impl PartialEq for NodeCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

impl Eq for NodeCacheKey {}

impl PartialOrd for NodeCacheKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeCacheKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.digest.cmp(&other.digest)
    }
}

pub trait BuildCache: Send + Sync {
    fn get(&self, key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>>;
    fn put(&self, key: &NodeCacheKey, outputs: &ArtifactOutputs) -> BuildResult<()>;

    fn reserve(
        &self,
        key: &NodeCacheKey,
        cancellation: &crate::CancellationToken,
    ) -> BuildResult<CacheReservation> {
        if cancellation.is_cancelled() {
            return Err(BuildError::cancelled(
                "build was cancelled before cache reservation",
            ));
        }
        match self.get(key)? {
            Some(outputs) => Ok(CacheReservation::Hit(outputs)),
            None => Ok(CacheReservation::Owner(Box::new(()))),
        }
    }
}

#[derive(Debug, Default)]
pub struct MemoryBuildCache {
    pub(crate) entries: Mutex<BTreeMap<NodeCacheKey, ArtifactOutputs>>,
}

impl MemoryBuildCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> BuildResult<usize> {
        self.entries
            .lock()
            .map(|entries| entries.len())
            .map_err(|_| BuildError::cache("in-memory build cache lock is poisoned"))
    }

    pub fn is_empty(&self) -> BuildResult<bool> {
        self.len().map(|length| length == 0)
    }
}

impl BuildCache for MemoryBuildCache {
    fn get(&self, key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>> {
        self.entries
            .lock()
            .map(|entries| entries.get(key).cloned())
            .map_err(|_| BuildError::cache("in-memory build cache lock is poisoned"))
    }

    fn put(&self, key: &NodeCacheKey, outputs: &ArtifactOutputs) -> BuildResult<()> {
        self.entries
            .lock()
            .map_err(|_| BuildError::cache("in-memory build cache lock is poisoned"))?
            .insert(key.clone(), outputs.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NullBuildCache;

impl BuildCache for NullBuildCache {
    fn get(&self, _key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>> {
        Ok(None)
    }

    fn put(&self, _key: &NodeCacheKey, _outputs: &ArtifactOutputs) -> BuildResult<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;
