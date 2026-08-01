use crate::*;

pub(super) fn sequence_duration(project: &Project, id: &SequenceId) -> Option<RationalTime> {
    sequence(project, id)?
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

pub(super) fn source_exists(
    project: &Project,
    sequence_id: &SequenceId,
    source: &AudioMixSource,
) -> bool {
    match source {
        AudioMixSource::Master => true,
        AudioMixSource::Track { track_id } => source_track_exists(project, sequence_id, track_id),
        AudioMixSource::Bus { bus_id } => source_bus_exists(project, sequence_id, bus_id),
    }
}

pub(super) fn source_track_exists(
    project: &Project,
    sequence_id: &SequenceId,
    track_id: &TrackId,
) -> bool {
    audio_tracks(project, sequence_id).any(|track| track.id == *track_id)
}

pub(super) fn source_bus_exists(
    project: &Project,
    sequence_id: &SequenceId,
    bus_id: &BusId,
) -> bool {
    audio_tracks(project, sequence_id).any(|track| {
        matches!(&track.routing, TrackRouting::AudioBus { bus_id: value } if value == bus_id)
    })
}

pub(super) fn image_extension(value: &str, format: ImageFormat) -> bool {
    extension_is(
        value,
        match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Exr => "exr",
        },
    )
}

pub(super) fn extension_is(value: &str, expected: &str) -> bool {
    value
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case(expected))
}

fn sequence<'a>(project: &'a Project, id: &SequenceId) -> Option<&'a Sequence> {
    project.sequences.iter().find(|value| value.id == *id)
}

fn audio_tracks<'a>(
    project: &'a Project,
    sequence_id: &SequenceId,
) -> impl Iterator<Item = &'a Track> {
    sequence(project, sequence_id)
        .into_iter()
        .flat_map(|value| &value.tracks)
        .filter(|track| matches!(track.kind, TrackKind::Video | TrackKind::Audio))
}
