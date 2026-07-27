use serde::{Deserialize, Serialize};
use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_provider::{ProviderFingerprint, Validate};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

const EXECUTION_IDENTITY_DOMAIN: &[u8] = b"veac.provider-execution.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderExecutionIdentity {
    pub provider: ProviderFingerprint,
    pub executable: ContentDigest,
    pub digest: ContentDigest,
}

impl ProviderExecutionIdentity {
    pub(super) fn from_pinned(
        provider: &ProviderFingerprint,
        pinned: &MediaIdentity,
    ) -> WorkflowResult<Self> {
        provider.validate()?;
        if pinned.algorithm != HashAlgorithm::Sha256 {
            return Err(WorkflowError::new(
                WorkflowErrorKind::ToolFailure,
                "pinned provider executable does not use SHA-256 identity",
            ));
        }
        let executable = ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: pinned.digest.clone(),
        };
        executable.validate()?;
        let provider_bytes = serde_json_canonicalizer::to_vec(provider).map_err(|error| {
            WorkflowError::with_source(
                WorkflowErrorKind::Serialization,
                "provider execution identity cannot be canonicalized",
                error,
            )
        })?;
        let mut bytes = EXECUTION_IDENTITY_DOMAIN.to_vec();
        append_field(&mut bytes, &provider_bytes);
        append_field(&mut bytes, executable.value.as_bytes());
        Ok(Self {
            provider: provider.clone(),
            executable,
            digest: ContentDigest::sha256(bytes),
        })
    }
}

fn append_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

#[cfg(test)]
mod tests;
