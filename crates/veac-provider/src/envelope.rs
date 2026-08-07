use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;

use crate::validation::invalid;
use crate::{
    Capability, NegotiatedCapability, ProviderFingerprint, ProviderOutput, ProviderRequest,
    ProviderResult, Validate, CAPABILITY_CONTRACT_VERSION, PROVIDER_SCHEMA_ID,
    PROVIDER_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderRequestEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub capability: Capability,
    pub contract_version: u32,
    pub provider: ProviderFingerprint,
    pub request: ProviderRequest,
}

impl ProviderRequestEnvelope {
    pub fn new(negotiated: NegotiatedCapability, request: ProviderRequest) -> ProviderResult<Self> {
        let value = Self {
            schema: PROVIDER_SCHEMA_ID.to_owned(),
            schema_version: PROVIDER_SCHEMA_VERSION,
            capability: negotiated.capability,
            contract_version: negotiated.contract_version,
            provider: negotiated.provider,
            request,
        };
        value.validate()?;
        Ok(value)
    }
}

impl Validate for ProviderRequestEnvelope {
    fn validate(&self) -> ProviderResult<()> {
        validate_header(
            &self.schema,
            self.schema_version,
            self.contract_version,
            &self.provider,
        )?;
        if self.capability != self.request.capability() {
            return invalid("request payload does not match the negotiated capability");
        }
        self.request.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderResponseEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub capability: Capability,
    pub contract_version: u32,
    pub provider: ProviderFingerprint,
    pub request_hash: ContentDigest,
    pub output: ProviderOutput,
}

impl ProviderResponseEnvelope {
    pub fn new(request: &ProviderRequestEnvelope, output: ProviderOutput) -> ProviderResult<Self> {
        request.validate()?;
        let value = Self {
            schema: PROVIDER_SCHEMA_ID.to_owned(),
            schema_version: PROVIDER_SCHEMA_VERSION,
            capability: request.capability,
            contract_version: request.contract_version,
            provider: request.provider.clone(),
            request_hash: request_hash(request)?,
            output,
        };
        value.validate_for(request)?;
        Ok(value)
    }

    pub fn validate_for(&self, request: &ProviderRequestEnvelope) -> ProviderResult<()> {
        request.validate()?;
        self.validate()?;
        if self.capability != request.capability
            || self.contract_version != request.contract_version
            || self.provider != request.provider
            || self.request_hash != request_hash(request)?
        {
            return invalid("provider response does not match its request envelope");
        }
        Ok(())
    }
}

impl Validate for ProviderResponseEnvelope {
    fn validate(&self) -> ProviderResult<()> {
        validate_header(
            &self.schema,
            self.schema_version,
            self.contract_version,
            &self.provider,
        )?;
        self.request_hash.validate()?;
        if self.capability != self.output.capability() {
            return invalid("response payload does not match the negotiated capability");
        }
        self.output.validate()?;
        for artifact in self.output.artifacts() {
            let request_dependencies = artifact
                .descriptor
                .dependencies
                .iter()
                .filter(|dependency| {
                    dependency.role == veac_artifact::ArtifactDependencyRole::ProviderRequest
                })
                .collect::<Vec<_>>();
            if artifact.descriptor.producer != self.provider.artifact_producer()
                || request_dependencies.len() != 1
                || request_dependencies[0].identity != self.request_hash
            {
                return invalid("provider artifact is not bound to this provider request");
            }
        }
        Ok(())
    }
}

pub fn canonical_request_bytes(value: &ProviderRequestEnvelope) -> ProviderResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(Into::into)
}

pub fn request_hash(value: &ProviderRequestEnvelope) -> ProviderResult<ContentDigest> {
    canonical_request_bytes(value).map(ContentDigest::sha256)
}

pub fn canonical_response_bytes(value: &ProviderResponseEnvelope) -> ProviderResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(Into::into)
}

pub fn response_hash(value: &ProviderResponseEnvelope) -> ProviderResult<ContentDigest> {
    canonical_response_bytes(value).map(ContentDigest::sha256)
}

fn validate_header(
    schema: &str,
    schema_version: u32,
    contract_version: u32,
    provider: &ProviderFingerprint,
) -> ProviderResult<()> {
    if schema != PROVIDER_SCHEMA_ID || schema_version != PROVIDER_SCHEMA_VERSION {
        return invalid("unsupported provider envelope schema");
    }
    if contract_version != CAPABILITY_CONTRACT_VERSION {
        return invalid("unsupported provider capability contract version");
    }
    provider.validate()
}
