use veac_ir::{
    Clip, ClipSource, HashAlgorithm, ItemId, Material, Precondition, ProjectEnvelope, RationalTime,
    TimeRange, Track, TrackKind,
};

use super::support;
use crate::{InputArtifact, MediaType, ProviderResult};

#[path = "media/output.rs"]
mod output;
pub(super) use output::output_material;
#[path = "media/operation.rs"]
mod operation;
pub(super) use operation::{
    add_track_precondition, append_generated_clip, append_material, append_replacement,
    append_stem_clip,
};

#[derive(Clone, Copy)]
pub(super) enum RequiredStream {
    Audio,
    Video,
}

pub(super) fn source_clip<'a>(
    project: &'a ProjectEnvelope,
    id: &ItemId,
    input: &InputArtifact,
    stream: RequiredStream,
    require_unlocked: bool,
) -> ProviderResult<(&'a Track, &'a Clip)> {
    let (track, clip) = support::clip(project, id)?;
    let component_matches = match stream {
        RequiredStream::Audio => track.kind == TrackKind::Audio && clip.audio.is_some(),
        RequiredStream::Video => {
            matches!(track.kind, TrackKind::Video | TrackKind::Visual) && clip.visual.is_some()
        }
    };
    if !component_matches || (require_unlocked && track.state.locked) {
        return support::invalid("provider source target has incompatible or locked media");
    }
    let ClipSource::Media { material_id } = &clip.source else {
        return support::invalid("provider source target must be a media clip");
    };
    let material = project
        .project
        .materials
        .iter()
        .find(|value| value.id == *material_id)
        .ok_or_else(|| invalid("provider source material does not exist"))?;
    validate_input(project, material, input, stream, clip.record_range)?;
    Ok((track, clip))
}

pub(super) fn source_preconditions(track: &Track, clip: &Clip) -> Vec<Precondition> {
    vec![
        Precondition::TrackUnlocked {
            track_id: track.id.clone(),
        },
        Precondition::ClipExists {
            clip_id: clip.id.clone(),
        },
        Precondition::ClipSourceEquals {
            clip_id: clip.id.clone(),
            source: Box::new(clip.source.clone()),
        },
    ]
}

pub(super) fn audio_target<'a>(
    project: &'a ProjectEnvelope,
    sequence_id: &veac_ir::SequenceId,
    track_id: &veac_ir::TrackId,
) -> ProviderResult<&'a Track> {
    let track = project
        .project
        .sequences
        .iter()
        .find(|value| value.id == *sequence_id)
        .and_then(|sequence| sequence.tracks.iter().find(|value| value.id == *track_id))
        .ok_or_else(|| invalid("provider audio target track does not exist"))?;
    if track.kind != TrackKind::Audio || track.state.locked {
        support::invalid("provider audio target must be an unlocked audio track")
    } else {
        Ok(track)
    }
}

pub(super) fn normalized_range(
    project: &ProjectEnvelope,
    value: TimeRange,
) -> ProviderResult<TimeRange> {
    super::super::time::range_to_timebase(value, project.project.timebase)
}

pub(super) fn normalized_time(
    project: &ProjectEnvelope,
    value: RationalTime,
) -> ProviderResult<RationalTime> {
    super::super::time::convert(value, project.project.timebase)
}

fn validate_input(
    project: &ProjectEnvelope,
    material: &Material,
    input: &InputArtifact,
    stream: RequiredStream,
    record_range: TimeRange,
) -> ProviderResult<()> {
    let expected_type = match stream {
        RequiredStream::Audio => MediaType::Audio,
        RequiredStream::Video => MediaType::Video,
    };
    let identity_matches = material.identity.as_ref().is_some_and(|identity| {
        identity.algorithm == HashAlgorithm::Sha256 && identity.digest == input.content.value
    });
    if input.media_type != expected_type
        || !identity_matches
        || input
            .range
            .map(|value| normalized_range(project, value))
            .transpose()?
            != Some(record_range)
    {
        return support::invalid("provider request input does not match its source clip");
    }
    if let Some(index) = input.stream_index {
        let selected = match stream {
            RequiredStream::Audio => material
                .probe
                .as_ref()
                .and_then(|probe| probe.selected_audio_stream),
            RequiredStream::Video => material
                .probe
                .as_ref()
                .and_then(|probe| probe.selected_video_stream),
        };
        if selected.map(|value| value.global_index) != Some(index) {
            return support::invalid("provider request stream does not match its source clip");
        }
    }
    Ok(())
}

fn invalid(message: &str) -> crate::ProviderError {
    crate::ProviderError::new(crate::ProviderErrorKind::InvalidContract, message)
}
