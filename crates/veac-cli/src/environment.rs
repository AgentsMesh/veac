use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use veac_artifact::{ArtifactRecord, FullRenderSegmentContract};
use veac_codegen::emitter::BackendCapabilityKind;
use veac_ir::{MediaIdentity, MediaProbeSnapshot, StreamIntent};
use veac_runtime::executor::{FfmpegFingerprint, FfmpegInvocation};

use crate::error::CliResult;

pub(crate) mod render_segment;
mod result;
mod system;

pub(crate) use system::SystemEnvironment;

pub(crate) trait Environment {
    fn identity(&self, path: &Path) -> CliResult<MediaIdentity>;
    fn probe(&self, path: &Path, intent: StreamIntent) -> CliResult<MediaProbeSnapshot>;
    fn probe_until(
        &self,
        path: &Path,
        intent: StreamIntent,
        _deadline: Instant,
    ) -> CliResult<MediaProbeSnapshot> {
        self.probe(path, intent)
    }
    fn validate_render_segment(
        &self,
        path: &Path,
        contract: &FullRenderSegmentContract,
        record: &ArtifactRecord,
        deadline: Instant,
    ) -> CliResult {
        render_segment::validate(path, contract, record, deadline, |intent| {
            self.probe_until(path, intent, deadline)
        })
    }
    fn ffmpeg_encoders(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_muxers(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_decoders(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_demuxers(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_filters(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_hardware_backends(&self) -> CliResult<BTreeSet<String>>;
    fn ffmpeg_hardware_devices(&self) -> CliResult<BTreeSet<String>>;
    fn execute_ffmpeg(&self, invocation: FfmpegInvocation<'_>) -> CliResult;
    fn ffmpeg_fingerprint(&self) -> CliResult<FfmpegFingerprint>;
    fn ffmpeg_fingerprint_until(&self, deadline: Instant) -> CliResult<FfmpegFingerprint> {
        ensure_deadline(deadline)?;
        let value = self.ffmpeg_fingerprint()?;
        ensure_deadline(deadline)?;
        Ok(value)
    }
    fn ffmpeg_capability_until(
        &self,
        kind: BackendCapabilityKind,
        deadline: Instant,
    ) -> CliResult<BTreeSet<String>> {
        ensure_deadline(deadline)?;
        let value = match kind {
            BackendCapabilityKind::Encoder => self.ffmpeg_encoders(),
            BackendCapabilityKind::Decoder => self.ffmpeg_decoders(),
            BackendCapabilityKind::Muxer => self.ffmpeg_muxers(),
            BackendCapabilityKind::Demuxer => self.ffmpeg_demuxers(),
            BackendCapabilityKind::Filter => self.ffmpeg_filters(),
            BackendCapabilityKind::HardwareBackend => self.ffmpeg_hardware_backends(),
            BackendCapabilityKind::HardwareDevice => self.ffmpeg_hardware_devices(),
        }?;
        ensure_deadline(deadline)?;
        Ok(value)
    }
}

fn ensure_deadline(deadline: Instant) -> CliResult {
    if Instant::now() < deadline {
        Ok(())
    } else {
        Err(crate::CliError::resource_limit(
            "EXECUTION_LIMIT",
            "FFmpeg setup exceeded its wall-clock limit",
        ))
    }
}
