mod bindings;
mod exact;
mod manifest;
mod material;
mod primitive;
mod shape;
mod value;

pub const MAX_BUILD_INPUTS: usize = 256;
pub const MAX_BUILD_INPUT_MANIFEST_BYTES: usize = 16 * 1024 * 1024;

pub(crate) use bindings::VerifiedBuildInputs;
pub use manifest::{
    build_input_manifest_json_schema, parse_build_input_manifest, BuildInputBinding,
    BuildInputManifestV1, BUILD_INPUT_MANIFEST_SCHEMA, BUILD_INPUT_MANIFEST_VERSION,
};
pub(crate) use material::matches as is_material_binding_type;
pub use material::{MaterialInputAuthority, MaterialInputKind};
pub use value::BuildInputManifestValue;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::expression::{BuildInputSlot, CoreBuildInputId, ValueType};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BuildInputRole {
    Parameter,
    AssetMetadata,
    Analysis,
    Material,
}

impl BuildInputRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parameter => "parameter",
            Self::AssetMetadata => "asset_metadata",
            Self::Analysis => "analysis",
            Self::Material => "material",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInputDeclaration {
    id: CoreBuildInputId,
    name: String,
    role: BuildInputRole,
    value_type: ValueType,
}

impl BuildInputDeclaration {
    pub(crate) fn new(
        source_id: &str,
        name: String,
        role: BuildInputRole,
        value_type: ValueType,
    ) -> Self {
        let id = CoreBuildInputId::for_declaration(
            source_id,
            &name,
            role.as_str(),
            &value_type.to_string(),
        );
        Self {
            id,
            name,
            role,
            value_type,
        }
    }

    pub fn id(&self) -> CoreBuildInputId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn role(&self) -> BuildInputRole {
        self.role
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub(crate) fn expression_slot(&self) -> BuildInputSlot {
        BuildInputSlot::new(self.id, self.name.clone(), self.value_type.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInputsError {
    code: &'static str,
    message: String,
}

impl BuildInputsError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for BuildInputsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BuildInputsError {}

#[cfg(test)]
#[path = "build_input/tests.rs"]
mod tests;
