use veac_plan::canonical::{Animatable, Mask};
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedClipSource};

use super::{
    animation, color, effects, mask, process_owner::ProcessOwner, time, visual_frame, visual_pivot,
    visual_shear, CodegenErrors, EmitContext,
};

pub(super) struct PreparedLayer {
    pub label: String,
    pub pivot_x: f64,
    pub pivot_y: f64,
}

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    label: String,
) -> Result<PreparedLayer, CodegenErrors> {
    let geometry =
        if matches!(clip.source, ResolvedClipSource::Generated { .. }) && visual.frame.is_none() {
            visual_frame::SourceGeometry::already_sized(visual.transform.anchor)
        } else {
            visual_frame::SourceGeometry::framed(visual.transform.anchor)
        };
    apply_source(context, clip, visual, label, geometry)
}

pub(super) fn apply_source(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    label: String,
    geometry: visual_frame::SourceGeometry,
) -> Result<PreparedLayer, CodegenErrors> {
    apply_inner(context, clip, visual, label, geometry)
}

pub(super) fn apply_alpha(
    context: &mut EmitContext<'_>,
    input: &str,
    masks: &[Mask],
    opacity: &Animatable<f64>,
) -> String {
    let masked = mask::apply(context, input, masks);
    opacity_value(context, &masked, opacity)
}

fn apply_inner(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    mut label: String,
    geometry: visual_frame::SourceGeometry,
) -> Result<PreparedLayer, CodegenErrors> {
    label = color::apply(
        context,
        ProcessOwner::clip(clip),
        label,
        visual.color_pipeline.as_ref(),
    )?;
    label = super::visual_crop::apply(context, &label, &visual.transform.crop);
    label = visual_frame::apply(context, &label, visual, geometry.mode);
    label = effects::video(context, clip, label)?;
    label = flip(context, &label, visual);
    label = scale(context, &label, visual);
    label = corners(context, &label, visual);
    label = mask::apply(context, &label, &visual.masks);
    let (sheared, pivot_x, pivot_y) = visual_shear::apply(
        context,
        &label,
        visual.transform.shear,
        geometry.pivot_x,
        geometry.pivot_y,
    );
    let (rotated, pivot_x, pivot_y) = rotate(context, &sheared, visual, pivot_x, pivot_y);
    label = opacity(context, &rotated, visual);
    Ok(PreparedLayer {
        label,
        pivot_x,
        pivot_y,
    })
}

fn flip(context: &mut EmitContext<'_>, input: &str, visual: &EffectiveVisualProperties) -> String {
    let filter = match (
        visual.transform.flip_horizontal,
        visual.transform.flip_vertical,
    ) {
        (false, false) => return input.to_owned(),
        (true, false) => "hflip",
        (false, true) => "vflip",
        (true, true) => "hflip,vflip",
    };
    context.graph.filter(&[input], filter, "flipv")
}

fn scale(context: &mut EmitContext<'_>, input: &str, visual: &EffectiveVisualProperties) -> String {
    let x = animation::vec_x(&visual.transform.scale, "t");
    let y = animation::vec_y(&visual.transform.scale, "t");
    if matches!(&visual.transform.scale, Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0)
    {
        input.to_owned()
    } else {
        context.graph.filter(
            &[input],
            format!("scale=w='max(1\\,iw*({x}))':h='max(1\\,ih*({y}))':eval=frame"),
            "transformscalev",
        )
    }
}

fn rotate(
    context: &mut EmitContext<'_>,
    input: &str,
    visual: &EffectiveVisualProperties,
    pivot_x: f64,
    pivot_y: f64,
) -> (String, f64, f64) {
    if matches!(&visual.transform.rotation_degrees, Animatable::Constant { value } if *value == 0.0)
    {
        return (input.to_owned(), pivot_x, pivot_y);
    }
    let padded = visual_pivot::center(context, input, pivot_x, pivot_y, "pivotv");
    let angle = animation::number(&visual.transform.rotation_degrees, "t");
    let rotated = context.graph.filter(
        &[&padded],
        format!("rotate=angle='({angle})*PI/180':ow='ceil(hypot(iw\\,ih))':oh='ceil(hypot(iw\\,ih))':c=black@0"),
        "rotatev",
    );
    (rotated, 0.5, 0.5)
}

fn opacity(
    context: &mut EmitContext<'_>,
    input: &str,
    visual: &EffectiveVisualProperties,
) -> String {
    opacity_value(context, input, &visual.opacity)
}

fn opacity_value(context: &mut EmitContext<'_>, input: &str, opacity: &Animatable<f64>) -> String {
    if matches!(opacity, Animatable::Constant { value } if *value == 1.0) {
        return input.to_owned();
    }
    let alpha = animation::number(opacity, "T");
    context.graph.filter(
        &[input],
        format!(
            "format=rgba,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='alpha(X\\,Y)*({alpha})'"
        ),
        "opacityv",
    )
}

fn corners(
    context: &mut EmitContext<'_>,
    input: &str,
    visual: &EffectiveVisualProperties,
) -> String {
    let Some(card) = &visual.card else {
        return input.to_owned();
    };
    if card.corner_radius_pixels == 0.0 {
        return input.to_owned();
    }
    let radius = time::number(card.corner_radius_pixels);
    let alpha = format!("if(gt(abs(X-W/2)\\,W/2-{radius})*gt(abs(Y-H/2)\\,H/2-{radius})\\,if(lte(hypot(abs(X-W/2)-(W/2-{radius})\\,abs(Y-H/2)-(H/2-{radius}))\\,{radius})\\,alpha(X\\,Y)\\,0)\\,alpha(X\\,Y))");
    context.graph.filter(
        &[input],
        format!("format=rgba,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='{alpha}'"),
        "cornerv",
    )
}
