use veac_artifact::MediaRole;
use veac_plan::canonical::{AudioOutput, RationalTime};
use veac_plan::{ResolvedClip, ResolvedMulticamAngle, ResolvedMulticamSource};

use super::{source::unsupported, time, CodegenErrors, EmitContext};

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    source: &ResolvedMulticamSource,
) -> Result<String, CodegenErrors> {
    let mut labels = Vec::with_capacity(source.switches.len());
    for value in &source.switches {
        let angle = angle(source, &value.angle_id)
            .ok_or_else(|| unsupported(clip, "multicam switch angle is missing"))?;
        let route = input(context, clip, angle, MediaRole::Video)?;
        let info = context
            .plan
            .inputs
            .iter()
            .find(|value| value.id == angle.input_id)
            .and_then(|value| value.video.as_ref())
            .map(|value| value.info.clone())
            .ok_or_else(|| unsupported(clip, "multicam video facts are missing"))?;
        let logical = source_start(clip, angle.source_offset, value.range.start)?;
        let range = veac_plan::canonical::TimeRange {
            start: logical,
            duration: value.range.duration,
        };
        if !route.clock().covers_range(range) {
            return Err(unsupported(
                clip,
                "multicam video exceeds its bound source range",
            ));
        }
        let start = route
            .clock()
            .map_boundary(logical)
            .ok_or_else(|| unsupported(clip, "multicam video start is outside its binding"))?;
        let raw = format!("{}:{}", route.input_index(), route.global_stream());
        let trimmed = context.graph.filter(
            &[&raw],
            format!(
                "trim=start={}:duration={},setpts=PTS-STARTPTS",
                time::seconds(start),
                time::seconds(value.range.duration)
            ),
            "multitrimv",
        );
        let normalized = super::video_source::normalize_geometry(context, &trimmed, &info);
        labels.push(sample(context, &normalized));
    }
    concat(context, clip, &labels, "v=1:a=0", "multicamv")
}

pub(super) fn audio(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    source: &ResolvedMulticamSource,
    output: &AudioOutput,
) -> Result<String, CodegenErrors> {
    let mut labels = Vec::with_capacity(source.switches.len());
    for value in &source.switches {
        let angle = angle(source, &value.angle_id)
            .ok_or_else(|| unsupported(clip, "multicam switch angle is missing"))?;
        let stream = angle
            .audio_stream
            .ok_or_else(|| unsupported(clip, "multicam angle audio is missing"))?;
        let _logical_stream = stream;
        let route = input(context, clip, angle, MediaRole::Audio)?;
        let logical = source_start(clip, angle.source_offset, value.range.start)?;
        let range = veac_plan::canonical::TimeRange {
            start: logical,
            duration: value.range.duration,
        };
        if !route.clock().covers_range(range) {
            return Err(unsupported(
                clip,
                "multicam audio exceeds its bound source range",
            ));
        }
        let start = route
            .clock()
            .map_boundary(logical)
            .ok_or_else(|| unsupported(clip, "multicam audio start is outside its binding"))?;
        let raw = format!("{}:{}", route.input_index(), route.global_stream());
        labels.push(context.graph.filter(
            &[&raw],
            format!(
                "atrim=start={}:duration={},asetpts=PTS-STARTPTS,aresample={}",
                time::seconds(start),
                time::seconds(value.range.duration),
                output.sample_rate
            ),
            "multitrima",
        ));
    }
    concat(context, clip, &labels, "v=0:a=1", "multicama")
}

fn angle<'a>(
    source: &'a ResolvedMulticamSource,
    id: &veac_plan::canonical::MulticamAngleId,
) -> Option<&'a ResolvedMulticamAngle> {
    source.angles.iter().find(|value| value.id == *id)
}

fn input<'a>(
    context: &'a EmitContext<'_>,
    clip: &ResolvedClip,
    angle: &ResolvedMulticamAngle,
    role: MediaRole,
) -> Result<&'a super::input::RoutedStream, CodegenErrors> {
    context
        .input_routes
        .stream(&angle.input_id, role)
        .ok_or_else(|| super::source::missing_input(clip, angle.input_id.to_string()))
}

fn source_start(
    clip: &ResolvedClip,
    offset: RationalTime,
    local: RationalTime,
) -> Result<RationalTime, CodegenErrors> {
    offset
        .checked_add(local)
        .map_err(|_| unsupported(clip, "multicam source time overflowed"))
}

fn sample(context: &mut EmitContext<'_>, input: &str) -> String {
    let rate = context.canvas.frame_rate;
    context.graph.filter(
        &[input],
        format!("fps={}/{}", rate.numerator, rate.denominator),
        "multisamplev",
    )
}

fn concat(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    labels: &[String],
    streams: &str,
    prefix: &str,
) -> Result<String, CodegenErrors> {
    if labels.is_empty() {
        return Err(unsupported(clip, "multicam switch list is empty"));
    }
    if labels.len() == 1 {
        return Ok(labels[0].clone());
    }
    let inputs: Vec<_> = labels.iter().map(String::as_str).collect();
    Ok(context.graph.filter(
        &inputs,
        format!("concat=n={}:{}", labels.len(), streams),
        prefix,
    ))
}
