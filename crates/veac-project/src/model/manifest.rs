use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    LocaleId, ProfileId, ProjectId, ProjectLocale, ProjectPath, ProjectProfile, ProjectTarget,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifestV1 {
    pub schema: String,
    pub version: u32,
    pub id: ProjectId,
    pub paths: ProjectPaths,
    pub defaults: ProjectDefaults,
    pub locales: Vec<ProjectLocale>,
    pub profiles: Vec<ProjectProfile>,
    pub targets: Vec<ProjectTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectPaths {
    pub source_base: ProjectPath,
    pub material_root: ProjectPath,
    pub build_root: ProjectPath,
    pub cache_root: ProjectPath,
    pub delivery_root: ProjectPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDefaults {
    pub profile: Option<ProfileId>,
    pub locale: Option<LocaleId>,
    pub max_instances_per_target: u32,
    pub max_total_instances: u32,
}

impl Default for ProjectDefaults {
    fn default() -> Self {
        Self {
            profile: None,
            locale: None,
            max_instances_per_target: 256,
            max_total_instances: 4_096,
        }
    }
}
