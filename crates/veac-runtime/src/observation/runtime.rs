use veac_ir::ProbedStream;

use super::{command, parse, source};
use super::{DecodeObservation, DecodeRequest, DecodedFrame, FrameRequest, ObservationLimits};
use crate::asset::SystemFfprobe;
use crate::executor::SystemFfmpeg;
use crate::RuntimeError;

#[derive(Debug, Clone)]
pub struct MediaObserver {
    ffmpeg: SystemFfmpeg,
    ffprobe: SystemFfprobe,
    limits: ObservationLimits,
}

impl MediaObserver {
    pub fn new(ffmpeg: SystemFfmpeg, limits: ObservationLimits) -> Result<Self, RuntimeError> {
        Self::with_tools(ffmpeg, SystemFfprobe::default(), limits)
    }

    pub fn with_tools(
        ffmpeg: SystemFfmpeg,
        ffprobe: SystemFfprobe,
        limits: ObservationLimits,
    ) -> Result<Self, RuntimeError> {
        Ok(Self {
            ffmpeg,
            ffprobe,
            limits: limits.validate()?,
        })
    }

    pub fn frame(&self, request: &FrameRequest) -> Result<DecodedFrame, RuntimeError> {
        if !request.time.is_valid() || request.time.value < 0 {
            return Err(RuntimeError::new(
                "frame observation time must be exact and non-negative",
            ));
        }
        let deadline = self.limits.deadline()?;
        let snapshot = source::probe(&self.ffprobe, &request.source, deadline)?;
        let (stream, width, height) = selected_video(&snapshot)?;
        let time_base = stream.time_base.ok_or_else(|| {
            RuntimeError::new("selected observation stream has no exact time base")
        })?;
        let expected =
            self.limits
                .frame_bytes(width, height, request.pixel_format.bytes_per_pixel())?;
        let arguments = command::frame(
            &request.source.path,
            stream.type_index,
            request.time,
            request.pixel_format,
        )?;
        let output = self.ffmpeg.capture_until(&arguments, expected, deadline)?;
        if let Err(error) = crate::executor::ensure_success(&output, "frame observation") {
            if request.pixel_format == super::FramePixelFormat::Alpha16 {
                return Err(error.context("alpha observation requires a source alpha plane"));
            }
            return Err(error);
        }
        source::verify(&request.source)?;
        if output.stdout.len() as u64 != expected {
            return Err(RuntimeError::new(
                "frame observation returned an unexpected raster size",
            ));
        }
        Ok(DecodedFrame {
            requested_time: request.time,
            actual_pts: parse::frame_pts(&output.stderr, time_base)?,
            width,
            height,
            pixel_format: request.pixel_format,
            bytes: output.stdout,
        })
    }

    pub fn decode(&self, request: &DecodeRequest) -> Result<DecodeObservation, RuntimeError> {
        let deadline = self.limits.deadline()?;
        let snapshot = source::probe(&self.ffprobe, &request.source, deadline)?;
        let (stream, _, _) = selected_video(&snapshot)?;
        let arguments = command::decode(&request.source.path, stream.type_index)?;
        let output = self.ffmpeg.capture_until(
            &arguments,
            veac_artifact::MAX_ARTIFACT_METADATA_BYTES,
            deadline,
        )?;
        crate::executor::ensure_success(&output, "complete decode observation")?;
        source::verify(&request.source)?;
        let (decoded_frames, last_pts) = parse::decode_progress(&output.stdout)?;
        Ok(DecodeObservation {
            complete: true,
            identity: request.source.identity.clone(),
            decoded_frames,
            last_pts,
            errors: Vec::new(),
        })
    }
}

impl Default for MediaObserver {
    fn default() -> Self {
        Self::new(SystemFfmpeg::default(), ObservationLimits::default())
            .expect("default observation limits are valid")
    }
}

fn selected_video(
    snapshot: &veac_ir::MediaProbeSnapshot,
) -> Result<(&ProbedStream, u32, u32), RuntimeError> {
    let selection = snapshot
        .selected_video_stream
        .ok_or_else(|| RuntimeError::new("media observation requires one selected video stream"))?;
    let stream = snapshot
        .streams
        .iter()
        .find(|value| value.global_index == selection.global_index)
        .ok_or_else(|| RuntimeError::new("selected observation stream is missing"))?;
    let video = stream
        .video
        .as_ref()
        .ok_or_else(|| RuntimeError::new("selected observation stream has no video geometry"))?;
    Ok((stream, video.width, video.height))
}

#[cfg(test)]
#[path = "runtime/tests.rs"]
mod tests;
