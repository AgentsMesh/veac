use std::time::Instant;

use veac_artifact::ProducerFingerprint;

use crate::executor::{FfmpegFingerprint, SystemFfmpeg};
use crate::input_policy;
use crate::RuntimeError;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn verify(
    ffmpeg: &SystemFfmpeg,
    expected: &ProducerFingerprint,
    deadline: Instant,
) -> WorkflowResult<()> {
    let fingerprint = ffmpeg.fingerprint_until(deadline).map_err(|error| {
        let kind = if error.kind == crate::RuntimeErrorKind::ResourceLimit {
            WorkflowErrorKind::ResourceLimit
        } else {
            WorkflowErrorKind::ToolFailure
        };
        WorkflowError::with_source(
            kind,
            "failed to fingerprint FFmpeg artifact producer",
            error,
        )
    })?;
    let actual = match media_artifact_producer(&fingerprint) {
        Ok(actual) => actual,
        Err(error) => {
            return Err(WorkflowError::with_source(
                WorkflowErrorKind::ToolFailure,
                "FFmpeg artifact producer fingerprint is invalid",
                error,
            ))
        }
    };
    if actual != *expected {
        return Err(WorkflowError::new(
            WorkflowErrorKind::InvalidContract,
            "artifact producer does not match the configured FFmpeg binary",
        ));
    }
    Ok(())
}

pub fn media_artifact_producer(
    fingerprint: &FfmpegFingerprint,
) -> Result<ProducerFingerprint, RuntimeError> {
    fingerprint.configuration.validate().map_err(|error| {
        RuntimeError::new(format!("invalid FFmpeg producer configuration: {error}"))
    })?;
    if fingerprint.version.trim().is_empty() {
        return Err(RuntimeError::new(
            "FFmpeg producer version must not be empty",
        ));
    }
    let mut contract = b"veac.media-workflow-producer".to_vec();
    contract.extend_from_slice(&3_u32.to_be_bytes());
    field(&mut contract, fingerprint.configuration.value.as_bytes());
    for argument in input_policy::string_arguments() {
        field(&mut contract, argument.as_bytes());
    }
    Ok(ProducerFingerprint {
        name: "veac-ffmpeg".to_owned(),
        version: fingerprint.version.clone(),
        configuration: veac_artifact::ContentDigest::sha256(contract),
    })
}

fn field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

#[cfg(test)]
#[path = "tool/tests.rs"]
mod tests;
