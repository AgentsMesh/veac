use std::path::{Path, PathBuf};

use veac_ir::{MediaIdentity, StreamSelection};

use super::SourceClock;
use crate::{ArtifactDescriptor, ArtifactRecord, ArtifactResult, ContentDigest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaRole {
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingProvenanceKind {
    Original,
    Artifact,
}

/// A protected machine resource constructible only by verified binding workflows.
///
/// ```compile_fail
/// use std::path::PathBuf;
/// use veac_artifact::BoundResource;
/// use veac_ir::MediaIdentity;
///
/// fn forge(path: PathBuf, identity: MediaIdentity) -> BoundResource {
///     BoundResource { path, identity, provenance: todo!() }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundResource {
    path: PathBuf,
    identity: MediaIdentity,
    provenance: BindingProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BindingProvenance {
    Original { proof: ContentDigest },
    Artifact(ArtifactBindingProof),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactBindingProof {
    kind: crate::ArtifactKind,
    key: ContentDigest,
    content: ContentDigest,
    size_bytes: u64,
    proof: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundStream {
    resource: BoundResource,
    physical_stream: StreamSelection,
    clock: SourceClock,
}

impl BoundResource {
    pub(super) fn original(path: PathBuf, identity: MediaIdentity) -> Self {
        let proof = super::proof::original_resource(&identity);
        Self {
            path,
            identity,
            provenance: BindingProvenance::Original { proof },
        }
    }

    pub(super) fn artifact(
        path: PathBuf,
        identity: MediaIdentity,
        descriptor: &ArtifactDescriptor,
        record: &ArtifactRecord,
    ) -> Self {
        let proof = super::proof::artifact_resource(descriptor, record);
        Self {
            path,
            identity,
            provenance: BindingProvenance::Artifact(ArtifactBindingProof {
                kind: descriptor.kind,
                key: record.key.clone(),
                content: record.content.clone(),
                size_bytes: record.size_bytes,
                proof,
            }),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn identity(&self) -> &MediaIdentity {
        &self.identity
    }

    pub fn read_verified(&self) -> ArtifactResult<Vec<u8>> {
        crate::read_verified_source(&self.path, Some(&self.identity))
    }

    pub fn read_verified_bounded(&self, max_bytes: u64) -> ArtifactResult<Vec<u8>> {
        crate::read_verified_source_bounded(&self.path, Some(&self.identity), max_bytes)
    }

    pub fn provenance_kind(&self) -> BindingProvenanceKind {
        match self.provenance {
            BindingProvenance::Original { .. } => BindingProvenanceKind::Original,
            BindingProvenance::Artifact(_) => BindingProvenanceKind::Artifact,
        }
    }

    pub fn artifact_key(&self) -> Option<&ContentDigest> {
        match &self.provenance {
            BindingProvenance::Artifact(value) => Some(&value.key),
            BindingProvenance::Original { .. } => None,
        }
    }

    pub fn artifact_kind(&self) -> Option<crate::ArtifactKind> {
        match &self.provenance {
            BindingProvenance::Original { .. } => None,
            BindingProvenance::Artifact(value) => Some(value.kind),
        }
    }

    pub fn proof_digest(&self) -> &ContentDigest {
        match &self.provenance {
            BindingProvenance::Original { proof } => proof,
            BindingProvenance::Artifact(value) => &value.proof,
        }
    }

    pub(super) fn proof_fields(&self) -> Option<(&ContentDigest, &ContentDigest, u64)> {
        match &self.provenance {
            BindingProvenance::Artifact(value) => {
                Some((&value.key, &value.content, value.size_bytes))
            }
            BindingProvenance::Original { .. } => None,
        }
    }
}

impl BoundStream {
    pub(super) fn new(
        resource: BoundResource,
        physical_stream: StreamSelection,
        clock: SourceClock,
    ) -> Self {
        Self {
            resource,
            physical_stream,
            clock,
        }
    }

    pub fn resource(&self) -> &BoundResource {
        &self.resource
    }

    pub fn physical_stream(&self) -> StreamSelection {
        self.physical_stream
    }

    pub fn clock(&self) -> SourceClock {
        self.clock
    }
}
