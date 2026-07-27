use super::super::{
    effects, effects::EffectSpec, process_owner::ProcessOwner, time, CodegenErrors, EmitContext,
};
use veac_plan::canonical::ParameterValue;
use veac_plan::ResolvedClip;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    effect: EffectSpec<'_>,
    input: &str,
) -> Result<String, CodegenErrors> {
    if !matches!(
        effect.parameters.get("enabled"),
        Some(ParameterValue::Boolean { value: true })
    ) {
        return Ok(input.to_owned());
    }
    let Some(clip) = owner.as_clip() else {
        return Err(effects::unsupported(
            owner,
            effect,
            "stabilization requires a source clip",
        ));
    };
    if effect.active_range.start.value == 0
        && effect.active_range.duration == clip.record_range.duration
    {
        return Ok(context.graph.filter(&[input], "deshake", "effectv"));
    }
    partial(context, clip, effect, input)
}

fn partial(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    effect: EffectSpec<'_>,
    input: &str,
) -> Result<String, CodegenErrors> {
    let end = effect.active_range.end().map_err(|_| {
        effects::unsupported(ProcessOwner::clip(clip), effect, "invalid active range")
    })?;
    let has_before = effect.active_range.start.value > 0;
    let has_after = end < clip.record_range.duration;
    let count = 1 + usize::from(has_before) + usize::from(has_after);
    let mut inputs = context
        .graph
        .filter_many(&[input], format!("split={count}"), "stabilize", count)
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
    segments.push(context.graph.filter(&[&middle], "deshake", "effectv"));
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
        .filter(&labels, format!("concat=n={count}:v=1:a=0"), "effectv"))
}

fn trim(context: &mut EmitContext<'_>, input: &str, start: &str, end: &str) -> String {
    context.graph.filter(
        &[input],
        format!("trim=start={start}:end={end},setpts=PTS-STARTPTS"),
        "stabilize",
    )
}
