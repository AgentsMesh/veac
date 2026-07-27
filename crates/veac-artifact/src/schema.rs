use schemars::schema_for;

use crate::{
    ArtifactDescriptor, BuildManifest, ExecutionBindingManifest, MediaArtifactRequest,
    PackageManifest,
};

pub fn artifact_descriptor_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ArtifactDescriptor))
}

pub fn build_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(BuildManifest))
}

pub fn package_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(PackageManifest))
}

pub fn media_artifact_request_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(MediaArtifactRequest))
}

pub fn execution_binding_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ExecutionBindingManifest))
}
