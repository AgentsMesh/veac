mod arguments;
mod capability;
#[cfg(test)]
mod capability_tests;
mod environment;
mod runner;
mod system_capabilities;

use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::tool::{LaunchExecutable, PinnedExecutable};
use crate::RuntimeError;
use veac_artifact::ContentDigest;

pub(in crate::executor) use arguments::for_command as arguments;
pub(in crate::executor) use arguments::for_filter_script as script_arguments;
pub use environment::{FfmpegEnvironment, FfmpegInvocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfmpegFingerprint {
    pub version: String,
    pub configuration: ContentDigest,
}

#[derive(Debug, Clone)]
pub struct SystemFfmpeg {
    binary: PathBuf,
    pinned: Arc<OnceLock<PinnedExecutable>>,
}

impl SystemFfmpeg {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            pinned: Arc::new(OnceLock::new()),
        }
    }

    fn pinned_until(&self, deadline: Instant) -> Result<&PinnedExecutable, RuntimeError> {
        PinnedExecutable::cached_until(&self.pinned, &self.binary, deadline).map_err(|error| {
            error.context(&format!("failed to run FFmpeg {}", self.binary.display()))
        })
    }

    pub(crate) fn launch_until(&self, deadline: Instant) -> Result<LaunchExecutable, RuntimeError> {
        self.pinned_until(deadline)?.launch_until(deadline)
    }

    fn run_until(
        &self,
        arguments: &[String],
        output_root: Option<&Path>,
        working_directory: Option<&Path>,
        deadline: Instant,
    ) -> Result<Output, RuntimeError> {
        let executable = self.launch_until(deadline)?;
        runner::run(
            executable.path(),
            arguments,
            runner::ProcessLimits {
                deadline,
                max_stdout_bytes: runner::MAX_STDOUT_BYTES,
                max_stderr_bytes: runner::MAX_STDERR_BYTES,
                output_root,
                working_directory,
                max_output_bytes: runner::MAX_OUTPUT_BYTES,
            },
        )
        .map_err(|error| error.context(&format!("failed to run FFmpeg {}", self.binary.display())))
    }

    pub(crate) fn capture_until(
        &self,
        arguments: &[String],
        max_stdout_bytes: u64,
        deadline: Instant,
    ) -> Result<Output, RuntimeError> {
        let executable = self.launch_until(deadline)?;
        runner::run(
            executable.path(),
            arguments,
            runner::ProcessLimits {
                deadline,
                max_stdout_bytes,
                max_stderr_bytes: runner::MAX_STDERR_BYTES,
                output_root: None,
                working_directory: None,
                max_output_bytes: runner::MAX_OUTPUT_BYTES,
            },
        )
        .map_err(|error| error.context("failed to capture FFmpeg observation"))
    }
}

fn hard_deadline() -> Instant {
    Instant::now() + runner::MAX_WALL_TIME
}

#[cfg(test)]
#[path = "process/system_tests.rs"]
mod system_tests;

impl Default for SystemFfmpeg {
    fn default() -> Self {
        Self::new("ffmpeg")
    }
}

pub(super) fn append_field(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}

pub(super) fn validate_requirements_until(
    environment: &dyn FfmpegEnvironment,
    requirements: &[veac_codegen::emitter::BackendRequirement],
    deadline: Instant,
) -> Result<(), RuntimeError> {
    capability::validate(environment, requirements, deadline)
}

pub(crate) fn ensure_success(output: &Output, operation: &str) -> Result<(), RuntimeError> {
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    let detail = detail.trim();
    let message = if detail.is_empty() {
        format!("{operation} failed ({})", output.status)
    } else {
        format!("{operation} failed: {detail} ({})", output.status)
    };
    Err(RuntimeError::new(message))
}
