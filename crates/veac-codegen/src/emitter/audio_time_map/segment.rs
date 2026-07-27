use veac_artifact::SourceClock;
use veac_plan::canonical::{
    AudioOutput, PitchPolicy, RationalTime, SourceTimeInterpolation, SourceTimeSegment, TimeRange,
};
use veac_plan::ResolvedClip;

use super::super::{audio, audio_processing, audio_source::invalid, time};
use super::{CodegenErrors, EmitContext};

#[cfg(test)]
#[path = "../../unit_tests/audio_segment_internal_tests.rs"]
mod internal_tests;

pub(super) fn filter(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segment: &SourceTimeSegment,
    clock: SourceClock,
    output: &AudioOutput,
    pitch: PitchPolicy,
) -> Result<String, CodegenErrors> {
    if segment.record_duration.value <= 0 {
        return Err(invalid(clip, "source-time segment duration is invalid"));
    }
    if segment.interpolation == SourceTimeInterpolation::Hold {
        return Ok(silence(context, segment, output));
    }
    let delta = segment
        .source_end
        .value
        .checked_sub(segment.source_start.value)
        .ok_or_else(|| invalid(clip, "source-time segment arithmetic overflowed"))?;
    if delta == 0 {
        return Err(invalid(
            clip,
            "linear source-time segment has zero source span",
        ));
    }
    let source_span = delta
        .checked_abs()
        .ok_or_else(|| invalid(clip, "source-time segment arithmetic overflowed"))?;
    let logical_start = RationalTime {
        value: segment.source_start.value.min(segment.source_end.value),
        timescale: segment.source_start.timescale,
    };
    let source_duration = RationalTime {
        value: source_span,
        timescale: segment.source_start.timescale,
    };
    let logical_range = TimeRange {
        start: logical_start,
        duration: source_duration,
    };
    if !clock.covers_range(logical_range) {
        return Err(invalid(clip, "audio ramp exceeds its bound source range"));
    }
    let source_start = clock
        .map_boundary(logical_start)
        .ok_or_else(|| invalid(clip, "audio ramp start is outside its binding"))?;
    let mut label = trim(context, raw, source_start, source_duration, output);
    if delta < 0 {
        label = context.graph.filter(&[&label], "areverse", "rampreversea");
    }
    let rate = source_span as f64 / segment.record_duration.value as f64;
    Ok(audio_processing::speed_factor(
        context,
        &label,
        rate,
        output.sample_rate,
        pitch,
    ))
}

pub(super) fn trim(
    context: &mut EmitContext<'_>,
    raw: &str,
    start: RationalTime,
    duration: RationalTime,
    output: &AudioOutput,
) -> String {
    context.graph.filter(
        &[raw],
        format!(
            "atrim=start={}:duration={},asetpts=PTS-STARTPTS,aresample={}",
            time::seconds(start),
            time::seconds(duration),
            output.sample_rate
        ),
        "trima",
    )
}

fn silence(
    context: &mut EmitContext<'_>,
    segment: &SourceTimeSegment,
    output: &AudioOutput,
) -> String {
    context.graph.source(
        format!(
            "anullsrc=r={}:cl={},atrim=duration={},asetpts=PTS-STARTPTS",
            output.sample_rate,
            audio::channel_layout(output.channels),
            time::seconds(segment.record_duration)
        ),
        "rampholda",
    )
}
