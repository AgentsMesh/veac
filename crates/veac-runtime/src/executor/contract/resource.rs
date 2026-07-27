use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use veac_artifact::ContentDigest;
use veac_codegen::emitter::BackendResource;
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::invalid;
use crate::executor::model::RuntimeBundle;
use crate::RuntimeError;

pub(super) struct ValidatedResources {
    pub paths: BTreeSet<PathBuf>,
    pub fingerprint: ContentDigest,
}

pub(super) fn validate_while(
    bundle: &RuntimeBundle,
    guard: &mut impl FnMut() -> bool,
) -> Result<ValidatedResources, RuntimeError> {
    active(guard)?;
    bundle
        .substitution_proof
        .validate()
        .map_err(|error| RuntimeError::new(format!("invalid substitution proof: {error}")))?;
    let mut previous: Option<&str> = None;
    let mut canonical = BTreeMap::<String, (PathBuf, MediaIdentity)>::new();
    for resource in &bundle.protected_resources {
        active(guard)?;
        let text = utf8(resource)?;
        if previous.is_some_and(|value| value >= text) {
            return invalid("protected resources must be unique and sorted by path");
        }
        previous = Some(text);
        validate_expected(&resource.expected_identity)?;
        let path = canonical_path(resource)?;
        let key = path.to_str().unwrap().to_owned();
        if let Some((_, identity)) = canonical.get(&key) {
            if identity != &resource.expected_identity {
                return invalid("aliased protected resources have conflicting identities");
            }
            return invalid("protected resource paths may not alias one another");
        }
        verify_identity(resource, &path, guard)?;
        canonical.insert(key, (path, resource.expected_identity.clone()));
    }
    let fingerprint = fingerprint(&canonical, &bundle.substitution_proof);
    Ok(ValidatedResources {
        paths: canonical.into_values().map(|(path, _)| path).collect(),
        fingerprint,
    })
}

pub(super) fn verify_until(
    bundle: &RuntimeBundle,
    expected: &ContentDigest,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let actual = validate_while(bundle, &mut || Instant::now() < deadline)?;
    if actual.fingerprint != *expected {
        return Err(RuntimeError::new(
            "protected resource paths changed during bundle execution",
        ));
    }
    Ok(())
}

fn verify_identity(
    resource: &BackendResource,
    canonical: &PathBuf,
    guard: &mut impl FnMut() -> bool,
) -> Result<(), RuntimeError> {
    let verified = veac_artifact::verify_source_bounded_while(
        &resource.path,
        Some(&resource.expected_identity),
        veac_artifact::MAX_VERIFIED_SOURCE_BYTES,
        &mut *guard,
    )
    .map_err(|error| resource_error(resource, error))?;
    if verified.identity != resource.expected_identity {
        return Err(RuntimeError::new(format!(
            "protected resource identity changed: {}",
            resource.path.display()
        )));
    }
    if canonical_path(resource)? != *canonical {
        return Err(RuntimeError::new(format!(
            "protected resource path changed while hashing: {}",
            resource.path.display()
        )));
    }
    Ok(())
}

fn active(guard: &mut impl FnMut() -> bool) -> Result<(), RuntimeError> {
    if guard() {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "protected resource verification exceeded its execution deadline",
        ))
    }
}

fn resource_error(resource: &BackendResource, error: veac_artifact::ArtifactError) -> RuntimeError {
    if error.kind == veac_artifact::ArtifactErrorKind::IdentityMismatch {
        return RuntimeError::new(format!(
            "protected resource identity changed: {}",
            resource.path.display()
        ));
    }
    let message = format!(
        "cannot hash protected resource {}: {error}",
        resource.path.display()
    );
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}

fn validate_expected(identity: &MediaIdentity) -> Result<(), RuntimeError> {
    if identity.algorithm != HashAlgorithm::Sha256
        || identity.digest.len() != 64
        || !identity
            .digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return invalid("protected resource identity must be a lowercase SHA-256 digest");
    }
    Ok(())
}

fn canonical_path(resource: &BackendResource) -> Result<PathBuf, RuntimeError> {
    let metadata = fs::symlink_metadata(&resource.path).map_err(path_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return invalid("protected resource must be a regular non-symlink file");
    }
    let canonical = fs::canonicalize(&resource.path).map_err(path_error)?;
    if canonical.to_str().is_none() {
        return invalid("protected resource path must be valid UTF-8");
    }
    Ok(canonical)
}

fn fingerprint(
    resources: &BTreeMap<String, (PathBuf, MediaIdentity)>,
    substitution: &ContentDigest,
) -> ContentDigest {
    let mut bytes = b"veac-protected-resources-v1\0".to_vec();
    bytes.extend_from_slice(substitution.value.as_bytes());
    for (path, (_, identity)) in resources {
        bytes.extend_from_slice(&(path.len() as u64).to_be_bytes());
        bytes.extend_from_slice(path.as_bytes());
        bytes.extend_from_slice(identity.digest.as_bytes());
    }
    ContentDigest::sha256(bytes)
}

fn utf8(resource: &BackendResource) -> Result<&str, RuntimeError> {
    resource
        .path
        .to_str()
        .ok_or_else(|| RuntimeError::new("backend resource paths must be valid UTF-8"))
}

fn path_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!(
        "cannot validate protected backend resource: {error}"
    ))
}
