use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::{
    artifact_key, ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactKind,
    ArtifactParameters, ArtifactRecord,
};

use crate::{ProviderFingerprint, ProviderResult, Validate};

pub use veac_artifact::ProviderArtifactSlot;

mod binding;
pub use binding::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderArtifact {
    pub role: ProviderArtifactSlot,
    pub descriptor: ArtifactDescriptor,
    pub record: ArtifactRecord,
}

impl ProviderArtifact {
    pub fn kind(&self) -> ArtifactKind {
        self.descriptor.kind()
    }
}

impl Validate for ProviderArtifact {
    fn validate(&self) -> ProviderResult<()> {
        self.descriptor.validate()?;
        self.record.key.validate()?;
        self.record.content.validate()?;
        if self.descriptor.parameters.provider_slot() != Some(&self.role)
            || artifact_key(&self.descriptor)? != self.record.key
            || self.record.size_bytes > veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES
        {
            return crate::validation::invalid("artifact record does not match its descriptor");
        }
        Ok(())
    }
}

pub fn provider_artifact_descriptor(
    provider: &ProviderFingerprint,
    request_hash: veac_artifact::ContentDigest,
    mut dependencies: Vec<ArtifactDependency>,
    parameters: ArtifactParameters,
) -> ProviderResult<ArtifactDescriptor> {
    provider.validate()?;
    request_hash.validate()?;
    dependencies.push(ArtifactDependency::new(
        ArtifactDependencyRole::ProviderRequest,
        request_hash,
    ));
    dependencies.sort_by(|left, right| {
        (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
    });
    let descriptor =
        ArtifactDescriptor::new(provider.artifact_producer(), dependencies, parameters);
    descriptor.validate()?;
    Ok(descriptor)
}
