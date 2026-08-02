use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::limits::ProviderResourceLimits;
use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

mod child;

pub const PROVIDER_MODE_ENV: &str = "VEAC_PROVIDER_MODE";
pub const PROVIDER_STAGING_ENV: &str = "VEAC_PROVIDER_STAGING";

pub(super) fn manifest(
    program: &Path,
    arguments: &[OsString],
    limits: ProviderResourceLimits,
    deadline: Instant,
) -> WorkflowResult<Vec<u8>> {
    limits.validate()?;
    validate_deadline(limits, deadline)?;
    let mut command = Command::new(program);
    command
        .args(arguments)
        .env(PROVIDER_MODE_ENV, "manifest")
        .env_remove(PROVIDER_STAGING_ENV);
    checked(child::run(command, None, limits, deadline)?, "manifest")
}

pub(super) fn execute(
    program: &Path,
    arguments: &[OsString],
    staging: &Path,
    request: &[u8],
    limits: ProviderResourceLimits,
    deadline: Instant,
) -> WorkflowResult<Vec<u8>> {
    limits.validate()?;
    validate_deadline(limits, deadline)?;
    if request.len() as u64 > limits.max_request_bytes {
        return resource("provider request exceeds the protocol byte limit");
    }
    let mut command = Command::new(program);
    command
        .args(arguments)
        .env(PROVIDER_MODE_ENV, "execute")
        .env(PROVIDER_STAGING_ENV, staging);
    checked(
        child::run(command, Some(request), limits, deadline)?,
        "execution",
    )
}

fn validate_deadline(limits: ProviderResourceLimits, deadline: Instant) -> WorkflowResult<()> {
    let remaining = deadline.checked_duration_since(Instant::now());
    if remaining.is_none_or(|value| value.is_zero() || value > limits.max_wall_time) {
        return Err(WorkflowError::new(
            WorkflowErrorKind::InvalidContract,
            "provider process deadline is outside the configured policy",
        ));
    }
    Ok(())
}

fn checked(output: child::ChildOutput, phase: &str) -> WorkflowResult<Vec<u8>> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WorkflowError::new(
            WorkflowErrorKind::ToolFailure,
            format!("provider {phase} failed: {}", stderr.trim()),
        ));
    }
    Ok(output.stdout)
}

fn resource<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ResourceLimit,
        message,
    ))
}
