use veac_plan::canonical::EffectParameter;
use veac_plan::{ResolvedClip, ResolvedEffect};

use super::super::{time, CodegenErrors, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    effect: &ResolvedEffect,
    input: &str,
) -> Result<String, CodegenErrors> {
    let target = super::number(effect, EffectParameter::TargetLufs, -16.0, clip)?;
    if effect.active_range.start.value == 0
        && effect.active_range.duration == clip.record_range.duration
    {
        return Ok(loudnorm(context, input, &target));
    }
    partial(context, clip, effect, input, &target)
}

fn partial(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    effect: &ResolvedEffect,
    input: &str,
    target: &str,
) -> Result<String, CodegenErrors> {
    let end = effect
        .active_range
        .end()
        .map_err(|_| super::unsupported_clip(clip, effect, "invalid active range"))?;
    let has_before = effect.active_range.start.value > 0;
    let has_after = end < clip.record_range.duration;
    let count = 1 + usize::from(has_before) + usize::from(has_after);
    let mut inputs = context
        .graph
        .filter_many(&[input], format!("asplit={count}"), "norm", count)
        .into_iter();
    let mut segments = Vec::with_capacity(count);
    if has_before {
        let before = inputs.next().expect("split count includes prefix");
        segments.push(trim(
            context,
            &before,
            "0",
            &time::seconds(effect.active_range.start),
        ));
    }
    let middle = inputs.next().expect("split count includes effect range");
    let middle = trim(
        context,
        &middle,
        &time::seconds(effect.active_range.start),
        &time::seconds(end),
    );
    segments.push(loudnorm(context, &middle, target));
    if has_after {
        let after = inputs.next().expect("split count includes suffix");
        segments.push(trim(
            context,
            &after,
            &time::seconds(end),
            &time::seconds(clip.record_range.duration),
        ));
    }
    let labels: Vec<_> = segments.iter().map(String::as_str).collect();
    Ok(context
        .graph
        .filter(&labels, format!("concat=n={count}:v=0:a=1"), "effecta"))
}

fn trim(context: &mut EmitContext<'_>, input: &str, start: &str, end: &str) -> String {
    context.graph.filter(
        &[input],
        format!("atrim=start={start}:end={end},asetpts=PTS-STARTPTS"),
        "norm",
    )
}

fn loudnorm(context: &mut EmitContext<'_>, input: &str, target: &str) -> String {
    context.graph.filter(
        &[input],
        format!("loudnorm=I={target}:LRA=11:TP=-1.5"),
        "effecta",
    )
}
