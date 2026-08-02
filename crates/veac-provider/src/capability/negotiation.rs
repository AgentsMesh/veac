use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Capability, ProviderFingerprint, ProviderManifest};
use crate::validation::invalid;
use crate::{
    ProviderError, ProviderErrorKind, ProviderResult, Validate, CAPABILITY_CONTRACT_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    pub capability: Capability,
    pub accepted_versions: Vec<u32>,
    pub deterministic: bool,
}

impl CapabilityRequirement {
    pub fn current(capability: Capability) -> Self {
        Self {
            capability,
            accepted_versions: vec![CAPABILITY_CONTRACT_VERSION],
            deterministic: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NegotiatedCapability {
    pub capability: Capability,
    pub contract_version: u32,
    pub provider: ProviderFingerprint,
}

pub fn negotiate(
    manifest: &ProviderManifest,
    requirement: &CapabilityRequirement,
) -> ProviderResult<NegotiatedCapability> {
    manifest.validate()?;
    if requirement.accepted_versions.is_empty()
        || !requirement.accepted_versions.iter().all(|value| *value > 0)
    {
        return invalid("accepted capability versions must be positive and non-empty");
    }
    let offer = manifest
        .offers
        .iter()
        .find(|offer| offer.capability == requirement.capability)
        .ok_or_else(|| unsupported(ProviderErrorKind::UnsupportedCapability, requirement))?;
    if requirement.deterministic && !offer.deterministic {
        return Err(unsupported(
            ProviderErrorKind::NondeterministicProvider,
            requirement,
        ));
    }
    let contract_version = offer
        .contract_versions
        .iter()
        .filter(|version| requirement.accepted_versions.contains(version))
        .max()
        .copied()
        .ok_or_else(|| unsupported(ProviderErrorKind::VersionMismatch, requirement))?;
    Ok(NegotiatedCapability {
        capability: requirement.capability,
        contract_version,
        provider: manifest.fingerprint.clone(),
    })
}

fn unsupported(kind: ProviderErrorKind, requirement: &CapabilityRequirement) -> ProviderError {
    ProviderError::new(
        kind,
        format!(
            "provider cannot satisfy {:?} contract",
            requirement.capability
        ),
    )
}
