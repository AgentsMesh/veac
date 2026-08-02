use std::collections::BTreeSet;
use std::time::Instant;

use veac_codegen::emitter::BackendCapabilityKind;
use veac_runtime::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use veac_runtime::RuntimeError;

use crate::environment::Environment;

pub(super) struct RuntimeEnvironment<'a>(pub &'a dyn Environment);

impl FfmpegEnvironment for RuntimeEnvironment<'_> {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.0.ffmpeg_fingerprint().map_err(runtime_error)
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_encoders().map_err(runtime_error)
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_muxers().map_err(runtime_error)
    }

    fn decoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_decoders().map_err(runtime_error)
    }

    fn demuxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_demuxers().map_err(runtime_error)
    }

    fn filters(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_filters().map_err(runtime_error)
    }

    fn hardware_backends(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_hardware_backends().map_err(runtime_error)
    }

    fn hardware_devices(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.0.ffmpeg_hardware_devices().map_err(runtime_error)
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        self.0.execute_ffmpeg(invocation).map_err(runtime_error)
    }

    fn fingerprint_until(&self, deadline: Instant) -> Result<FfmpegFingerprint, RuntimeError> {
        self.0
            .ffmpeg_fingerprint_until(deadline)
            .map_err(runtime_error)
    }

    fn capability_until(
        &self,
        kind: BackendCapabilityKind,
        deadline: Instant,
    ) -> Result<BTreeSet<String>, RuntimeError> {
        self.0
            .ffmpeg_capability_until(kind, deadline)
            .map_err(runtime_error)
    }
}

fn runtime_error(error: crate::CliError) -> RuntimeError {
    if error.is_resource_limit() {
        RuntimeError::resource_limit(error.to_string())
    } else {
        RuntimeError::new(error.to_string())
    }
}

#[cfg(test)]
#[path = "adapter/tests.rs"]
mod tests;
