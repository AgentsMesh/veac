use veac_plan::canonical::{Anchor, Animatable, BlendMode, Placement as VisualPlacement};
use veac_plan::{
    EffectiveVisualProperties, ResolvedApplyTarget, ResolvedClip, ResolvedClipSource,
    ResolvedSequence,
};

use super::{
    apply, blend, generated, geometry, matte, source, text, time, visual_pipeline, CodegenErrors,
    EmitContext,
};

pub(super) fn render(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<String, CodegenErrors> {
    let prepared = match &clip.source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            text::prepare(context, clip, content, visual)?
        }
        _ => {
            let source = source::video(context, clip)?;
            visual_pipeline::apply(context, clip, visual, source)?
        }
    };
    let target = place(context, clip, visual, prepared);
    let target = match &visual.track_matte {
        Some(track_matte) => matte::apply(context, sequence, clip, target, track_matte),
        None => Ok(target),
    }?;
    item_applies(context, sequence, clip, target)
}

fn item_applies(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    mut target: String,
) -> Result<String, CodegenErrors> {
    for value in &sequence.applies {
        let ResolvedApplyTarget::ItemSet { items } = &value.target else {
            continue;
        };
        if items.iter().any(|item| item.item_id == clip.id) {
            target = apply::render(context, sequence, value, target, clip.record_range)?;
        }
    }
    Ok(target)
}

fn place(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    prepared: visual_pipeline::PreparedLayer,
) -> String {
    let layer = context
        .graph
        .filter(&[&prepared.label], "format=gbrap16le", "layerprecisionv");
    if is_canvas_identity(clip, visual, &prepared) {
        return layer;
    }
    let canvas = generated::high_precision_canvas(context, clip, "layercanvasv");
    let (x, y) = geometry::overlay_position(visual, "t", prepared.pivot_x, prepared.pivot_y);
    blend::composite(
        context,
        canvas,
        &layer,
        blend::Placement {
            x: &x,
            y: &y,
            start: "0".to_owned(),
            end: time::seconds(clip.record_range.duration),
            mode: BlendMode::Normal,
            shadow: visual.card.as_ref().and_then(|card| card.shadow.as_ref()),
        },
    )
}

fn is_canvas_identity(
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    prepared: &visual_pipeline::PreparedLayer,
) -> bool {
    matches!(clip.source, ResolvedClipSource::Generated { .. })
        && clip.effects.is_empty()
        && visual.frame.is_none()
        && visual.transform.crop.is_none()
        && prepared.pivot_x == 0.5
        && prepared.pivot_y == 0.5
        && visual.transform.anchor.x == 0.5
        && visual.transform.anchor.y == 0.5
        && matches!(
            visual.placement,
            VisualPlacement::Anchor {
                anchor: Anchor::Center,
                inset
            } if inset.x == 0.0 && inset.y == 0.0
        )
        && matches!(
            &visual.transform.position,
            Animatable::Constant { value }
                if value.x.value == 0.0 && value.y.value == 0.0
        )
        && matches!(
            &visual.transform.scale,
            Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0
        )
        && matches!(
            &visual.transform.rotation_degrees,
            Animatable::Constant { value } if *value == 0.0
        )
        && visual
            .card
            .as_ref()
            .and_then(|card| card.shadow.as_ref())
            .is_none()
}
