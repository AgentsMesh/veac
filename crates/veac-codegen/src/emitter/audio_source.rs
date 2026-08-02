use veac_plan::canonical::{Generator, PitchPolicy};
use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedSourceMapping};

use super::audio::AudioRenderSpec;
use super::error::{diagnostic, CodegenErrorKind};
use super::{
    audio, audio_fades, audio_processing, audio_sidechain, audio_transition_fades, effects, time,
    CodegenErrors, EmitContext,
};

pub(super) fn build(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    output: &AudioRenderSpec,
    fades: audio_transition_fades::TransitionFades,
    sidechain: Option<String>,
) -> Result<Option<String>, CodegenErrors> {
    let Some(properties) = &clip.audio else {
        return Ok(None);
    };
    if properties.muted {
        return Ok(None);
    }
    let source = match &clip.source {
        ResolvedClipSource::Media {
            input_id,
            audio_stream: Some(stream),
            ..
        } => {
            let mapping = super::source::source_mapping(clip)?;
            media(
                context,
                clip,
                input_id,
                stream.global_index,
                mapping,
                output,
                properties.pitch_policy,
            )?
        }
        ResolvedClipSource::Generated {
            generator: Generator::Silence,
        } => generated(context, clip, output),
        ResolvedClipSource::Sequence { sequence_id } => nested(context, clip, sequence_id, output)?,
        ResolvedClipSource::Multicam { source } => {
            super::multicam_source::audio(context, clip, source, output)?
        }
        _ => return Ok(None),
    };
    let source = audio_processing::properties(context, clip, source, output)?;
    let source = effects::audio(context, clip, source)?;
    let source = match (&properties.sidechain, sidechain) {
        (Some(value), Some(sidechain)) => {
            audio_sidechain::apply(context, clip, source, sidechain, value)?
        }
        _ => source,
    };
    let source = audio_fades::apply(context, clip, source, properties.crossfade);
    let source = audio_transition_fades::apply(context, clip, source, fades);
    let samples = time::samples(clip.record_range.start, output.sample_rate).ok_or_else(|| {
        invalid(
            clip,
            "record start is not aligned to an output audio sample",
        )
    })?;
    if samples == 0 {
        Ok(Some(source))
    } else {
        Ok(Some(context.graph.filter(
            &[&source],
            format!(
                "aresample={},adelay=delays={samples}S:all=1",
                output.sample_rate
            ),
            "delaya",
        )))
    }
}

pub(super) fn can_build(clip: &ResolvedClip) -> bool {
    if clip.audio.as_ref().is_none_or(|audio| audio.muted) {
        return false;
    }
    matches!(
        &clip.source,
        ResolvedClipSource::Media {
            audio_stream: Some(_),
            ..
        } | ResolvedClipSource::Generated {
            generator: Generator::Silence
        } | ResolvedClipSource::Sequence { .. }
            | ResolvedClipSource::Multicam { .. }
    )
}

fn media(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input_id: &veac_plan::PlanInputId,
    stream: u32,
    mapping: &ResolvedSourceMapping,
    output: &AudioRenderSpec,
    pitch: PitchPolicy,
) -> Result<String, CodegenErrors> {
    let label =
        super::audio_time_map::apply(context, clip, input_id, stream, mapping, output, pitch)?;
    Ok(context.graph.filter(
        &[&label],
        format!(
            "apad=whole_dur={},atrim=duration={},asetpts=PTS-STARTPTS",
            time::seconds(clip.record_range.duration),
            time::seconds(clip.record_range.duration)
        ),
        "bounda",
    ))
}

fn generated(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    output: &AudioRenderSpec,
) -> String {
    context.graph.source(
        format!(
            "anullsrc=r={}:cl={},atrim=duration={},asetpts=PTS-STARTPTS",
            output.sample_rate,
            audio::channel_layout(output.channels),
            time::seconds(clip.record_range.duration)
        ),
        "generateda",
    )
}

fn nested(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    sequence_id: &veac_plan::canonical::SequenceId,
    output: &AudioRenderSpec,
) -> Result<String, CodegenErrors> {
    let sequence = context
        .plan
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id)
        .cloned()
        .ok_or_else(|| invalid(clip, "nested audio sequence is missing"))?;
    let raw = audio::build_master(context, &sequence, output)?;
    let raw = context.graph.filter(
        &[&raw],
        format!(
            "asettb=1/{},asetpts=PTS-STARTPTS",
            sequence.duration.timescale
        ),
        "nestedtba",
    );
    let mapping = super::source::source_mapping(clip)?;
    let clock = super::source::nested_clock(sequence.duration, clip)?;
    let pitch = clip
        .audio
        .as_ref()
        .ok_or_else(|| invalid(clip, "nested audio source is missing audio properties"))?
        .pitch_policy;
    let label = super::audio_time_map::apply_label(
        context,
        clip,
        super::audio_time_map::LabelRequest {
            raw: &raw,
            mapping,
            output,
            pitch,
            clock,
            source_duration: Some(sequence.duration),
        },
    )?;
    Ok(context.graph.filter(
        &[&label],
        format!(
            "atrim=duration={},asetpts=PTS-STARTPTS",
            time::seconds(clip.record_range.duration)
        ),
        "nesteda",
    ))
}

pub(super) fn invalid(clip: &ResolvedClip, message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "AUDIO_PLAN_INVALID",
        Some(clip.id.to_string()),
        message,
    ))
}
