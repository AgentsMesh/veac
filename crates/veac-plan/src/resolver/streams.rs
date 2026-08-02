use veac_ir::{MediaProbeSnapshot, ProbedStream, ProbedStreamType, StreamSelection};

use crate::{ResolvedAudioStream, ResolvedVideoStream};

pub(super) fn resolved_streams(
    probe: &MediaProbeSnapshot,
) -> (Option<ResolvedVideoStream>, Option<ResolvedAudioStream>) {
    (
        selected_video(probe.selected_video_stream, &probe.streams),
        selected_audio(probe.selected_audio_stream, &probe.streams),
    )
}

fn selected_video(
    selection: Option<StreamSelection>,
    streams: &[ProbedStream],
) -> Option<ResolvedVideoStream> {
    let selection = selection?;
    let stream = selected(selection, ProbedStreamType::Video, streams)?;
    Some(ResolvedVideoStream {
        selection,
        codec: stream.codec.clone(),
        start_time: stream.start_time,
        duration: stream.duration,
        disposition: stream.disposition,
        info: stream.video.clone()?,
    })
}

fn selected_audio(
    selection: Option<StreamSelection>,
    streams: &[ProbedStream],
) -> Option<ResolvedAudioStream> {
    let selection = selection?;
    let stream = selected(selection, ProbedStreamType::Audio, streams)?;
    Some(ResolvedAudioStream {
        selection,
        codec: stream.codec.clone(),
        start_time: stream.start_time,
        duration: stream.duration,
        disposition: stream.disposition,
        info: stream.audio.clone()?,
    })
}

fn selected(
    selection: StreamSelection,
    kind: ProbedStreamType,
    streams: &[ProbedStream],
) -> Option<&ProbedStream> {
    streams.iter().find(|stream| {
        stream.global_index == selection.global_index
            && stream.type_index == selection.type_index
            && stream.media_type == kind
    })
}
