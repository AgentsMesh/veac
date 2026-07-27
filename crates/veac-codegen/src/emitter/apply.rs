use veac_plan::canonical::{Animatable, BlendMode, TimeRange};
use veac_plan::{ResolvedApply, ResolvedApplyOperation, ResolvedSequence};

use super::{
    apply_slice, blend, color, effects, matte, process_owner::ProcessOwner, time, visual_pipeline,
    CodegenErrors, EmitContext,
};

pub(super) fn render(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    apply: &ResolvedApply,
    input: String,
    window: TimeRange,
) -> Result<String, CodegenErrors> {
    let Some(active) = intersect(apply.record_range, window) else {
        return Ok(input);
    };
    apply_slice::map(
        context,
        input,
        window,
        active,
        active.start,
        "applyrangev",
        |context, segment| process(context, sequence, apply, segment, active),
    )
}

pub(super) fn join(
    context: &mut EmitContext<'_>,
    underlay: String,
    band: &str,
    sequence: &ResolvedSequence,
) -> String {
    blend::composite(
        context,
        underlay,
        band,
        blend::Placement {
            x: "0",
            y: "0",
            start: "0".to_owned(),
            end: time::seconds(sequence.duration),
            mode: BlendMode::Normal,
            shadow: None,
        },
    )
}

pub(super) fn replace(
    context: &mut EmitContext<'_>,
    current: String,
    replacement: String,
    range: TimeRange,
) -> String {
    let current = context.graph.filter(
        &[&current],
        "settb=AVTB,setpts=PTS-STARTPTS",
        "applycurrentclockv",
    );
    let replacement = context.graph.filter(
        &[&replacement],
        "settb=AVTB,setpts=PTS-STARTPTS",
        "applyreplacementclockv",
    );
    context.graph.filter(
        &[&current, &replacement],
        format!(
            "blend=all_expr='if(gte(T\\,{})*lt(T\\,{})\\,B\\,A)'",
            time::seconds(range.start),
            time::end(range)
        ),
        "applyreplacev",
    )
}

fn process(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    apply: &ResolvedApply,
    input: String,
    window: TimeRange,
) -> Result<String, CodegenErrors> {
    let composite = needs_mix(apply);
    let (original, mut processed) = if composite {
        let (original, processed) = context.graph.split(&input, "applymixsplitv");
        (Some(original), processed)
    } else {
        (None, input)
    };
    for stage in &apply.stages {
        let Some(active) = intersect(stage.active_range, window) else {
            continue;
        };
        processed = apply_slice::map(
            context,
            processed,
            window,
            active,
            apply.record_range.start,
            "applystagev",
            |context, segment| match &stage.operation {
                ResolvedApplyOperation::Color { pipeline } => {
                    color::apply(context, ProcessOwner::apply(apply), segment, Some(pipeline))
                }
                ResolvedApplyOperation::Effect { .. } => {
                    let local = TimeRange {
                        start: apply_slice::offset(apply.record_range.start, active.start),
                        duration: active.duration,
                    };
                    effects::apply_stage(context, apply, stage, local, segment)
                }
            },
        )?;
    }
    if let Some(value) = &apply.matte {
        processed = matte::apply_window(
            context,
            sequence,
            &apply.id.to_string(),
            processed,
            value,
            window,
        )?;
    }
    let Some(original) = original else {
        return Ok(processed);
    };
    let processed =
        visual_pipeline::apply_alpha(context, &processed, &apply.mix.masks, &apply.mix.opacity);
    Ok(blend::composite(
        context,
        original,
        &processed,
        blend::Placement {
            x: "0",
            y: "0",
            start: "0".to_owned(),
            end: time::seconds(window.duration),
            mode: apply.mix.blend_mode,
            shadow: None,
        },
    ))
}

fn needs_mix(apply: &ResolvedApply) -> bool {
    !apply.mix.masks.is_empty()
        || apply.matte.is_some()
        || apply.mix.blend_mode != BlendMode::Normal
        || !matches!(&apply.mix.opacity, Animatable::Constant { value } if *value == 1.0)
}

fn intersect(left: TimeRange, right: TimeRange) -> Option<TimeRange> {
    let start = if left.start > right.start {
        left.start
    } else {
        right.start
    };
    let left_end = left.end().ok()?;
    let right_end = right.end().ok()?;
    let end = if left_end < right_end {
        left_end
    } else {
        right_end
    };
    (start < end).then(|| TimeRange {
        start,
        duration: apply_slice::offset(start, end),
    })
}
