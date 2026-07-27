use veac_ir::{Animatable, EditOperation, Interpolation, VisualProperty};

use crate::{ClipTimeBinding, ProviderOutput, TransformComponent, TransformSample};

pub(super) fn transform(
    source: &ProviderOutput,
    operation: &EditOperation,
    indices: &[u32],
    component: TransformComponent,
) -> bool {
    let samples = match source {
        ProviderOutput::MotionTracking(value) => &value.transforms,
        ProviderOutput::Stabilization(value) => &value.transforms,
        _ => return false,
    };
    if !complete_indices(indices, samples.len()) {
        return false;
    }
    let EditOperation::SetVisualProperty { property, .. } = operation else {
        return false;
    };
    match (component, property) {
        (TransformComponent::Position, VisualProperty::Position(value)) => {
            curve_values(value, samples, |sample| sample.position)
        }
        (TransformComponent::Scale, VisualProperty::Scale(value)) => {
            curve_values(value, samples, |sample| sample.scale)
        }
        (TransformComponent::Rotation, VisualProperty::RotationDegrees(value)) => {
            curve_values(value, samples, |sample| sample.rotation_degrees)
        }
        _ => false,
    }
}

pub(super) fn crop(
    source: &ProviderOutput,
    operation: &EditOperation,
    indices: &[u32],
    binding: ClipTimeBinding,
    timebase: u32,
    prefix: &str,
) -> bool {
    let samples = match source {
        ProviderOutput::Stabilization(value) => &value.crops,
        ProviderOutput::AutoReframe(value) => &value.crops,
        _ => return false,
    };
    if samples.is_empty() || !complete_indices(indices, samples.len()) {
        return false;
    }
    let EditOperation::SetVisualProperty {
        property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
        ..
    } = operation
    else {
        return false;
    };
    keyframes.len() == samples.len()
        && keyframes
            .iter()
            .zip(samples)
            .enumerate()
            .all(|(index, (keyframe, sample))| {
                keyframe.value == sample.rect
                    && matches!(keyframe.interpolation, Interpolation::Linear)
                    && super::super::time::clip_time(sample.time, binding, timebase)
                        .is_ok_and(|time| time == keyframe.time)
                    && veac_ir::KeyframeId::new(format!("{prefix}_crop_{:04}", index + 1))
                        .is_ok_and(|id| id == keyframe.id)
            })
}

fn curve_values<T: PartialEq>(
    value: &Animatable<T>,
    samples: &[TransformSample],
    expected: impl Fn(&TransformSample) -> T,
) -> bool {
    let Animatable::Keyframes { keyframes } = value else {
        return false;
    };
    keyframes.len() == samples.len()
        && keyframes
            .iter()
            .zip(samples)
            .all(|(keyframe, sample)| keyframe.value == expected(sample))
}

fn complete_indices(values: &[u32], len: usize) -> bool {
    values.len() == len
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| usize::try_from(*value).ok() == Some(index))
}
