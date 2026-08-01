use veac_plan::canonical::{transition_parameters_valid, Animatable, Mask, MaskShape, Vec2};
use veac_plan::ResolvedSequence;

use super::Check;

mod matte;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence) {
    for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
        if let Some(visual) = &clip.visual {
            validate_masks(check, &clip.id.to_string(), &visual.masks);
        }
    }
    for track in &sequence.tracks {
        for transition in &track.transitions {
            if !transition_parameters_valid(&transition.kind) {
                check.push(
                    "PLAN_TRANSITION_PARAMETERS_INVALID",
                    Some(transition.outgoing_clip_id.to_string()),
                    "transition parameters are invalid",
                );
            }
        }
    }
    matte::validate(check, sequence);
}

pub(super) fn validate_masks(check: &mut Check, owner_id: &str, masks: &[Mask]) {
    for mask in masks {
        validate_mask(check, owner_id, mask);
    }
}

fn validate_mask(check: &mut Check, owner_id: &str, mask: &Mask) {
    let shape_valid = match &mask.shape {
        MaskShape::RoundedRectangle { radius } => {
            radius.is_finite() && (0.0..=0.5).contains(radius)
        }
        MaskShape::Polygon { points } | MaskShape::Path { points } => valid_path(points),
        _ => true,
    };
    let valid = shape_valid
        && values(&mask.position).all(unit_vec)
        && values(&mask.scale).all(positive_vec)
        && values(&mask.rotation_degrees).all(|value| value.is_finite())
        && values(&mask.feather_pixels).all(|value| value.is_finite() && *value >= 0.0)
        && values(&mask.expansion_pixels).all(|value| value.is_finite());
    if !valid {
        check.push(
            "PLAN_MASK_INVALID",
            Some(owner_id.to_owned()),
            "mask shape or animation values are invalid",
        );
    }
}

fn values<T>(value: &Animatable<T>) -> Box<dyn Iterator<Item = &T> + '_> {
    match value {
        Animatable::Constant { value } => Box::new(std::iter::once(value)),
        Animatable::Keyframes { keyframes } => Box::new(keyframes.iter().map(|key| &key.value)),
    }
}

fn valid_path(points: &[Vec2]) -> bool {
    points.len() >= 3
        && points.iter().all(unit_vec)
        && points.windows(2).all(|pair| pair[0] != pair[1])
        && polygon_area(points).abs() > f64::EPSILON
}

fn polygon_area(points: &[Vec2]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(left, right)| left.x * right.y - right.x * left.y)
        .sum::<f64>()
        / 2.0
}

fn unit_vec(value: &Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=1.0).contains(&value.x)
        && (0.0..=1.0).contains(&value.y)
}

fn positive_vec(value: &Vec2) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.x > 0.0 && value.y > 0.0
}
