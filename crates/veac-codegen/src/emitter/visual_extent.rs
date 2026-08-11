use veac_plan::canonical::*;
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

#[path = "visual_extent/source.rs"]
mod source;

use source::source_extent;

pub(super) fn placement_extent(
    plan: &ResolvedRenderPlan,
    sequence: &veac_plan::ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Option<(u128, u128)> {
    let (width, height) = base_extent(plan, sequence, clip, visual)?;
    let (scale_x, scale_y) = max_visual_scale(&visual.transform.scale)?;
    let rotated = !matches!(
        visual.transform.rotation_degrees,
        Animatable::Constant { value } if value == 0.0
    );
    let shear = visual.transform.shear;
    let pivot_off_center = visual.transform.anchor != Vec2 { x: 0.5, y: 0.5 };
    let extent = visual_transform_extent(
        width,
        height,
        Vec2 {
            x: scale_x,
            y: scale_y,
        },
        pivot_off_center,
        shear,
        rotated,
    )?;
    expand_shadow(extent, visual)
}

pub(super) fn maximum_scaled_extent(
    plan: &ResolvedRenderPlan,
    sequence: &veac_plan::ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Option<(u128, u128)> {
    let (width, height) = base_extent(plan, sequence, clip, visual)?;
    let (x, y) = max_visual_scale(&visual.transform.scale)?;
    visual_transform_extent(
        width,
        height,
        Vec2 { x, y },
        false,
        Vec2 { x: 0.0, y: 0.0 },
        false,
    )
}

pub(super) fn requires_canvas_placement(
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> bool {
    veac_plan::canonical::requires_canvas_placement(CanvasPlacementFacts {
        generated: matches!(clip.source, ResolvedClipSource::Generated { .. }),
        effect_free: clip.effects.is_empty(),
        frameless: visual.frame.is_none(),
        uncropped: visual.transform.crop.is_none(),
        placement_default: matches!(
            visual.placement,
            Placement::Anchor {
                anchor: Anchor::Center,
                inset,
            } if inset.x == 0.0 && inset.y == 0.0
        ),
        transform_anchor_centered: visual.transform.anchor == Vec2 { x: 0.5, y: 0.5 },
        position_zero: matches!(
            &visual.transform.position,
            Animatable::Constant { value }
                if value.x.value == 0.0 && value.y.value == 0.0
        ),
        scale_identity: matches!(
            &visual.transform.scale,
            Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0
        ),
        shear_zero: visual.transform.shear.x == 0.0 && visual.transform.shear.y == 0.0,
        rotation_zero: matches!(
            &visual.transform.rotation_degrees,
            Animatable::Constant { value } if *value == 0.0
        ),
        shadowless: visual
            .card
            .as_ref()
            .and_then(|card| card.shadow.as_ref())
            .is_none(),
    })
}

fn base_extent(
    plan: &ResolvedRenderPlan,
    sequence: &veac_plan::ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Option<(u32, u32)> {
    let canvas = (sequence.settings.width, sequence.settings.height);
    let Some(frame) = visual.frame else {
        let source = source_extent(plan, clip).unwrap_or(canvas);
        return Some((canvas.0.max(source.0), canvas.1.max(source.1)));
    };
    let target = (
        length_pixels(frame.width, canvas.0)?,
        length_pixels(frame.height, canvas.1)?,
    );
    match &clip.source {
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            let layout = content.styled()?.layout;
            if layout.overflow != TextOverflow::Visible
                || layout.box_width_pixels.is_none()
                || layout.box_height_pixels.is_none()
            {
                return Some(target);
            }
            let basis = (layout.box_width_pixels?, layout.box_height_pixels?);
            visible_overflow_frame_extent(canvas, basis, target, frame.fit)
        }
        _ => Some(target),
    }
}

fn expand_shadow(extent: (u128, u128), visual: &EffectiveVisualProperties) -> Option<(u128, u128)> {
    let Some(shadow) = visual.card.as_ref().and_then(|card| card.shadow.as_ref()) else {
        return Some(extent);
    };
    let (width, height) =
        shadow_extent_expansion(shadow.blur_pixels, shadow.offset.x, shadow.offset.y)?;
    Some((
        extent.0.checked_add(u128::from(width))?,
        extent.1.checked_add(u128::from(height))?,
    ))
}

#[cfg(test)]
#[path = "visual_extent/tests.rs"]
mod tests;
