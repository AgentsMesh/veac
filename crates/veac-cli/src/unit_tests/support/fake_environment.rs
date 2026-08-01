use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

use veac_artifact::{ArtifactRecord, FullRenderSegmentContract};
use veac_ir::{ImageSequencePattern, MediaIdentity, MediaProbeSnapshot, StreamIntent};
use veac_runtime::executor::{FfmpegFingerprint, FfmpegInvocation};

use super::{snapshot, FakeEnvironment};
use crate::environment::Environment;
use crate::error::{CliError, CliResult};

impl Environment for FakeEnvironment {
    fn identity(&self, path: &Path) -> CliResult<MediaIdentity> {
        self.identity_paths.borrow_mut().push(path.to_owned());
        Ok(self.observed.clone())
    }

    fn probe(&self, path: &Path, intent: StreamIntent) -> CliResult<MediaProbeSnapshot> {
        self.probe_paths.borrow_mut().push(path.to_owned());
        if self.fail_probe {
            return Err(CliError::new("FAKE_PROBE", "probe failed"));
        }
        let bytes = std::fs::read(path).ok();
        let local_output = bytes
            .as_deref()
            .is_some_and(|value| value == b"rendered" || value == b"semantic-invalid");
        let observed =
            if local_output || path.file_name().is_some_and(|value| value == "payload.bin") {
                veac_runtime::asset::sha256_identity(path)
                    .map_err(|error| CliError::new("FAKE_PROBE", error.to_string()))?
            } else {
                self.observed.clone()
            };
        let mut probe = snapshot(intent, observed);
        if bytes.as_deref() == Some(b"semantic-invalid") {
            probe.streams[0].codec = "hevc".to_owned();
        }
        Ok(probe)
    }

    fn validate_render_segment(
        &self,
        path: &Path,
        contract: &FullRenderSegmentContract,
        record: &ArtifactRecord,
        deadline: Instant,
    ) -> CliResult {
        if self.fail_probe {
            return Err(CliError::new("FAKE_PROBE", "probe failed"));
        }
        let identity = veac_ir::MediaIdentity {
            algorithm: veac_ir::HashAlgorithm::Sha256,
            digest: record.content.value.clone(),
        };
        let mut probe = snapshot(
            veac_ir::StreamIntent {
                video: veac_ir::StreamChoice::Auto,
                audio: veac_ir::StreamChoice::Disabled,
            },
            identity,
        );
        let profile = contract.media_profile();
        let duration = contract.range().duration;
        probe.container_duration = Some(duration);
        probe.streams[0].duration = Some(duration);
        let video = probe.streams[0].video.as_mut().unwrap();
        (video.width, video.height) = (profile.width(), profile.height());
        video.frame_rate = Some(profile.frame_rate());
        if std::fs::read(path).is_ok_and(|bytes| bytes == b"semantic-invalid") {
            probe.streams[0].codec = "hevc".to_owned();
        }
        crate::environment::render_segment::validate(path, contract, record, deadline, |_| {
            Ok(probe)
        })
    }

    fn execute_ffmpeg(&self, invocation: FfmpegInvocation<'_>) -> CliResult {
        let args = invocation.arguments();
        let inputs = args
            .windows(2)
            .filter(|pair| pair[0] == "-i")
            .map(|pair| {
                std::fs::read(&pair[1])
                    .map_err(|error| CliError::new("FAKE_INPUT", error.to_string()))
            })
            .collect::<CliResult<Vec<_>>>()?;
        self.consumed_inputs.borrow_mut().push(inputs);
        self.executed.borrow_mut().push(args.to_vec());
        if self.fail_execute {
            return Err(CliError::new("FAKE_RENDER", "render failed"));
        }
        if let Some(prefix) =
            option(args, "-passlogfile").filter(|_| option(args, "-pass") == Some("1"))
        {
            std::fs::write(format!("{prefix}-0.log"), b"passlog").unwrap();
            return Ok(());
        }
        let Some(output) = args.last() else {
            return Err(CliError::new("FAKE_RENDER", "missing output argument"));
        };
        let output_path = Path::new(output);
        let pattern = output_path
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(ImageSequencePattern::parse);
        if let Some(pattern) = pattern {
            std::fs::write(
                output_path.with_file_name(pattern.format_index(1)),
                b"frame",
            )
            .unwrap();
        } else {
            std::fs::write(output, b"rendered").unwrap();
        }
        Ok(())
    }

    fn ffmpeg_encoders(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.encoders.clone())
    }

    fn ffmpeg_muxers(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.muxers.clone())
    }

    fn ffmpeg_decoders(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.decoders.clone())
    }

    fn ffmpeg_demuxers(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.demuxers.clone())
    }

    fn ffmpeg_filters(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.filters.clone())
    }

    fn ffmpeg_hardware_backends(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.hardware_backends.clone())
    }

    fn ffmpeg_hardware_devices(&self) -> CliResult<BTreeSet<String>> {
        Ok(self.hardware_devices.clone())
    }

    fn ffmpeg_fingerprint(&self) -> CliResult<FfmpegFingerprint> {
        if self.fail_version {
            return Err(CliError::new("FAKE_VERSION", "version failed"));
        }
        Ok(FfmpegFingerprint {
            version: "ffmpeg version fixture".to_owned(),
            configuration: self.fingerprint_configuration.clone(),
        })
    }
}

fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
