use veac_artifact::SourceClock;
use veac_plan::canonical::{SourceTimeInterpolation, SourceTimeSegment, VideoStreamInfo};
use veac_plan::ResolvedClip;

use super::super::{source::unsupported, time, video_source, CodegenErrors, EmitContext};

#[cfg(test)]
#[path = "../../unit_tests/video_curve_internal_tests.rs"]
mod internal_tests;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segments: &[SourceTimeSegment],
    clock: SourceClock,
    image: bool,
    info: Option<&VideoStreamInfo>,
) -> Result<String, CodegenErrors> {
    if segments.is_empty() {
        return Err(unsupported(clip, "resolved source-time curve is empty"));
    }
    let labels: Vec<_> = segments
        .iter()
        .map(|segment| segment_filter(context, clip, raw, segment, clock, image, info))
        .collect::<Result<_, _>>()?;
    if labels.len() == 1 {
        return Ok(labels[0].clone());
    }
    let refs: Vec<_> = labels.iter().map(String::as_str).collect();
    Ok(context
        .graph
        .filter(&refs, format!("concat=n={}:v=1:a=0", refs.len()), "rampv"))
}

fn segment_filter(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segment: &SourceTimeSegment,
    clock: SourceClock,
    image: bool,
    info: Option<&VideoStreamInfo>,
) -> Result<String, CodegenErrors> {
    if segment.record_duration.value <= 0 {
        return Err(unsupported(clip, "source-time segment duration is invalid"));
    }
    let mut label = match segment.interpolation {
        SourceTimeInterpolation::Hold => hold(context, clip, raw, segment, clock, image)?,
        SourceTimeInterpolation::Linear => linear(context, clip, raw, segment, clock, image)?,
    };
    if let Some(info) = info {
        label = video_source::normalize_geometry(context, &label, info);
    }
    Ok(label)
}

fn linear(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segment: &SourceTimeSegment,
    clock: SourceClock,
    image: bool,
) -> Result<String, CodegenErrors> {
    let delta = segment
        .source_end
        .value
        .checked_sub(segment.source_start.value)
        .ok_or_else(|| unsupported(clip, "source-time segment arithmetic overflowed"))?;
    if delta == 0 {
        return Err(unsupported(
            clip,
            "linear source-time segment has zero source span",
        ));
    }
    let source_span = delta
        .checked_abs()
        .ok_or_else(|| unsupported(clip, "source-time segment arithmetic overflowed"))?;
    let logical_start = veac_plan::canonical::RationalTime {
        value: segment.source_start.value.min(segment.source_end.value),
        timescale: segment.source_start.timescale,
    };
    let source_duration = veac_plan::canonical::RationalTime {
        value: source_span,
        timescale: segment.source_start.timescale,
    };
    let logical_range = veac_plan::canonical::TimeRange {
        start: logical_start,
        duration: source_duration,
    };
    if !clock.covers_range(logical_range) {
        return Err(unsupported(
            clip,
            "video ramp exceeds its bound source range",
        ));
    }
    let source_start = clock
        .map_boundary(logical_start)
        .ok_or_else(|| unsupported(clip, "video ramp start is outside its binding"))?;
    let mut label = context.graph.filter(
        &[raw],
        format!(
            "{}trim=start={}:duration={},setpts=PTS-STARTPTS",
            super::image_prefix(image),
            time::seconds(source_start),
            time::seconds(source_duration)
        ),
        "ramptrimv",
    );
    if delta < 0 && !image {
        label = super::frame_limit(context, &label, delta.abs(), clock.timescale());
        let reverse = super::reverse_filter(context);
        label = context.graph.filter(&[&label], reverse, "rampreversev");
    }
    Ok(context.graph.filter(
        &[&label],
        format!(
            "setpts=PTS*{}/{}",
            segment.record_duration.value, source_span
        ),
        "rampspeedv",
    ))
}

fn hold(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: &str,
    segment: &SourceTimeSegment,
    clock: SourceClock,
    image: bool,
) -> Result<String, CodegenErrors> {
    let source = clock
        .map_point(segment.source_start)
        .ok_or_else(|| unsupported(clip, "video hold point is outside its binding"))?;
    Ok(context.graph.filter(
        &[raw],
        super::super::frame_hold::filter(
            source,
            segment.record_duration,
            image,
            context.canvas.frame_rate,
        ),
        "rampholdv",
    ))
}
