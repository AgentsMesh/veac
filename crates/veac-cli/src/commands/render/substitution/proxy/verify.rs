use std::time::Instant;

use veac_artifact::{
    select_proxy_while, verify_source_bounded_while, MediaArtifactSpec, ProxyBinding,
    ProxySelectionRequest, VerifiedArtifact,
};
use veac_ir::{HashAlgorithm, MediaIdentity, StreamChoice, StreamIntent};

use crate::arguments::SubstitutionPolicy;
use crate::environment::Environment;
use crate::error::{CliError, CliResult};

#[derive(Clone, Copy)]
enum Role {
    Video,
    Audio,
}

pub(super) fn select(
    store: &veac_artifact::ArtifactStore,
    request: &ProxySelectionRequest,
    policy: SubstitutionPolicy,
    environment: &dyn Environment,
    deadline: Instant,
) -> CliResult<ProxyBinding> {
    let mut selection = match select_proxy_while(store, request, || Instant::now() < deadline) {
        Ok(value) => value,
        Err(error)
            if policy == SubstitutionPolicy::Prefer
                && error.kind != veac_artifact::ArtifactErrorKind::ResourceLimit
                && Instant::now() < deadline =>
        {
            return Ok(empty(request));
        }
        Err(error) => return Err(selection_error(error)),
    };
    selection.video = role(
        selection.video.take(),
        Role::Video,
        policy,
        environment,
        deadline,
    )?;
    selection.audio = role(
        selection.audio.take(),
        Role::Audio,
        policy,
        environment,
        deadline,
    )?;
    Ok(selection)
}

fn role(
    artifact: Option<VerifiedArtifact>,
    role: Role,
    policy: SubstitutionPolicy,
    environment: &dyn Environment,
    deadline: Instant,
) -> CliResult<Option<VerifiedArtifact>> {
    let Some(artifact) = artifact else {
        return Ok(None);
    };
    match validate(&artifact, role, environment, deadline) {
        Ok(()) => Ok(Some(artifact)),
        Err(error)
            if policy == SubstitutionPolicy::Prefer
                && !error.is_resource_limit()
                && Instant::now() < deadline =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn validate(
    artifact: &VerifiedArtifact,
    role: Role,
    environment: &dyn Environment,
    deadline: Instant,
) -> CliResult {
    check_deadline(deadline)?;
    let spec: MediaArtifactSpec = serde_json::from_value(artifact.descriptor().parameters.clone())
        .map_err(|error| postflight_error(error.to_string()))?;
    if !role_matches(role, &spec) {
        return Err(postflight_error("proxy role and media spec differ"));
    }
    let expected = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: artifact.record().content.value.clone(),
    };
    verify_payload(artifact, &expected, deadline)?;
    let probe = environment
        .probe_until(artifact.payload_path(), intent(role), deadline)
        .map_err(probe_error)?;
    check_deadline(deadline)?;
    if probe.observed_identity != expected {
        return Err(postflight_error("proxy probe observed a different payload"));
    }
    veac_runtime::workflow::validate_media_artifact_snapshot(&probe, &spec)
        .map_err(workflow_error)?;
    verify_payload(artifact, &expected, deadline)?;
    Ok(())
}

fn verify_payload(
    artifact: &VerifiedArtifact,
    expected: &MediaIdentity,
    deadline: Instant,
) -> CliResult {
    verify_source_bounded_while(
        artifact.payload_path(),
        Some(expected),
        artifact.record().size_bytes.max(1),
        || Instant::now() < deadline,
    )
    .map(|_| ())
    .map_err(|error| {
        if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
            resource_error(error.to_string())
        } else {
            postflight_error(error.to_string())
        }
    })
}

fn intent(role: Role) -> StreamIntent {
    match role {
        Role::Video => StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Disabled,
        },
        Role::Audio => StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Auto,
        },
    }
}

fn role_matches(role: Role, spec: &MediaArtifactSpec) -> bool {
    matches!(
        (role, spec),
        (Role::Video, MediaArtifactSpec::ProxyVideo(_))
            | (Role::Audio, MediaArtifactSpec::ProxyAudio(_))
    )
}

fn empty(request: &ProxySelectionRequest) -> ProxyBinding {
    ProxyBinding {
        source_identity: request.source_identity.clone(),
        video: None,
        audio: None,
    }
}

fn selection_error(error: veac_artifact::ArtifactError) -> CliError {
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        CliError::resource_limit("PROXY_SELECTION_FAILED", error.to_string())
    } else {
        CliError::new("PROXY_SELECTION_FAILED", error.to_string())
    }
}

fn postflight_error(message: impl Into<String>) -> CliError {
    CliError::new("PROXY_POSTFLIGHT_FAILED", message)
}

fn resource_error(message: impl Into<String>) -> CliError {
    CliError::resource_limit("PROXY_POSTFLIGHT_FAILED", message)
}

fn check_deadline(deadline: Instant) -> CliResult {
    if Instant::now() >= deadline {
        Err(resource_error("proxy postflight exceeded its wall budget"))
    } else {
        Ok(())
    }
}

fn workflow_error(error: veac_runtime::workflow::WorkflowError) -> CliError {
    if error.kind == veac_runtime::workflow::WorkflowErrorKind::ResourceLimit {
        resource_error(error.to_string())
    } else {
        postflight_error(error.to_string())
    }
}

fn probe_error(error: CliError) -> CliError {
    if error.is_resource_limit() {
        resource_error(error.to_string())
    } else {
        postflight_error(error.to_string())
    }
}

#[cfg(test)]
#[path = "verify/tests.rs"]
mod tests;
