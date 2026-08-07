use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ArtifactResult, ContentDigest};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDependencyRole {
    Input,
    Source,
    Plan,
    Profile,
    SourceClocks,
    PreviousPass,
    Resources,
    Task,
    ProviderRequest,
    ProviderExecutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDependency {
    pub role: ArtifactDependencyRole,
    pub identity: ContentDigest,
}

impl ArtifactDependency {
    pub fn new(role: ArtifactDependencyRole, identity: ContentDigest) -> Self {
        Self { role, identity }
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        self.identity.validate().map_err(|_| {
            crate::ArtifactError::new(
                crate::ArtifactErrorKind::InvalidContract,
                "artifact dependency identity is invalid",
            )
        })
    }
}
