use veac_plan::canonical::BlendMode;
use veac_plan::{
    EffectiveVisualProperties, ResolvedApplyTarget, ResolvedClip, ResolvedClipSource,
    ResolvedSequence,
};

use super::{
    apply, blend, layer_place, matte, source, text, time, visual_pipeline, CodegenErrors,
    EmitContext,
};

pub(super) struct RenderedLayer {
    pub foreground: String,
    pub shadow: Option<String>,
}

pub(super) fn render(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<String, CodegenErrors> {
    let rendered = render_inner(context, sequence, clip, visual)?;
    let Some(shadow) = rendered.shadow else {
        return Ok(rendered.foreground);
    };
    Ok(blend::composite(
        context,
        shadow,
        &rendered.foreground,
        blend::Placement {
            start: "0".to_owned(),
            end: time::seconds(clip.record_range.duration),
            mode: BlendMode::Normal,
        },
    ))
}

pub(super) fn render_with_shadow(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<RenderedLayer, CodegenErrors> {
    render_inner(context, sequence, clip, visual)
}

fn render_inner(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Result<RenderedLayer, CodegenErrors> {
    let prepared = match &clip.source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            text::prepare(context, clip, content, visual)?
        }
        _ => {
            let source = source::video(context, clip)?;
            visual_pipeline::apply(context, clip, visual, source)?
        }
    };
    let placed = layer_place::place(context, sequence, clip, visual, prepared);
    process_branches(context, sequence, clip, visual, placed)
}

fn process_branches(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    placed: RenderedLayer,
) -> Result<RenderedLayer, CodegenErrors> {
    let shadow = placed
        .shadow
        .map(|target| process_branch(context, sequence, clip, visual, target))
        .transpose()?;
    let foreground = process_branch(context, sequence, clip, visual, placed.foreground)?;
    Ok(RenderedLayer { foreground, shadow })
}

fn process_branch(
    context: &mut EmitContext<'_>,
    sequence: &ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
    target: String,
) -> Result<String, CodegenErrors> {
    // Canonical branch order: placed pixels -> clip matte -> item-scoped applies.
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
        if items
            .iter()
            .find(|item| item.item_id == clip.id)
            .is_some_and(|item| apply::target_used(value, item.active_range))
        {
            target = apply::render(context, sequence, value, target, clip.record_range)?;
        }
    }
    Ok(target)
}
