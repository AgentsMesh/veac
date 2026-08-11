use veac_artifact::{
    MediaArtifactSpec, OpticalFlowMethod, OpticalFlowSpec, ProxyAudioSpec, ProxyVideoSpec,
    SourceSegmentAudioSpec, SourceSegmentSpec, ThumbnailSpec, WaveformSpec,
};
use veac_build::ProjectBackendError;
use veac_ir::StreamSelection;
use veac_project::{MediaDerivation, ProjectOpticalFlowMethod, ProjectStreamSelection};

mod time;

use time::{clock, common_times, rate, time};

pub(super) fn convert(value: &MediaDerivation) -> Result<MediaArtifactSpec, ProjectBackendError> {
    Ok(match value {
        MediaDerivation::ProxyVideo {
            source_stream,
            source_clock,
            width,
            height,
            frame_rate,
            crf,
            ..
        } => MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(*source_stream),
            source_clock: clock(*source_clock)?,
            width: *width,
            height: *height,
            frame_rate: rate(*frame_rate)?,
            crf: *crf,
        }),
        MediaDerivation::ProxyAudio {
            source_stream,
            source_clock,
            sample_rate,
            channels,
            ..
        } => MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(*source_stream),
            source_clock: clock(*source_clock)?,
            sample_rate: *sample_rate,
            channels: *channels,
        }),
        MediaDerivation::Thumbnail {
            source_stream,
            at,
            width,
            height,
            ..
        } => MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: stream(*source_stream),
            at: time(*at)?,
            width: *width,
            height: *height,
        }),
        MediaDerivation::Waveform {
            source_stream,
            source_clock,
            sample_rate,
            width,
            height,
            color,
            ..
        } => MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(*source_stream),
            source_clock: clock(*source_clock)?,
            sample_rate: *sample_rate,
            width: *width,
            height: *height,
            color: color.clone(),
        }),
        MediaDerivation::OpticalFlow {
            source_stream,
            source_clock,
            width,
            height,
            frame_rate,
            method,
            ..
        } => MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(*source_stream),
            source_clock: clock(*source_clock)?,
            width: *width,
            height: *height,
            frame_rate: rate(*frame_rate)?,
            method: match method {
                ProjectOpticalFlowMethod::BlockMatching => OpticalFlowMethod::BlockMatching,
                ProjectOpticalFlowMethod::MotionCompensated => OpticalFlowMethod::MotionCompensated,
            },
        }),
        MediaDerivation::SourceSegment {
            video_stream,
            start,
            duration,
            width,
            height,
            frame_rate,
            audio,
            crf,
            ..
        } => {
            let (start, duration) = common_times(*start, *duration)?;
            MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
                video_stream: stream(*video_stream),
                start,
                duration,
                width: *width,
                height: *height,
                frame_rate: rate(*frame_rate)?,
                audio: audio.map(|value| SourceSegmentAudioSpec {
                    source_stream: stream(value.source_stream),
                    sample_rate: value.sample_rate,
                    channels: value.channels,
                }),
                crf: *crf,
            })
        }
    })
}

fn stream(value: ProjectStreamSelection) -> StreamSelection {
    StreamSelection {
        global_index: value.global_index,
        type_index: value.type_index,
    }
}

#[cfg(test)]
#[path = "spec/tests.rs"]
mod tests;
