use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use veac_artifact::{ArtifactRecord, FullRenderSegmentContract};
use veac_codegen::emitter::BackendCapabilityKind;
use veac_ir::{MediaIdentity, MediaProbeSnapshot, StreamIntent};
use veac_runtime::asset::SystemFfprobe;
use veac_runtime::executor::{
    FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation, SystemFfmpeg,
};

use super::{result, Environment};
use crate::error::CliResult;

pub(crate) struct SystemEnvironment {
    ffmpeg: SystemFfmpeg,
    ffprobe: SystemFfprobe,
}

impl SystemEnvironment {
    pub(crate) fn new(binary: impl Into<std::path::PathBuf>) -> Self {
        Self::with_tools(binary, "ffprobe")
    }

    pub(crate) fn with_tools(
        ffmpeg: impl Into<std::path::PathBuf>,
        ffprobe: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            ffmpeg: SystemFfmpeg::new(ffmpeg),
            ffprobe: SystemFfprobe::new(ffprobe),
        }
    }
}

impl Default for SystemEnvironment {
    fn default() -> Self {
        Self::new("ffmpeg")
    }
}

impl Environment for SystemEnvironment {
    fn identity(&self, path: &Path) -> CliResult<MediaIdentity> {
        result::probe(
            veac_runtime::asset::sha256_identity(path),
            "IDENTITY_FAILED",
        )
    }

    fn probe(&self, path: &Path, intent: StreamIntent) -> CliResult<MediaProbeSnapshot> {
        result::probe(self.ffprobe.probe_with_intent(path, intent), "PROBE_FAILED")
    }

    fn probe_until(
        &self,
        path: &Path,
        intent: StreamIntent,
        deadline: Instant,
    ) -> CliResult<MediaProbeSnapshot> {
        result::probe(
            self.ffprobe.probe_with_intent_until(path, intent, deadline),
            "PROBE_FAILED",
        )
    }

    fn validate_render_segment(
        &self,
        path: &Path,
        contract: &FullRenderSegmentContract,
        record: &ArtifactRecord,
        deadline: Instant,
    ) -> CliResult {
        result::workflow(
            veac_runtime::workflow::FullRenderSegmentValidator::new(self.ffprobe.clone())
                .validate_until(path, contract, record, deadline),
            "RENDER_SEGMENT_POSTFLIGHT_FAILED",
        )
    }

    fn execute_ffmpeg(&self, invocation: FfmpegInvocation<'_>) -> CliResult {
        result::runtime(
            FfmpegEnvironment::execute(&self.ffmpeg, invocation),
            "RENDER_FAILED",
        )
    }

    fn ffmpeg_encoders(&self) -> CliResult<BTreeSet<String>> {
        runtime(self.ffmpeg.encoders(), "FFMPEG_ENCODERS_FAILED")
    }

    fn ffmpeg_muxers(&self) -> CliResult<BTreeSet<String>> {
        runtime(self.ffmpeg.muxers(), "FFMPEG_MUXERS_FAILED")
    }

    fn ffmpeg_decoders(&self) -> CliResult<BTreeSet<String>> {
        runtime(self.ffmpeg.decoders(), "FFMPEG_DECODERS_FAILED")
    }

    fn ffmpeg_demuxers(&self) -> CliResult<BTreeSet<String>> {
        runtime(self.ffmpeg.demuxers(), "FFMPEG_DEMUXERS_FAILED")
    }

    fn ffmpeg_filters(&self) -> CliResult<BTreeSet<String>> {
        runtime(self.ffmpeg.filters(), "FFMPEG_FILTERS_FAILED")
    }

    fn ffmpeg_hardware_backends(&self) -> CliResult<BTreeSet<String>> {
        runtime(
            self.ffmpeg.hardware_backends(),
            "FFMPEG_HARDWARE_BACKENDS_FAILED",
        )
    }

    fn ffmpeg_hardware_devices(&self) -> CliResult<BTreeSet<String>> {
        runtime(
            self.ffmpeg.hardware_devices(),
            "FFMPEG_HARDWARE_DEVICES_FAILED",
        )
    }

    fn ffmpeg_fingerprint(&self) -> CliResult<FfmpegFingerprint> {
        runtime(self.ffmpeg.fingerprint(), "FFMPEG_VERSION_FAILED")
    }

    fn ffmpeg_fingerprint_until(&self, deadline: Instant) -> CliResult<FfmpegFingerprint> {
        runtime(
            FfmpegEnvironment::fingerprint_until(&self.ffmpeg, deadline),
            "FFMPEG_VERSION_FAILED",
        )
    }

    fn ffmpeg_capability_until(
        &self,
        kind: BackendCapabilityKind,
        deadline: Instant,
    ) -> CliResult<BTreeSet<String>> {
        runtime(
            FfmpegEnvironment::capability_until(&self.ffmpeg, kind, deadline),
            "FFMPEG_CAPABILITY_FAILED",
        )
    }
}

fn runtime<T>(value: Result<T, veac_runtime::RuntimeError>, code: &str) -> CliResult<T> {
    result::runtime(value, code)
}
