use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use veac_artifact::{
    artifact_key, ArtifactDependency, ArtifactDescriptor, ArtifactKind, ArtifactRecord,
};

use crate::validation::text;
use crate::{ProviderFingerprint, ProviderResult, Validate};

mod binding;
pub use binding::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderArtifact {
    pub role: String,
    pub descriptor: ArtifactDescriptor,
    pub record: ArtifactRecord,
}

impl ProviderArtifact {
    pub fn kind(&self) -> ArtifactKind {
        self.descriptor.kind
    }
}

impl Validate for ProviderArtifact {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.role, "artifact role")?;
        self.descriptor.validate()?;
        self.record.key.validate()?;
        self.record.content.validate()?;
        if artifact_key(&self.descriptor)? != self.record.key
            || self.record.size_bytes > veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES
        {
            return crate::validation::invalid("artifact record does not match its descriptor");
        }
        Ok(())
    }
}

pub fn provider_artifact_descriptor(
    kind: ArtifactKind,
    provider: &ProviderFingerprint,
    request_hash: veac_artifact::ContentDigest,
    mut dependencies: Vec<ArtifactDependency>,
    parameters: Value,
) -> ProviderResult<ArtifactDescriptor> {
    provider.validate()?;
    request_hash.validate()?;
    dependencies.push(ArtifactDependency {
        role: "provider_request".to_owned(),
        identity: request_hash,
    });
    dependencies.sort_by(|left, right| {
        (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
    });
    let descriptor =
        ArtifactDescriptor::new(kind, provider.artifact_producer(), dependencies, parameters);
    descriptor.validate()?;
    Ok(descriptor)
}
