use veac_artifact::ArtifactKind;
use veac_ir::{MaterialKind, ProjectEnvelope, TimeRange};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use crate::{
    AudioClipInsertion, DubbingApplication, DubbingRequest, DubbingResult,
    GeneratedAudioApplication, ProviderArtifact, ProviderResult, SpeechResult, TtsRequest,
};

pub(super) fn tts(
    project: &ProjectEnvelope,
    request: &TtsRequest,
    result: &SpeechResult,
    context: &GeneratedAudioApplication,
) -> ProviderResult<BuiltApplication> {
    if result.audio.kind() != ArtifactKind::Speech {
        return support::invalid("TTS application requires a speech artifact");
    }
    let expected = media::normalized_range(project, result.range)?;
    if request
        .target_range
        .map(|value| media::normalized_range(project, value))
        .transpose()?
        .is_some_and(|value| value != expected)
    {
        return support::invalid("TTS output range does not match its request target");
    }
    insert_audio(project, &result.audio, &context.output, expected)
}

pub(super) fn dubbing(
    project: &ProjectEnvelope,
    request: &DubbingRequest,
    result: &DubbingResult,
    context: &DubbingApplication,
) -> ProviderResult<BuiltApplication> {
    if result.audio.kind() != ArtifactKind::Speech {
        return support::invalid("dubbing application requires a speech artifact");
    }
    let expected = dubbing_range(project, request, result)?;
    let (source_track, source_clip) = media::source_clip(
        project,
        &context.source_clip_id,
        &request.audio,
        RequiredStream::Audio,
        true,
    )?;
    let mut built = insert_audio(project, &result.audio, &context.output, expected)?;
    built
        .preconditions
        .extend(media::source_preconditions(source_track, source_clip));
    Ok(built)
}

fn insert_audio(
    project: &ProjectEnvelope,
    artifact: &ProviderArtifact,
    context: &AudioClipInsertion,
    expected: TimeRange,
) -> ProviderResult<BuiltApplication> {
    let record_range = media::normalized_range(project, context.record_range)?;
    let source_start = media::normalized_time(project, context.source_start)?;
    if record_range != expected || context.before_id.is_some() && context.after_id.is_some() {
        return support::invalid("provider audio clip range or insertion anchor is invalid");
    }
    let track = media::audio_target(project, &context.sequence_id, &context.track_id)?;
    media::output_material(
        project,
        artifact,
        &context.material,
        MaterialKind::Audio,
        RequiredStream::Audio,
        source_start,
        record_range.duration,
    )?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    media::append_material(&mut built, &context.material, artifact)?;
    media::append_generated_clip(&mut built, context, artifact, record_range, source_start)?;
    media::add_track_precondition(&mut built, track);
    Ok(built)
}

fn dubbing_range(
    project: &ProjectEnvelope,
    request: &DubbingRequest,
    result: &DubbingResult,
) -> ProviderResult<TimeRange> {
    if request.turns.is_empty() || request.turns.len() != result.turns.len() {
        return support::invalid("dubbing result must bind every requested turn");
    }
    let mut rendered = Vec::with_capacity(result.turns.len());
    for (requested, output) in request.turns.iter().zip(&result.turns) {
        if requested.id != output.id
            || media::normalized_range(project, requested.source_range)?
                != media::normalized_range(project, output.source_range)?
        {
            return support::invalid("dubbing turn does not match its request");
        }
        rendered.push(media::normalized_range(project, output.rendered_range)?);
    }
    if rendered.windows(2).any(|pair| match pair[0].end() {
        Ok(end) => end > pair[1].start,
        Err(_) => true,
    }) {
        return support::invalid("dubbed turns must be ordered and non-overlapping");
    }
    let start = rendered[0].start;
    let end = rendered.last().unwrap().end().map_err(|error| {
        crate::ProviderError::with_source(
            crate::ProviderErrorKind::InvalidContract,
            "dubbing rendered range overflows",
            error,
        )
    })?;
    let value = end.value.checked_sub(start.value).ok_or_else(|| {
        crate::ProviderError::new(
            crate::ProviderErrorKind::InvalidContract,
            "dubbing rendered duration overflows",
        )
    })?;
    let duration = veac_ir::RationalTime::new(value, start.timescale).map_err(|error| {
        crate::ProviderError::with_source(
            crate::ProviderErrorKind::InvalidContract,
            "dubbing rendered duration is invalid",
            error,
        )
    })?;
    TimeRange::new(start, duration).map_err(|error| {
        crate::ProviderError::with_source(
            crate::ProviderErrorKind::InvalidContract,
            "dubbing rendered range is invalid",
            error,
        )
    })
}
