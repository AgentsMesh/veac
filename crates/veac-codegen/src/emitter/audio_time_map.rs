use veac_artifact::{MediaRole, SourceClock};
use veac_plan::canonical::{
    AudioOutput, PitchPolicy, PlaybackDirection, RationalTime, SourceTimeSegment,
};
use veac_plan::{PlanInputId, ResolvedClip, ResolvedSourceMapping, ResolvedSourceTimeMap};

use super::{audio_processing, audio_source::invalid, CodegenErrors, EmitContext};

mod segment;

#[cfg(test)]
#[path = "../unit_tests/audio_time_map_internal_tests.rs"]
mod internal_tests;

pub(super) struct LabelRequest<'a> {
    pub raw: &'a str,
    pub mapping: &'a ResolvedSourceMapping,
    pub output: &'a AudioOutput,
    pub pitch: PitchPolicy,
    pub clock: SourceClock,
    pub source_duration: Option<RationalTime>,
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input_id: &PlanInputId,
    stream: u32,
    mapping: &ResolvedSourceMapping,
    output: &AudioOutput,
    pitch: PitchPolicy,
) -> Result<String, CodegenErrors> {
    let Some(route) = context.input_routes.stream(input_id, MediaRole::Audio) else {
        return Err(invalid(clip, "resolved audio input is missing"));
    };
    let _logical_stream = stream;
    let raw = format!("{}:{}", route.input_index(), route.global_stream());
    let clock = route.clock();
    apply_label(
        context,
        clip,
        LabelRequest {
            raw: &raw,
            mapping,
            output,
            pitch,
            clock,
            source_duration: route.source_duration(),
        },
    )
}

pub(super) fn apply_label(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    request: LabelRequest<'_>,
) -> Result<String, CodegenErrors> {
    let (raw, clock) = super::source_boundary::audio(
        context,
        clip,
        request.raw,
        request.mapping,
        request.clock,
        request.source_duration,
        request.output,
    )?;
    match &request.mapping.time_map {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            rate,
            repeat,
            direction,
        } => {
            if !clock.covers_range(*source_range_per_repeat) {
                return Err(invalid(clip, "audio source range exceeds its binding"));
            }
            let source_start = clock
                .map_boundary(source_range_per_repeat.start)
                .ok_or_else(|| invalid(clip, "audio source start is outside its binding"))?;
            let mut label = segment::trim(
                context,
                &raw,
                source_start,
                source_range_per_repeat.duration,
                request.output,
            );
            if *direction == PlaybackDirection::Reverse {
                label = context.graph.filter(&[&label], "areverse", "reversea");
            }
            label = audio_processing::speed(
                context,
                &label,
                *rate,
                request.output.sample_rate,
                request.pitch,
            );
            Ok(audio_processing::repeat(context, &label, *repeat))
        }
        ResolvedSourceTimeMap::Curve { segments } => curve(
            context,
            clip,
            &raw,
            segments,
            clock,
            request.output,
            request.pitch,
        ),
    }
}

fn curve(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segments: &[SourceTimeSegment],
    clock: SourceClock,
    output: &AudioOutput,
    pitch: PitchPolicy,
) -> Result<String, CodegenErrors> {
    if segments.is_empty() {
        return Err(invalid(clip, "resolved source-time curve is empty"));
    }
    let labels: Vec<_> = segments
        .iter()
        .map(|value| segment::filter(context, clip, raw, value, clock, output, pitch))
        .collect::<Result<_, _>>()?;
    if labels.len() == 1 {
        return Ok(labels[0].clone());
    }
    let refs: Vec<_> = labels.iter().map(String::as_str).collect();
    Ok(context
        .graph
        .filter(&refs, format!("concat=n={}:v=0:a=1", refs.len()), "rampa"))
}
