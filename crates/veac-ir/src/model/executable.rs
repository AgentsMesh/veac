use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CURRENT_CORE_VERSION, CURRENT_DOMAIN_OPSET_VERSION, TEMPORAL_OPSET_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutableLanguage {
    Veac,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutableDigests {
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub domain_registry_sha256: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub main_core_sha256: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub source_graph_sha256: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub declared_inputs_sha256: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub compiler_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutableManifest {
    pub language: ExecutableLanguage,
    pub language_version: String,
    pub core_version: u16,
    pub domain_opset_version: u16,
    pub temporal_opset_version: u16,
    pub digests: ExecutableDigests,
}

impl ExecutableManifest {
    pub fn current(language_version: impl Into<String>, digests: ExecutableDigests) -> Self {
        Self {
            language: ExecutableLanguage::Veac,
            language_version: language_version.into(),
            core_version: CURRENT_CORE_VERSION,
            domain_opset_version: CURRENT_DOMAIN_OPSET_VERSION,
            temporal_opset_version: TEMPORAL_OPSET_VERSION,
            digests,
        }
    }
}
