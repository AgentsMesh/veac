use std::fmt;

use veac_ir::{MediaProbeSnapshot, ProbedStream, ProbedStreamType, StreamSelection};

/// A non-panicking human-readable view over a possibly external snapshot.
pub struct ProbeDisplay<'a> {
    snapshot: &'a MediaProbeSnapshot,
}

impl<'a> ProbeDisplay<'a> {
    pub fn new(snapshot: &'a MediaProbeSnapshot) -> Self {
        Self { snapshot }
    }
}

impl fmt::Display for ProbeDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(duration) = self.snapshot.container_duration {
            writeln!(
                formatter,
                "Duration:    {}/{}s",
                duration.value, duration.timescale
            )?;
        } else {
            writeln!(formatter, "Duration:    unknown")?;
        }
        if let Some(stream) = selected_video(self.snapshot) {
            if let Some(video) = &stream.video {
                writeln!(formatter, "Resolution:  {}x{}", video.width, video.height)?;
                writeln!(formatter, "Video codec: {}", stream.codec)?;
            }
        }
        if let Some(stream) = selected_audio(self.snapshot) {
            if let Some(audio) = &stream.audio {
                writeln!(formatter, "Audio codec: {}", stream.codec)?;
                writeln!(formatter, "Sample rate: {} Hz", audio.sample_rate)?;
            }
        } else {
            writeln!(formatter, "Audio:       none")?;
        }
        Ok(())
    }
}

pub fn selected_video(snapshot: &MediaProbeSnapshot) -> Option<&ProbedStream> {
    selected(snapshot, snapshot.selected_video_stream?).filter(|stream| {
        stream.media_type == ProbedStreamType::Video
            && stream.video.is_some()
            && !stream.disposition.attached_picture
            && !stream.disposition.timed_thumbnail
    })
}

pub fn selected_audio(snapshot: &MediaProbeSnapshot) -> Option<&ProbedStream> {
    selected(snapshot, snapshot.selected_audio_stream?)
        .filter(|stream| stream.media_type == ProbedStreamType::Audio && stream.audio.is_some())
}

fn selected(snapshot: &MediaProbeSnapshot, selection: StreamSelection) -> Option<&ProbedStream> {
    snapshot.streams.iter().find(|stream| {
        stream.global_index == selection.global_index && stream.type_index == selection.type_index
    })
}
