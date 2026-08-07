use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod catalog;
mod validation;

use super::{StandardLibrarySpec, VocabularyValidationError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginEffectSpec {
    #[schemars(length(min = 1))]
    pub descriptor_constructor: String,
    #[schemars(length(min = 1))]
    pub application_constructor: String,
    #[schemars(length(min = 1))]
    pub schema: String,
    pub schema_version: u16,
    #[schemars(length(min = 1))]
    pub namespace: String,
    #[schemars(length(min = 1))]
    pub implementation: String,
    #[schemars(length(min = 1))]
    pub effect_type: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub digest: String,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub parameters: Vec<PluginParameterSpec>,
    pub determinism: PluginDeterminismSpec,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub supported_backends: Vec<PluginBackendSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginParameterSpec {
    #[schemars(length(min = 1))]
    pub name: String,
    pub value_type: PluginParameterTypeSpec,
    #[schemars(regex(pattern = r"^-?(0|[1-9][0-9]*)(\.[0-9]+)?(e[+-]?[0-9]+)?$"))]
    pub minimum: Option<String>,
    #[schemars(regex(pattern = r"^-?(0|[1-9][0-9]*)(\.[0-9]+)?(e[+-]?[0-9]+)?$"))]
    pub maximum: Option<String>,
    pub supports_curve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PluginParameterTypeSpec {
    Number,
    Boolean,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PluginDeterminismSpec {
    Deterministic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PluginBackendSpec {
    #[serde(rename = "ffmpeg-8")]
    #[schemars(rename = "ffmpeg-8")]
    Ffmpeg8,
}

pub(crate) fn current() -> Vec<PluginEffectSpec> {
    catalog::current()
}

pub(crate) fn validate(
    values: &[PluginEffectSpec],
    library: &StandardLibrarySpec,
) -> Result<(), VocabularyValidationError> {
    validation::validate(values, library)
}
