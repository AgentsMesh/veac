use schemars::schema_for;
use sha2::{Digest, Sha256};

use super::ApiMetadataV1;
use crate::package::{PackageError, Sha256Digest};

const DIGEST_DOMAIN: &[u8] = b"veac.package.api.v2\0";

pub fn parse_api_metadata_json(input: &str) -> Result<ApiMetadataV1, PackageError> {
    crate::package::json::decode(input, "package API", ApiMetadataV1::validate)
}

pub fn canonical_api_metadata_json(value: &ApiMetadataV1) -> Result<String, PackageError> {
    value.validate()?;
    crate::package::json::canonical(value, "package API")
}

pub fn package_api_digest(value: &ApiMetadataV1) -> Result<Sha256Digest, PackageError> {
    let canonical = canonical_api_metadata_json(value)?;
    let mut digest = Sha256::new();
    digest.update(DIGEST_DOMAIN);
    digest.update(canonical.as_bytes());
    Ok(Sha256Digest::from_bytes(digest.finalize().into()))
}

pub fn package_api_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ApiMetadataV1))
}
