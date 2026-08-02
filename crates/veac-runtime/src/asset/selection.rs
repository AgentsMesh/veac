use veac_ir::{ProbedStream, ProbedStreamType, StreamChoice, StreamIntent, StreamSelection};

use super::ProbeError;

pub(super) fn select_streams(
    streams: &[ProbedStream],
    intent: StreamIntent,
) -> Result<(Option<StreamSelection>, Option<StreamSelection>), ProbeError> {
    Ok((
        select(streams, intent.video, ProbedStreamType::Video)?,
        select(streams, intent.audio, ProbedStreamType::Audio)?,
    ))
}

fn select(
    streams: &[ProbedStream],
    choice: StreamChoice,
    media_type: ProbedStreamType,
) -> Result<Option<StreamSelection>, ProbeError> {
    match choice {
        StreamChoice::Disabled => Ok(None),
        StreamChoice::Auto => Ok(auto(streams, media_type).map(selection)),
        StreamChoice::GlobalIndex { global_index } => streams
            .iter()
            .find(|stream| stream.global_index == global_index)
            .filter(|stream| eligible(stream, media_type))
            .map(selection)
            .map(Some)
            .ok_or(ProbeError::StreamSelection {
                media_type,
                global_index,
            }),
    }
}

fn auto(streams: &[ProbedStream], media_type: ProbedStreamType) -> Option<&ProbedStream> {
    let mut eligible = streams.iter().filter(|stream| eligible(stream, media_type));
    let first = eligible.next()?;
    Some(
        std::iter::once(first)
            .chain(eligible)
            .find(|stream| stream.disposition.default)
            .unwrap_or(first),
    )
}

fn eligible(stream: &ProbedStream, media_type: ProbedStreamType) -> bool {
    stream.media_type == media_type
        && (media_type != ProbedStreamType::Video
            || (!stream.disposition.attached_picture && !stream.disposition.timed_thumbnail))
}

fn selection(stream: &ProbedStream) -> StreamSelection {
    StreamSelection {
        global_index: stream.global_index,
        type_index: stream.type_index,
    }
}
