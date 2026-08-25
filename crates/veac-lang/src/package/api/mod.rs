//! Canonical package API metadata.

mod json;
mod model;
mod project;
mod validation;

pub use json::{
    canonical_api_metadata_json, package_api_digest, package_api_json_schema,
    parse_api_metadata_json,
};
pub use model::{
    ApiCallableSemantics, ApiDomainCapability, ApiEffect, ApiEnumVariant, ApiExport, ApiField,
    ApiFunctionParameter, ApiMetadataV1, ApiParameterDependency, ApiPrimitive, ApiResultSemantics,
    ApiSemanticEffect, ApiStage, ApiType, ApiTypeDefinition, ApiTypeName, MapKeyType,
};
pub use project::from_module_interface;

pub const PACKAGE_API_SCHEMA: &str = "https://veac.dev/schemas/package-api/v2";
pub const PACKAGE_API_SCHEMA_VERSION: u32 = 2;
