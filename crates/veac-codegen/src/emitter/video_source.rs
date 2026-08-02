use veac_artifact::{MediaRole, SourceClock};
use veac_plan::canonical::{FrameSynthesisPolicy, RationalTime, VideoStreamInfo};
use veac_plan::{PlanInputId, ResolvedClip, ResolvedSourceMapping};

use super::{source::missing_input, time, CodegenErrors, EmitContext};

pub(super) fn media(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input_id: &PlanInputId,
    mapping: &ResolvedSourceMapping,
    image: bool,
    info: &VideoStreamInfo,
) -> Result<String, CodegenErrors> {
    let route = input_route(context, clip, input_id)?;
    let raw = format!("{}:{}", route.input_index(), route.global_stream());
    let clock = route.clock();
    let label = super::video_time_map::apply(
        context,
        clip,
        super::video_time_map::Request {
            raw: &raw,
            mapping,
            clock,
            source_duration: route.source_duration(),
            time_invariant: route.time_invariant(),
            image,
            info: Some(info),
        },
    )?;
    let label = sample_frames(context, &label, mapping.frame_synthesis);
    Ok(context.graph.filter(
        &[&label],
        format!(
            "trim=duration={},setpts=PTS-STARTPTS",
            time::seconds(clip.record_range.duration)
        ),
        "boundv",
    ))
}

pub(super) fn nested(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    raw: String,
    mapping: &ResolvedSourceMapping,
    clock: SourceClock,
) -> Result<String, CodegenErrors> {
    let label = super::video_time_map::apply(
        context,
        clip,
        super::video_time_map::Request {
            raw: &raw,
            mapping,
            clock,
            source_duration: clock.logical_range().map(|range| range.duration),
            time_invariant: false,
            image: false,
            info: None,
        },
    )?;
    let label = sample_frames(context, &label, mapping.frame_synthesis);
    Ok(context.graph.filter(
        &[&label],
        format!(
            "trim=duration={},setpts=PTS-STARTPTS",
            time::seconds(clip.record_range.duration)
        ),
        "boundv",
    ))
}

pub(super) fn normalize_geometry(
    context: &mut EmitContext<'_>,
    input: &str,
    info: &VideoStreamInfo,
) -> String {
    let mut filters = Vec::new();
    let sar = info.sample_aspect_ratio;
    if sar.numerator != i64::from(sar.denominator) {
        filters.push(format!(
            "scale=w='iw*{}/{}':h=ih,setsar=1",
            sar.numerator, sar.denominator
        ));
    }
    let rotation = info.rotation_degrees.rem_euclid(360);
    match rotation {
        0 => {}
        90 => filters.push("transpose=clock".to_owned()),
        180 => filters.push("hflip,vflip".to_owned()),
        270 => filters.push("transpose=cclock".to_owned()),
        degrees => filters.push(format!(
            "format=rgba,rotate=angle='{degrees}*PI/180':ow='rotw({degrees}*PI/180)':oh='roth({degrees}*PI/180)':c=black@0"
        )),
    }
    if filters.is_empty() {
        input.to_owned()
    } else {
        context
            .graph
            .filter(&[input], filters.join(","), "geometryv")
    }
}

pub(super) fn freeze(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    input_id: &PlanInputId,
    source_time: RationalTime,
    image: bool,
    info: &VideoStreamInfo,
) -> Result<String, CodegenErrors> {
    let route = input_route(context, clip, input_id)?;
    let source_time = route
        .clock()
        .map_point(source_time)
        .ok_or_else(|| super::source::unsupported(clip, "freeze time is outside bound media"))?;
    let label = context.graph.filter(
        &[&format!(
            "{}:{}",
            route.input_index(),
            route.global_stream()
        )],
        super::frame_hold::filter(
            source_time,
            clip.record_range.duration,
            image,
            context.canvas.frame_rate,
        ),
        "freezev",
    );
    Ok(normalize_geometry(context, &label, info))
}

fn input_route<'a>(
    context: &'a EmitContext<'_>,
    clip: &ResolvedClip,
    input_id: &PlanInputId,
) -> Result<&'a super::input::RoutedStream, CodegenErrors> {
    let Some(route) = context.input_routes.stream(input_id, MediaRole::Video) else {
        return Err(missing_input(clip, input_id.to_string()));
    };
    Ok(route)
}

fn sample_frames(
    context: &mut EmitContext<'_>,
    label: &str,
    sampling: FrameSynthesisPolicy,
) -> String {
    let rate = &context.canvas.frame_rate;
    let fps = format!("{}/{}", rate.numerator, rate.denominator);
    let filter = match sampling {
        FrameSynthesisPolicy::Nearest => format!("fps={fps}"),
        FrameSynthesisPolicy::Blend => format!("minterpolate=fps={fps}:mi_mode=blend"),
        FrameSynthesisPolicy::MotionCompensated => {
            format!("minterpolate=fps={fps}:mi_mode=mci:mc_mode=aobmc:me_mode=bidir:vsbmc=1")
        }
    };
    context.graph.filter(&[label], filter, "samplev")
}
