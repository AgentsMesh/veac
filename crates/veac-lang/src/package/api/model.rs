use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::package::PackageIdentity;

mod semantics;

pub use semantics::{
    ApiCallableSemantics, ApiEffect, ApiParameterDependency, ApiResultSemantics, ApiSemanticEffect,
    ApiStage,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiMetadataV1 {
    #[schemars(extend("const" = super::PACKAGE_API_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = super::PACKAGE_API_SCHEMA_VERSION))]
    pub schema_version: u32,
    pub package: PackageIdentity,
    pub exports: Vec<ApiExport>,
    pub domain_capabilities: Vec<ApiDomainCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApiExport {
    Function {
        name: String,
        parameters: Vec<ApiFunctionParameter>,
        return_type: ApiType,
        semantics: ApiCallableSemantics,
    },
    Method {
        receiver: ApiTypeName,
        name: String,
        parameters: Vec<ApiFunctionParameter>,
        return_type: ApiType,
        semantics: ApiCallableSemantics,
    },
    Type {
        name: String,
        definition: ApiTypeDefinition,
    },
    Constant {
        name: String,
        value_type: ApiType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiFunctionParameter {
    pub name: String,
    pub value_type: ApiType,
    pub has_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApiType {
    Primitive {
        name: ApiPrimitive,
    },
    List {
        element: Box<ApiType>,
    },
    Range {
        element: ApiPrimitive,
    },
    Map {
        key: MapKeyType,
        value: Box<ApiType>,
    },
    Tuple {
        elements: Vec<ApiType>,
    },
    Function {
        parameters: Vec<ApiType>,
        return_type: Box<ApiType>,
        effect: ApiEffect,
    },
    Named {
        name: ApiTypeName,
    },
    Domain {
        name: String,
        opcode: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "definition", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApiTypeDefinition {
    Struct { fields: Vec<ApiField> },
    Enum { variants: Vec<ApiEnumVariant> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiField {
    pub name: String,
    pub value_type: ApiType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiEnumVariant {
    pub name: String,
    pub fields: Vec<ApiField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiTypeName {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiDomainCapability {
    pub opcode: u16,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MapKeyType {
    Text,
    Identifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiPrimitive {
    Int,
    Scalar,
    Time,
    Length,
    Percent,
    Angle,
    Text,
    Color,
    Bool,
    Identifier,
}

impl ApiMetadataV1 {
    pub fn new(package: PackageIdentity, exports: Vec<ApiExport>) -> Self {
        Self {
            schema: super::PACKAGE_API_SCHEMA.to_owned(),
            schema_version: super::PACKAGE_API_SCHEMA_VERSION,
            package,
            exports,
            domain_capabilities: Vec::new(),
        }
    }

    pub fn with_domain_capabilities(mut self, values: Vec<ApiDomainCapability>) -> Self {
        self.domain_capabilities = values;
        self
    }

    pub fn validate(&self) -> Result<(), crate::package::PackageError> {
        super::validation::metadata(self)
    }
}
