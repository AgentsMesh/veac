use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use super::{
    BuildInputManifestValue, BuildInputsError, MAX_BUILD_INPUTS, MAX_BUILD_INPUT_MANIFEST_BYTES,
};

pub const BUILD_INPUT_MANIFEST_SCHEMA: &str = "https://veac.dev/schemas/build-inputs";
pub const BUILD_INPUT_MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildInputManifestV1 {
    #[schemars(extend("const" = BUILD_INPUT_MANIFEST_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = BUILD_INPUT_MANIFEST_VERSION))]
    pub schema_version: u32,
    #[schemars(length(max = MAX_BUILD_INPUTS))]
    pub inputs: Vec<BuildInputBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildInputBinding {
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = r"^[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z0-9_]+)*$")
    )]
    pub name: String,
    pub value: BuildInputManifestValue,
}

impl BuildInputManifestV1 {
    pub fn empty() -> Self {
        Self {
            schema: BUILD_INPUT_MANIFEST_SCHEMA.to_owned(),
            schema_version: BUILD_INPUT_MANIFEST_VERSION,
            inputs: Vec::new(),
        }
    }

    pub fn validate_identity(&self) -> Result<(), BuildInputsError> {
        if self.schema != BUILD_INPUT_MANIFEST_SCHEMA
            || self.schema_version != BUILD_INPUT_MANIFEST_VERSION
        {
            return Err(BuildInputsError::new(
                "PROGRAM_INPUT_MANIFEST_VERSION",
                "unsupported Build input manifest schema identity",
            ));
        }
        if self.inputs.len() > MAX_BUILD_INPUTS {
            return Err(BuildInputsError::new(
                "PROGRAM_INPUT_LIMIT",
                format!("manifest exceeds the {MAX_BUILD_INPUTS} Build input limit"),
            ));
        }
        for binding in &self.inputs {
            if !crate::name::is_name(&binding.name) {
                return Err(BuildInputsError::new(
                    "PROGRAM_INPUT_NAME",
                    "Build input binding name must be a VEAC declaration name",
                ));
            }
            binding.value.validate_shape()?;
        }
        Ok(())
    }
}

pub fn parse_build_input_manifest(input: &str) -> Result<BuildInputManifestV1, BuildInputsError> {
    if input.len() > MAX_BUILD_INPUT_MANIFEST_BYTES {
        return Err(BuildInputsError::new(
            "PROGRAM_INPUT_MANIFEST_LIMIT",
            format!("Build input manifest exceeds the {MAX_BUILD_INPUT_MANIFEST_BYTES} byte limit"),
        ));
    }
    let manifest: BuildInputManifestV1 = serde_json::from_str(input)
        .map_err(|error| BuildInputsError::new("PROGRAM_INPUT_MANIFEST_JSON", error.to_string()))?;
    manifest.validate_identity()?;
    Ok(manifest)
}

pub fn build_input_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(BuildInputManifestV1))
}
