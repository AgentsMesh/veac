use crate::*;

use crate::edit::operation_error;

#[cfg(test)]
mod tests;

pub(super) fn upsert<T: Clone + PartialEq>(
    curve: &mut Animatable<T>,
    keyframe: Keyframe<T>,
) -> Result<bool, Diagnostic> {
    match curve {
        Animatable::Binding { binding_id } => Err(operation_error(
            binding_id.as_str(),
            "a temporal binding cannot be mutated as a keyframe curve",
        )),
        Animatable::Constant { .. } => {
            *curve = Animatable::Keyframes {
                keyframes: vec![keyframe],
            };
            Ok(true)
        }
        Animatable::Keyframes { keyframes } => {
            let current = keyframes.iter().position(|value| value.id == keyframe.id);
            if current.is_some_and(|index| keyframes[index] == keyframe) {
                return Ok(false);
            }
            if keyframes
                .iter()
                .enumerate()
                .any(|(index, value)| Some(index) != current && value.time == keyframe.time)
            {
                return Err(operation_error(
                    keyframe.id.as_str(),
                    "another keyframe already exists at that time",
                ));
            }
            if let Some(index) = current {
                keyframes[index] = keyframe;
            } else {
                keyframes.push(keyframe);
            }
            sort(keyframes);
            Ok(true)
        }
    }
}

pub(super) fn remove<T>(curve: &mut Animatable<T>, id: &KeyframeId) -> Result<bool, Diagnostic> {
    let Animatable::Keyframes { keyframes } = curve else {
        return Ok(false);
    };
    let Some(index) = keyframes.iter().position(|value| value.id == *id) else {
        return Ok(false);
    };
    if keyframes.len() == 1 {
        return Err(operation_error(
            id.as_str(),
            "cannot remove the last keyframe",
        ));
    }
    keyframes.remove(index);
    Ok(true)
}

pub(super) fn move_time<T>(
    curve: &mut Animatable<T>,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<Option<bool>, Diagnostic> {
    let Animatable::Keyframes { keyframes } = curve else {
        return Ok(None);
    };
    let Some(index) = keyframes.iter().position(|value| value.id == *id) else {
        return Ok(None);
    };
    if keyframes[index].time == time {
        return Ok(Some(false));
    }
    if keyframes
        .iter()
        .any(|value| value.id != *id && value.time == time)
    {
        return Err(operation_error(
            id.as_str(),
            "another keyframe already exists at that time",
        ));
    }
    keyframes[index].time = time;
    sort(keyframes);
    Ok(Some(true))
}

fn sort<T>(keyframes: &mut [Keyframe<T>]) {
    keyframes.sort_by(|left, right| {
        left.time
            .partial_cmp(&right.time)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
}
