use veac_plan::canonical::{Animatable, MAX_VISUAL_SCALE};
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedSequence};

use super::super::{
    animation, process_owner::ProcessOwner, time, visual_extent, visual_pivot, EmitContext,
};

const MIN_RUNTIME_SCALE: f64 = 0.000_001;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    input: &str,
    visual: &EffectiveVisualProperties,
) -> String {
    let x = bounded(
        &visual.transform.scale,
        animation::vec_x(context.plan, owner, &visual.transform.scale, "t"),
    );
    let y = bounded(
        &visual.transform.scale,
        animation::vec_y(context.plan, owner, &visual.transform.scale, "t"),
    );
    if matches!(&visual.transform.scale, Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0)
    {
        return input.to_owned();
    }
    context.graph.filter(
        &[input],
        format!("scale=w='max(1\\,iw*({x}))':h='max(1\\,ih*({y}))':eval=frame"),
        "transformscalev",
    )
}

pub(super) fn stabilize(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    input: &str,
    pivot_x: f64,
    pivot_y: f64,
) -> String {
    if matches!(visual.transform.scale, Animatable::Constant { .. }) {
        return input.to_owned();
    }
    let extent = visual_extent::maximum_scaled_extent(context.plan, sequence, clip, visual)
        .expect("validated animated scale has a bounded extent");
    visual_pivot::stabilize(context, input, pivot_x, pivot_y, extent)
}

fn bounded(value: &Animatable<veac_plan::canonical::Vec2>, expression: String) -> String {
    if matches!(value, Animatable::Binding { .. }) {
        return format!(
            "clip(({expression})\\,{}\\,{})",
            time::number(MIN_RUNTIME_SCALE),
            time::number(MAX_VISUAL_SCALE)
        );
    }
    expression
}

#[cfg(test)]
#[path = "scale/tests.rs"]
mod tests;
