use crate::*;

use super::super::Validator;

pub(super) fn validate(validator: &mut Validator, project: &Project) {
    for sequence in &project.sequences {
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            let Some(visual) = &clip.visual else { continue };
            let extent = placement_extent(clip, visual, &sequence.settings);
            let placement = extent
                .filter(|_| {
                    matches!(visual.transform.position, Animatable::Constant { .. })
                        && clip_requires_canvas_placement(clip, visual)
                })
                .and_then(|extent| {
                    placement_intermediate_extent(
                        (sequence.settings.width, sequence.settings.height),
                        extent,
                    )
                });
            let shadow_blur = visual
                .card
                .as_ref()
                .and_then(|card| card.shadow.as_ref())
                .map(|shadow| shadow.blur_pixels);
            let exceeds = extent
                .and_then(|extent| {
                    visual_intermediate_pixels(
                        extent,
                        placement,
                        (sequence.settings.width, sequence.settings.height),
                        shadow_blur,
                    )
                })
                .is_none_or(|pixels| pixels > MAX_VISUAL_INTERMEDIATE_PIXELS);
            if exceeds {
                validator.push(
                    "BUDGET_VISUAL_INTERMEDIATE_PIXELS",
                    Some(clip.id.to_string()),
                    format!("/project/sequences/{}/tracks", sequence.id),
                    "visual transform, placement, or shadow exceeds the intermediate-frame memory budget",
                    Some("reduce frame size, transform scale, shear, rotation, or shadow blur"),
                );
            }
        }
    }
}

fn placement_extent(
    clip: &Clip,
    visual: &VisualProperties,
    settings: &SequenceSettings,
) -> Option<(u128, u128)> {
    let (width, height) = base_extent(clip, visual, settings)?;
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

fn clip_requires_canvas_placement(clip: &Clip, visual: &VisualProperties) -> bool {
    requires_canvas_placement(CanvasPlacementFacts {
        generated: matches!(clip.source, ClipSource::Generated { .. }),
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
            visual.transform.position,
            Animatable::Constant { value } if value.x.value == 0.0 && value.y.value == 0.0
        ),
        scale_identity: matches!(
            visual.transform.scale,
            Animatable::Constant { value } if value.x == 1.0 && value.y == 1.0
        ),
        shear_zero: visual.transform.shear.x == 0.0 && visual.transform.shear.y == 0.0,
        rotation_zero: matches!(
            visual.transform.rotation_degrees,
            Animatable::Constant { value } if value == 0.0
        ),
        shadowless: visual
            .card
            .as_ref()
            .and_then(|card| card.shadow.as_ref())
            .is_none(),
    })
}

fn base_extent(
    clip: &Clip,
    visual: &VisualProperties,
    settings: &SequenceSettings,
) -> Option<(u32, u32)> {
    if let Some(frame) = visual.frame {
        let target = (
            length_pixels(frame.width, settings.width)?,
            length_pixels(frame.height, settings.height)?,
        );
        let canvas = (settings.width, settings.height);
        return match &clip.source {
            ClipSource::Generated { .. } => Some(canvas),
            ClipSource::Text { style, .. } | ClipSource::Caption { style, .. }
                if style.layout.overflow == TextOverflow::Visible
                    && style.layout.box_width_pixels.is_some()
                    && style.layout.box_height_pixels.is_some() =>
            {
                let basis = (
                    style.layout.box_width_pixels?,
                    style.layout.box_height_pixels?,
                );
                visible_overflow_frame_extent(canvas, basis, target, frame.fit)
            }
            _ => Some(target),
        };
    }
    Some((settings.width, settings.height))
}

#[cfg(test)]
#[path = "visual_tests.rs"]
mod tests;
