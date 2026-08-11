use veac_artifact::SourceClock;
use veac_plan::canonical::{PlaybackDirection, RationalTime, VideoStreamInfo};
use veac_plan::{ResolvedClip, ResolvedSourceMapping, ResolvedSourceTimeMap};

use super::{source::unsupported, time, video_source, CodegenErrors, EmitContext};

mod curve;
mod static_source;

pub(super) struct Request<'a> {
    pub raw: &'a str,
    pub mapping: &'a ResolvedSourceMapping,
    pub clock: SourceClock,
    pub source_duration: Option<RationalTime>,
    pub time_invariant: bool,
    pub image: bool,
    pub info: Option<&'a VideoStreamInfo>,
    pub reverse_facts: Option<super::reverse_ledger::VideoFacts>,
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    request: Request<'_>,
) -> Result<String, CodegenErrors> {
    if request.time_invariant || request.image {
        return static_source::apply(context, clip, request.raw, request.info);
    }
    let (raw, clock) = super::source_boundary::video(
        context,
        clip,
        request.raw,
        request.mapping,
        request.clock,
        request.source_duration,
    )?;
    match &request.mapping.time_map {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            rate,
            repeat,
            direction,
        } => {
            let prefix = image_prefix(request.image);
            let source_start = clock
                .map_boundary(source_range_per_repeat.start)
                .ok_or_else(|| unsupported(clip, "video source start is outside its binding"))?;
            if !clock.covers_range(*source_range_per_repeat) {
                return Err(unsupported(clip, "video source range exceeds its binding"));
            }
            let mut label = context.graph.filter(
                &[&raw],
                format!(
                    "{prefix}trim=start={}:duration={},setpts=PTS-STARTPTS",
                    time::seconds(source_start),
                    time::seconds(source_range_per_repeat.duration)
                ),
                "trimv",
            );
            if *direction == PlaybackDirection::Reverse && !request.image {
                label = context.reverse_video(
                    &label,
                    "reversev",
                    clip,
                    source_range_per_repeat.duration,
                    request.reverse_facts,
                )?;
            }
            if rate.numerator != i64::from(rate.denominator) {
                label = context.graph.filter(
                    &[&label],
                    format!("setpts=PTS/{}", time::rate(*rate)),
                    "speedv",
                );
            }
            if let Some(info) = request.info {
                label = video_source::normalize_geometry(context, &label, info);
            }
            Ok(repeat_video(context, &label, *repeat))
        }
        ResolvedSourceTimeMap::Curve { segments } => curve::apply(
            context,
            clip,
            curve::Request {
                raw: &raw,
                segments,
                clock,
                image: request.image,
                info: request.info,
                reverse_facts: request.reverse_facts,
            },
        ),
    }
}

fn repeat_video(context: &mut EmitContext<'_>, label: &str, count: u32) -> String {
    if count <= 1 {
        return label.to_owned();
    }
    let labels = context.graph.filter_many(
        &[label],
        format!("split={count}"),
        "loopcopyv",
        count as usize,
    );
    let refs: Vec<_> = labels.iter().map(String::as_str).collect();
    context
        .graph
        .filter(&refs, format!("concat=n={count}:v=1:a=0"), "loopv")
}

pub(super) fn image_prefix(image: bool) -> &'static str {
    if image {
        "loop=loop=-1:size=1:start=0,"
    } else {
        ""
    }
}
