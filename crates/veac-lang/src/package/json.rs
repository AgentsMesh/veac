use schemars::schema_for;
use serde::de::DeserializeOwned;

use super::{
    PackageError, PackageErrorKind, PackageLockV1, PackageManifestV1, MAX_PACKAGE_JSON_BYTES,
};

pub fn parse_package_manifest_json(input: &str) -> Result<PackageManifestV1, PackageError> {
    decode(input, "package manifest", PackageManifestV1::validate)
}

pub fn parse_package_lock_json(input: &str) -> Result<PackageLockV1, PackageError> {
    decode(input, "package lock", PackageLockV1::validate)
}

pub fn canonical_package_manifest_json(value: &PackageManifestV1) -> Result<String, PackageError> {
    value.validate()?;
    canonical(value, "package manifest")
}

pub fn canonical_package_lock_json(value: &PackageLockV1) -> Result<String, PackageError> {
    value.validate()?;
    canonical(value, "package lock")
}

pub fn package_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(PackageManifestV1))
}

pub fn package_lock_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(PackageLockV1))
}

pub(crate) fn decode<T>(
    input: &str,
    label: &str,
    validate: impl FnOnce(&T) -> Result<(), PackageError>,
) -> Result<T, PackageError>
where
    T: DeserializeOwned,
{
    if input.len() > MAX_PACKAGE_JSON_BYTES {
        return Err(PackageError::new(
            PackageErrorKind::Contract,
            format!("{label} exceeds the JSON byte limit"),
        ));
    }
    veac_ir::reject_duplicate_json_keys(input)
        .map_err(|error| json_error(label, error.to_string()))?;
    let value =
        serde_json::from_str(input).map_err(|error| json_error(label, error.to_string()))?;
    validate(&value)?;
    Ok(value)
}

pub(crate) fn canonical(
    value: &impl serde::Serialize,
    label: &str,
) -> Result<String, PackageError> {
    serde_json_canonicalizer::to_string(value).map_err(|error| json_error(label, error.to_string()))
}

fn json_error(label: &str, detail: impl std::fmt::Display) -> PackageError {
    PackageError::new(
        PackageErrorKind::Json,
        format!("invalid {label} JSON: {detail}"),
    )
}
