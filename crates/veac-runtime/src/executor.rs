//! Fail-closed execution for emitter commands and multi-deliverable bundles.

mod bundle;
mod checkpoint;
mod contract;
mod deadline;
mod locking;
mod model;
mod output;
mod process;
mod snapshot;
mod staging;

pub use bundle::{execute_bundle, BundleExecution, BundleExecutor, TaskExecution};
pub use deadline::{BundleSetupLimits, TaskExecutionLimits};
pub(crate) use process::ensure_success;
pub use process::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation, SystemFfmpeg};

use crate::RuntimeError;

/// Verify that system FFmpeg is available and return its version line.
pub fn check_ffmpeg() -> Result<String, RuntimeError> {
    Ok(FfmpegEnvironment::fingerprint(&SystemFfmpeg::default())?.version)
}

#[cfg(test)]
#[path = "executor/tests.rs"]
mod tests;
