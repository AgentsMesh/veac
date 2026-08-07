use std::collections::BTreeSet;

use crate::program::expression::{CoreTemporalInputIdentity, ResidualizedExpression};
use veac_ir::{
    TemporalBinding, TemporalClock, TemporalClockBinding, TemporalClockOwner,
    TemporalParameterBinding, TemporalParameterId, TemporalProgramId,
};

use super::sink::Context;
use super::{error, ExecutableLowerError};

pub(super) fn lower(
    sink: &crate::program::executable::ExecutableTemporalSink,
    binding_id: &veac_ir::TemporalBindingId,
    parameter_values: &std::collections::BTreeMap<TemporalParameterId, veac_ir::TemporalValue>,
    residual: &ResidualizedExpression,
    program_id: TemporalProgramId,
    context: &Context,
) -> Result<TemporalBinding, ExecutableLowerError> {
    let mut clocks = Vec::new();
    let mut parameters = Vec::new();
    let mut used_parameters = BTreeSet::new();
    for input in residual.inputs() {
        let input_id = input.temporal_input_id();
        match input.identity() {
            CoreTemporalInputIdentity::SequenceTime { sequence_id } => {
                sequence_owner(sequence_id, context)?;
                clocks.push(TemporalClockBinding {
                    input_id,
                    clock: TemporalClock::SequenceTime,
                    owner: TemporalClockOwner::Sequence {
                        sequence_id: sequence_id.clone(),
                    },
                });
            }
            CoreTemporalInputIdentity::Frame { sequence_id } => {
                sequence_owner(sequence_id, context)?;
                clocks.push(TemporalClockBinding {
                    input_id,
                    clock: TemporalClock::Frame,
                    owner: TemporalClockOwner::Sequence {
                        sequence_id: sequence_id.clone(),
                    },
                });
            }
            CoreTemporalInputIdentity::ClipTime { item_id } => {
                item_owner(item_id, context)?;
                clocks.push(item_clock(input_id, TemporalClock::ClipTime, context));
            }
            CoreTemporalInputIdentity::Progress { item_id } => {
                item_owner(item_id, context)?;
                clocks.push(item_clock(input_id, TemporalClock::Progress, context));
            }
            CoreTemporalInputIdentity::SourceTime { source_id } => {
                if context.source_material.as_ref() != Some(source_id) {
                    return Err(error(
                        "EXECUTABLE_TEMPORAL_SOURCE_OWNER",
                        "SourceTime material identity does not match the target clip source",
                    ));
                }
                clocks.push(item_clock(input_id, TemporalClock::SourceTime, context));
            }
            CoreTemporalInputIdentity::Parameter {
                parameter_id,
                value_type,
            } => {
                let value = parameter(parameter_values, parameter_id)?;
                if value.value_type() != *value_type {
                    return Err(error(
                        "EXECUTABLE_TEMPORAL_PARAMETER_TYPE",
                        "Temporal parameter value does not match its declared closed type",
                    ));
                }
                used_parameters.insert(parameter_id.clone());
                parameters.push(TemporalParameterBinding {
                    input_id,
                    parameter_id: parameter_id.clone(),
                    value: value.clone(),
                });
            }
        }
    }
    if used_parameters.len() != parameter_values.len() {
        return Err(error(
            "EXECUTABLE_TEMPORAL_PARAMETER_UNUSED",
            "Temporal host binding contains an undeclared parameter value",
        ));
    }
    Ok(TemporalBinding {
        id: binding_id.clone(),
        program_id,
        result_type: sink.expected_type(),
        clocks,
        parameters,
        provenance_id: residual.provenance().id.clone(),
    })
}

fn sequence_owner(
    value: &veac_ir::SequenceId,
    context: &Context,
) -> Result<(), ExecutableLowerError> {
    (value == &context.sequence_id)
        .then_some(())
        .ok_or_else(|| owner("sequence clock owner does not match the target clip sequence"))
}

fn item_owner(value: &veac_ir::ItemId, context: &Context) -> Result<(), ExecutableLowerError> {
    (context.item_id.as_ref() == Some(value))
        .then_some(())
        .ok_or_else(|| owner("item clock owner does not match the target item sink"))
}

fn item_clock(
    input_id: veac_ir::TemporalInputId,
    clock: TemporalClock,
    context: &Context,
) -> TemporalClockBinding {
    let item_id = context
        .item_id
        .as_ref()
        .expect("verified item clock has an item-owned sink")
        .clone();
    TemporalClockBinding {
        input_id,
        clock,
        owner: TemporalClockOwner::Item { item_id },
    }
}

fn parameter<'a>(
    values: &'a std::collections::BTreeMap<TemporalParameterId, veac_ir::TemporalValue>,
    id: &TemporalParameterId,
) -> Result<&'a veac_ir::TemporalValue, ExecutableLowerError> {
    values.get(id).ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_PARAMETER_MISSING",
            "Temporal parameter declaration has no typed host value",
        )
    })
}

fn owner(message: &'static str) -> ExecutableLowerError {
    error("EXECUTABLE_TEMPORAL_CLOCK_OWNER", message)
}
