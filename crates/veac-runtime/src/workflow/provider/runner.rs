use std::path::Path;
use std::time::Instant;

use veac_artifact::{ArtifactRecord, ArtifactStore};
use veac_provider::{
    bind_provider_executable, canonical_request_bytes, negotiate, CapabilityRequirement,
    ProviderArtifact, ProviderRequestEnvelope,
};

use crate::tool::{LaunchExecutable, PinnedExecutable};
use crate::RuntimeError;

use super::protocol::{decode_manifest, decode_response, input_contract, protocol_contract};
use super::{
    process, staging, ProviderExecution, ProviderExecutionIdentity, ProviderRunner, WorkflowError,
    WorkflowErrorKind, WorkflowResult,
};

pub(super) fn run(
    runner: &ProviderRunner,
    store: &ArtifactStore,
    request: &ProviderRequestEnvelope,
) -> WorkflowResult<ProviderExecution> {
    runner.limits.validate()?;
    let deadline = Instant::now() + runner.limits.max_wall_time;
    let request_bytes = canonical_request_bytes(request).map_err(input_contract)?;
    if request_bytes.len() as u64 > runner.limits.max_request_bytes {
        return resource("provider request exceeds the protocol byte limit");
    }
    let pinned = pinned(runner, deadline)?;
    ensure_deadline(deadline)?;
    let manifest_bytes = {
        let executable = launch(pinned, deadline)?;
        process::manifest(
            executable.path(),
            &runner.arguments,
            runner.limits,
            deadline,
        )?
    };
    let manifest = decode_manifest(&manifest_bytes)?;
    let negotiated = negotiate(
        &manifest,
        &CapabilityRequirement::current(request.capability),
    )
    .map_err(protocol_contract)?;
    if negotiated.contract_version != request.contract_version
        || negotiated.provider != request.provider
    {
        return protocol("provider manifest does not match the pinned request envelope");
    }
    ensure_deadline(deadline)?;
    let identity = ProviderExecutionIdentity::from_pinned(&negotiated.provider, pinned.identity())?;
    let staging_root = tempfile::tempdir()?;
    let response_bytes = {
        let executable = launch(pinned, deadline)?;
        process::execute(
            executable.path(),
            &runner.arguments,
            staging_root.path(),
            &request_bytes,
            runner.limits,
            deadline,
        )?
    };
    let raw_response = decode_response(&response_bytes, request)?;
    let payloads = staging::verify(staging_root.path(), &raw_response, runner.limits, deadline)?;
    ensure_deadline(deadline)?;
    let response = bind_provider_executable(&raw_response, identity.executable.clone())
        .map_err(protocol_contract)?;
    let bound = response.output.artifacts();
    if payloads.len() != bound.len() {
        return protocol("provider artifact binding changed the artifact set");
    }
    let mut artifacts = Vec::with_capacity(payloads.len());
    for (payload, artifact) in payloads.into_iter().zip(bound) {
        ensure_deadline(deadline)?;
        artifacts.push(commit(store, artifact, &payload.path, deadline)?);
        ensure_deadline(deadline)?;
    }
    Ok(ProviderExecution {
        manifest,
        identity,
        response,
        artifacts,
    })
}

fn commit(
    store: &ArtifactStore,
    artifact: &ProviderArtifact,
    path: &Path,
    deadline: Instant,
) -> WorkflowResult<ArtifactRecord> {
    commit_while(store, artifact, path, || Instant::now() < deadline)
}

pub(super) fn commit_while(
    store: &ArtifactStore,
    artifact: &ProviderArtifact,
    path: &Path,
    guard: impl FnMut() -> bool,
) -> WorkflowResult<ArtifactRecord> {
    store
        .put_verified_file_while(&artifact.descriptor, &artifact.record, path, guard)
        .map_err(artifact_failure)
}

fn ensure_deadline(deadline: Instant) -> WorkflowResult<()> {
    if Instant::now() >= deadline {
        return resource("provider run exceeded its wall-clock limit");
    }
    Ok(())
}

fn pinned(runner: &ProviderRunner, deadline: Instant) -> WorkflowResult<&PinnedExecutable> {
    PinnedExecutable::cached_until(&runner.pinned, &runner.program, deadline).map_err(|error| {
        let message = format!(
            "failed to pin provider executable {}",
            runner.program.display()
        );
        workflow_tool_error(message, error)
    })
}

fn launch(pinned: &PinnedExecutable, deadline: Instant) -> WorkflowResult<LaunchExecutable> {
    pinned.launch_until(deadline).map_err(tool_failure)
}

fn tool_failure(error: RuntimeError) -> WorkflowError {
    workflow_tool_error("failed to launch pinned provider executable", error)
}

fn workflow_tool_error(message: impl Into<String>, error: RuntimeError) -> WorkflowError {
    let kind = if error.kind == crate::RuntimeErrorKind::ResourceLimit {
        WorkflowErrorKind::ResourceLimit
    } else {
        WorkflowErrorKind::ToolFailure
    };
    WorkflowError::with_source(kind, message, error)
}

fn artifact_failure(error: veac_artifact::ArtifactError) -> WorkflowError {
    let kind = if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        WorkflowErrorKind::ResourceLimit
    } else {
        WorkflowErrorKind::Artifact
    };
    WorkflowError::with_source(kind, "failed to commit provider artifact", error)
}

#[cfg(test)]
mod tests;

fn protocol<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ProtocolViolation,
        message,
    ))
}

fn resource<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ResourceLimit,
        message,
    ))
}
