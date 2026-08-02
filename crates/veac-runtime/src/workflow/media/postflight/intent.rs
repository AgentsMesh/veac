use veac_artifact::MediaArtifactSpec;
use veac_ir::{MediaProbeSnapshot, ProbedStreamType, StreamChoice, StreamIntent};

use crate::workflow::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) struct ExpectedStreams {
    pub streams: StreamIntent,
    pub video_count: usize,
    pub audio_count: usize,
}

impl ExpectedStreams {
    pub(super) fn validate(&self, probe: &MediaProbeSnapshot) -> WorkflowResult<()> {
        let playable_video = probe.streams.iter().filter(|stream| {
            stream.media_type == ProbedStreamType::Video
                && !stream.disposition.attached_picture
                && !stream.disposition.timed_thumbnail
        });
        let audio = probe
            .streams
            .iter()
            .filter(|stream| stream.media_type == ProbedStreamType::Audio);
        if probe.streams.len() != self.video_count + self.audio_count
            || playable_video.count() != self.video_count
            || audio.count() != self.audio_count
        {
            return Err(WorkflowError::new(
                WorkflowErrorKind::ToolFailure,
                "derived artifact contains an unexpected media stream set",
            ));
        }
        Ok(())
    }
}

pub(super) fn expected(spec: &MediaArtifactSpec) -> ExpectedStreams {
    let (video_count, audio_count) = match spec {
        MediaArtifactSpec::ProxyAudio(_) => (0, 1),
        MediaArtifactSpec::SourceSegment(value) => (1, usize::from(value.audio.is_some())),
        _ => (1, 0),
    };
    ExpectedStreams {
        streams: StreamIntent {
            video: choice(video_count),
            audio: choice(audio_count),
        },
        video_count,
        audio_count,
    }
}

fn choice(count: usize) -> StreamChoice {
    if count == 0 {
        StreamChoice::Disabled
    } else {
        StreamChoice::Auto
    }
}
