use crate::program::executable::ExecutableTemporalSink;
use veac_ir::{Animatable, ApplyOperation, Project, TemporalBindingId};

use super::super::{error, ExecutableLowerError};
use super::clip::{mask_value, parameter_value};

pub(super) fn attach(
    project: &mut Project,
    target: &ExecutableTemporalSink,
    binding_id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    let (sequence_id, apply_id) = ids(target);
    let sequence = project
        .sequences
        .iter_mut()
        .find(|value| &value.id == sequence_id)
        .ok_or_else(missing)?;
    let apply = sequence
        .applies
        .iter_mut()
        .find(|value| &value.id == apply_id)
        .ok_or_else(missing)?;
    match target {
        ExecutableTemporalSink::ApplyOpacity { .. } => {
            apply.mix.opacity = Animatable::Binding { binding_id };
            Ok(())
        }
        ExecutableTemporalSink::ApplyMask {
            mask_index,
            property,
            ..
        } => {
            let mask = apply
                .mix
                .masks
                .get_mut(*mask_index as usize)
                .ok_or_else(optional)?;
            mask_value(mask, *property, binding_id);
            Ok(())
        }
        ExecutableTemporalSink::ApplyEffect {
            stage_id,
            effect_id,
            parameter,
            ..
        } => {
            let stage = apply
                .stages
                .iter_mut()
                .find(|value| &value.id == stage_id)
                .ok_or_else(missing)?;
            let ApplyOperation::Effect { effect } = &mut stage.operation else {
                return Err(owner());
            };
            if &effect.id != effect_id {
                return Err(missing());
            }
            parameter_value(&mut effect.effect, *parameter, binding_id)
        }
        _ => unreachable!("clip sinks dispatch separately"),
    }
}

fn ids(target: &ExecutableTemporalSink) -> (&veac_ir::SequenceId, &veac_ir::ApplyId) {
    match target {
        ExecutableTemporalSink::ApplyOpacity {
            sequence_id,
            apply_id,
        }
        | ExecutableTemporalSink::ApplyMask {
            sequence_id,
            apply_id,
            ..
        }
        | ExecutableTemporalSink::ApplyEffect {
            sequence_id,
            apply_id,
            ..
        } => (sequence_id, apply_id),
        _ => unreachable!("clip sinks dispatch separately"),
    }
}

fn missing() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
        "the requested apply temporal leaf does not exist",
    )
}

fn optional() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_OPTIONAL_SINK",
        "a temporal declaration cannot create an absent apply mask",
    )
}

fn owner() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_SINK_OWNER",
        "the requested apply stage is not an effect stage",
    )
}
