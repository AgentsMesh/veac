use crate::*;

use super::super::Validator;

pub(super) fn validate(validator: &mut Validator, project: &Project) {
    for sequence in &project.sequences {
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            let Some(visual) = &clip.visual else { continue };
            let Some((width, height)) = base_extent(visual, &sequence.settings) else {
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
                validator.push(
                    "BUDGET_VISUAL_INTERMEDIATE_PIXELS",
                    Some(clip.id.to_string()),
                    format!("/project/sequences/{}/tracks", sequence.id),
                    "visual transform or shadow exceeds the intermediate-frame memory budget",
                    Some("reduce frame size, transform scale, rotation, or shadow blur"),
                );
            }
        }
    }
}

fn base_extent(visual: &VisualProperties, settings: &SequenceSettings) -> Option<(u32, u32)> {
    match visual.frame {
        Some(frame) => Some((
            pixels(frame.width, settings.width)?,
            pixels(frame.height, settings.height)?,
        )),
        None => Some((settings.width, settings.height)),
    }
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
#[path = "visual_tests.rs"]
mod tests;
