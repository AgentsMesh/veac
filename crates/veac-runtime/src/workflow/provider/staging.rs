use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use veac_artifact::{artifact_key, ArtifactErrorKind, ContentDigest};
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_provider::{ProviderArtifact, ProviderResponseEnvelope};

use super::limits::ProviderResourceLimits;
use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

#[derive(Debug)]
pub(super) struct VerifiedPayload {
    pub artifact: ProviderArtifact,
    pub path: PathBuf,
}

pub(super) fn verify(
    root: &Path,
    response: &ProviderResponseEnvelope,
    limits: ProviderResourceLimits,
    deadline: Instant,
) -> WorkflowResult<Vec<VerifiedPayload>> {
    verify_while(root, response, limits, || Instant::now() < deadline)
}

pub(super) fn verify_while(
    root: &Path,
    response: &ProviderResponseEnvelope,
    limits: ProviderResourceLimits,
    mut guard: impl FnMut() -> bool,
) -> WorkflowResult<Vec<VerifiedPayload>> {
    active(&mut guard)?;
    limits.validate()?;
    active(&mut guard)?;
    verify_directory(root, &mut guard)?;
    let artifacts = response.output.artifacts();
    validate_budget(&artifacts, limits, &mut guard)?;
    let mut expected = BTreeSet::new();
    let mut payloads = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        active(&mut guard)?;
        let key = artifact_key(&artifact.descriptor)?;
        if !expected.insert(file_name(&key)) {
            return unsafe_staging("provider response contains duplicate artifact keys");
        }
        payloads.push(VerifiedPayload {
            artifact: artifact.clone(),
            path: root.join(file_name(&key)),
        });
    }
    if entries(root, limits.max_artifacts, &mut guard)? != expected {
        return unsafe_staging("provider staging payload set is missing or contains extra entries");
    }
    for payload in &payloads {
        active(&mut guard)?;
        verify_payload(payload, limits.max_payload_bytes, &mut guard)?;
    }
    active(&mut guard)?;
    Ok(payloads)
}

fn validate_budget(
    artifacts: &[&ProviderArtifact],
    limits: ProviderResourceLimits,
    guard: &mut impl FnMut() -> bool,
) -> WorkflowResult<()> {
    if artifacts.len() > limits.max_artifacts {
        return resource("provider artifact count exceeds the staging limit");
    }
    let mut total = 0_u64;
    for artifact in artifacts {
        active(guard)?;
        if artifact.record.size_bytes > limits.max_payload_bytes {
            return resource("provider payload exceeds the per-artifact byte limit");
        }
        total = total
            .checked_add(artifact.record.size_bytes)
            .ok_or_else(|| resource_error("provider payload byte total overflowed"))?;
        if total > limits.max_total_payload_bytes {
            return resource("provider payloads exceed the total staging byte limit");
        }
    }
    Ok(())
}

fn entries(
    root: &Path,
    limit: usize,
    guard: &mut impl FnMut() -> bool,
) -> WorkflowResult<BTreeSet<String>> {
    active(guard)?;
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(root)? {
        active(guard)?;
        if names.len() == limit {
            return resource("provider staging entry count exceeds the limit");
        }
        let name = entry?
            .file_name()
            .into_string()
            .map_err(|_| unsafe_error("provider staging entry name is not UTF-8"))?;
        names.insert(name);
        active(guard)?;
    }
    active(guard)?;
    Ok(names)
}

fn verify_directory(path: &Path, guard: &mut impl FnMut() -> bool) -> WorkflowResult<()> {
    active(guard)?;
    let metadata = fs::symlink_metadata(path)?;
    active(guard)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return unsafe_staging("provider staging root is not an isolated directory");
    }
    Ok(())
}

fn verify_payload(
    payload: &VerifiedPayload,
    max_bytes: u64,
    guard: &mut impl FnMut() -> bool,
) -> WorkflowResult<()> {
    active(guard)?;
    let metadata = fs::symlink_metadata(&payload.path)?;
    active(guard)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return unsafe_staging("provider payload is not a regular non-symlink file");
    }
    if metadata.len() != payload.artifact.record.size_bytes {
        return unsafe_staging("provider payload size differs from its declaration");
    }
    let expected = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: payload.artifact.record.content.value.clone(),
    };
    let verified = veac_artifact::verify_source_bounded_while(
        &payload.path,
        Some(&expected),
        max_bytes,
        &mut *guard,
    )
    .map_err(payload_error)?;
    active(guard)?;
    if verified.size_bytes != payload.artifact.record.size_bytes {
        return unsafe_staging("provider payload changed while it was verified");
    }
    Ok(())
}

fn payload_error(error: veac_artifact::ArtifactError) -> WorkflowError {
    if error.kind == ArtifactErrorKind::ResourceLimit {
        WorkflowError::with_source(
            WorkflowErrorKind::ResourceLimit,
            "provider payload exceeds the staging byte limit",
            error,
        )
    } else {
        WorkflowError::with_source(
            WorkflowErrorKind::UnsafeStaging,
            "provider payload failed identity or path verification",
            error,
        )
    }
}

fn file_name(key: &ContentDigest) -> String {
    format!("{}.payload", key.value)
}

fn active(guard: &mut impl FnMut() -> bool) -> WorkflowResult<()> {
    if guard() {
        Ok(())
    } else {
        resource("provider staging verification exceeded its wall-clock limit")
    }
}

fn unsafe_staging<T>(message: &str) -> WorkflowResult<T> {
    Err(unsafe_error(message))
}

fn unsafe_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::UnsafeStaging, message)
}

fn resource<T>(message: &str) -> WorkflowResult<T> {
    Err(resource_error(message))
}

fn resource_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ResourceLimit, message)
}

#[cfg(test)]
mod tests;
