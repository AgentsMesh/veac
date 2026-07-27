use veac_plan::canonical::*;
use veac_plan::{EffectiveVisualProperties, ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for sequence in &plan.sequences {
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            let Some(visual) = &clip.visual else { continue };
            let Some((width, height)) = base_extent(plan, sequence, clip, visual) else {
                continue;
            };
            let Some((scale_x, scale_y)) = max_scale(&visual.transform.scale) else {
                continue;
            };
            let rotated = !matches!(
                visual.transform.rotation_degrees,
                Animatable::Constant { value } if value == 0.0
            );
            let transformed = visual_transform_pixels(width, height, scale_x, scale_y, rotated);
            let shadow = visual
                .card
                .as_ref()
                .and_then(|card| card.shadow.as_ref())
                .and_then(|shadow| {
                    shadow_intermediate_pixels(
                        sequence.settings.width,
                        sequence.settings.height,
                        shadow.blur_pixels,
                    )
                });
            if transformed
                .into_iter()
                .chain(shadow)
                .any(|pixels| pixels > MAX_VISUAL_INTERMEDIATE_PIXELS)
            {
                check.push(
                    "PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS",
                    Some(clip.id.to_string()),
                    "visual transform or shadow exceeds the intermediate-frame memory budget",
                );
            }
        }
    }
}

fn base_extent(
    plan: &ResolvedRenderPlan,
    sequence: &veac_plan::ResolvedSequence,
    clip: &ResolvedClip,
    visual: &EffectiveVisualProperties,
) -> Option<(u32, u32)> {
    if let Some(frame) = visual.frame {
        return Some((
            pixels(frame.width, sequence.settings.width)?,
            pixels(frame.height, sequence.settings.height)?,
        ));
    }
    let canvas = (sequence.settings.width, sequence.settings.height);
    let source = source_extent(plan, clip).unwrap_or(canvas);
    Some((canvas.0.max(source.0), canvas.1.max(source.1)))
}

fn source_extent(plan: &ResolvedRenderPlan, clip: &ResolvedClip) -> Option<(u32, u32)> {
    match &clip.source {
        ResolvedClipSource::Media { input_id, .. }
        | ResolvedClipSource::FreezeFrame { input_id, .. } => input_extent(plan, input_id),
        ResolvedClipSource::Sequence { sequence_id } => plan
            .sequences
            .iter()
            .find(|value| value.id == *sequence_id)
            .map(|value| (value.settings.width, value.settings.height)),
        ResolvedClipSource::Multicam { source } => source
            .angles
            .iter()
            .filter_map(|angle| input_extent(plan, &angle.input_id))
            .reduce(|left, right| (left.0.max(right.0), left.1.max(right.1))),
        ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
            text_extent(content)
        }
        ResolvedClipSource::Generated { .. } => None,
    }
}

fn input_extent(plan: &ResolvedRenderPlan, id: &veac_plan::PlanInputId) -> Option<(u32, u32)> {
    let info = plan
        .inputs
        .iter()
        .find(|value| value.id == *id)?
        .video
        .as_ref()
        .map(|video| &video.info)?;
    normalized_video_dimensions(info)
}

fn text_extent(content: &veac_plan::ResolvedText) -> Option<(u32, u32)> {
    Some((
        content.style.layout.box_width_pixels?.round() as u32,
        content.style.layout.box_height_pixels?.round() as u32,
    ))
}

fn pixels(value: Length, extent: u32) -> Option<u32> {
    let value = match value.unit {
        LengthUnit::Pixels => value.value,
        LengthUnit::Normalized => value.value * f64::from(extent),
        LengthUnit::Percent => value.value * f64::from(extent) / 100.0,
    };
    (value.is_finite() && value > 0.0 && value <= f64::from(u32::MAX))
        .then(|| value.round().max(1.0) as u32)
}

fn max_scale(value: &Animatable<Vec2>) -> Option<(f64, f64)> {
    let mut values: Box<dyn Iterator<Item = Vec2> + '_> = match value {
        Animatable::Constant { value } => Box::new(std::iter::once(*value)),
        Animatable::Keyframes { keyframes } => Box::new(keyframes.iter().map(|key| key.value)),
    };
    values.try_fold((0.0_f64, 0.0_f64), |(x, y), value| {
        visual_scale_valid(value).then_some((x.max(value.x), y.max(value.y)))
    })
}

#[cfg(test)]
#[path = "visual/tests.rs"]
mod tests;
