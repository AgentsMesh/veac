use std::collections::{BTreeMap, BTreeSet};

use veac_plan::canonical::{transition_parameters_valid, Animatable, Mask, MaskShape, Vec2};
use veac_plan::{ResolvedClip, ResolvedMatte, ResolvedSequence};

use super::Check;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence) {
    let clips: BTreeMap<_, _> = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .map(|clip| (&clip.id, clip))
        .collect();
    let mut graph = BTreeMap::new();
    for clip in clips.values() {
        if let Some(visual) = &clip.visual {
            validate_masks(check, &clip.id.to_string(), &visual.masks);
            if let Some(matte) = &visual.track_matte {
                validate_matte(check, clip, matte, &clips);
                graph.insert(&clip.id, &matte.source_clip_id);
            }
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
    if cyclic(&graph) {
        check.push(
            "PLAN_MATTE_CYCLE",
            Some(sequence.id.to_string()),
            "track matte references are cyclic",
        );
    }
    if depth_exceeded(&graph) {
        check.push(
            "PLAN_MATTE_DEPTH_EXCEEDED",
            Some(sequence.id.to_string()),
            "track matte dependency depth exceeds the executable limit",
        );
    }
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

fn validate_matte(
    check: &mut Check,
    target: &ResolvedClip,
    matte: &ResolvedMatte,
    clips: &BTreeMap<&veac_plan::canonical::ItemId, &ResolvedClip>,
) {
    let valid = clips.get(&matte.source_clip_id).is_some_and(|source| {
        source.id != target.id
            && source.visual.is_some()
            && source.record_range.start <= target.record_range.start
            && source.record_range.end().is_ok_and(|source_end| {
                target
                    .record_range
                    .end()
                    .is_ok_and(|target_end| source_end >= target_end)
            })
    });
    if !valid {
        check.push(
            "PLAN_MATTE_REFERENCE_INVALID",
            Some(target.id.to_string()),
            "track matte source is missing, incompatible, or does not cover the target",
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

fn cyclic<'a>(
    graph: &BTreeMap<&'a veac_plan::canonical::ItemId, &'a veac_plan::canonical::ItemId>,
) -> bool {
    graph.keys().any(|start| {
        let mut seen = BTreeSet::new();
        let mut current = *start;
        while seen.insert(current) {
            let Some(next) = graph.get(current) else {
                return false;
            };
            current = next;
        }
        true
    })
}

fn depth_exceeded<'a>(
    graph: &BTreeMap<&'a veac_plan::canonical::ItemId, &'a veac_plan::canonical::ItemId>,
) -> bool {
    graph.keys().any(|start| {
        let mut seen = BTreeSet::new();
        let mut current = *start;
        let mut depth = 0;
        while seen.insert(current) {
            let Some(next) = graph.get(current) else {
                return false;
            };
            depth += 1;
            if depth > veac_plan::canonical::MAX_MATTE_NESTING_DEPTH {
                return true;
            }
            current = next;
        }
        false
    })
}
