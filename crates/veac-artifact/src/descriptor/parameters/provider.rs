use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ArtifactResult, MAX_ARTIFACT_JSON_STRING_BYTES};

use super::super::{invalid, resource_limit};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ProviderArtifactSlot(String);

impl ProviderArtifactSlot {
    pub fn new(value: impl Into<String>) -> ArtifactResult<Self> {
        let value = value.into();
        validate_slot(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        validate_slot(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderResultParameters {
    pub slot: ProviderArtifactSlot,
}

impl ProviderResultParameters {
    pub fn new(slot: impl Into<String>) -> ArtifactResult<Self> {
        Ok(Self {
            slot: ProviderArtifactSlot::new(slot)?,
        })
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        self.slot.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "origin",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProducedArtifactParameters {
    Provider(ProviderResultParameters),
    Render(super::RenderOutputParameters),
}

impl ProducedArtifactParameters {
    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        match self {
            Self::Provider(value) => value.validate(),
            Self::Render(value) => value.validate(),
        }
    }
}

fn validate_slot(value: &str) -> ArtifactResult<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return invalid("provider artifact slot must be a nominal identifier");
    }
    if value.len() > MAX_ARTIFACT_JSON_STRING_BYTES {
        return resource_limit("provider artifact slot exceeds its byte budget");
    }
    Ok(())
}
