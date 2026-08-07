use super::{error, type_table};
use crate::program::expression::core::{CoreProgram, CoreValueMetadata, Stage};
use crate::program::expression::{ExpressionError, ValueType, MAX_EXPRESSION_NODES};

pub(super) fn verify(
    program: &CoreProgram,
    nominal_types: &crate::program::TypeRegistry,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
) -> Result<(), ExpressionError> {
    if program.local_slots.len() > MAX_EXPRESSION_NODES {
        return Err(error("Core program contains too many local slots", 0..0));
    }
    let ambient = ambient(program, parameters, parameter_stages, captures);
    for (index, slot) in program.local_slots.iter().enumerate() {
        if slot.id.index() != Some(index) || slot.span.start > slot.span.end {
            return Err(error(
                "local slot declarations must be dense and valid",
                slot.span(),
            ));
        }
        let value_type = type_table::value(program, slot.type_id, slot.span())?;
        if value_type.contains_function_in(nominal_types) != Some(false) {
            return Err(error(
                "local slots cannot contain function values",
                slot.span(),
            ));
        }
        if slot.metadata != ambient {
            return Err(error(
                "local slot metadata is not the exact activation dependency contract",
                slot.span(),
            ));
        }
    }
    Ok(())
}

fn ambient(
    program: &CoreProgram,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
) -> CoreValueMetadata {
    let inputs = program.inputs.iter().map(|input| input.metadata());
    let parameters = parameters
        .iter()
        .zip(parameter_stages)
        .enumerate()
        .map(|(index, (value, stage))| CoreValueMetadata::typed_parameter(*stage, index, value));
    let captures = captures
        .iter()
        .enumerate()
        .map(|(index, value)| CoreValueMetadata::capture(Stage::Const, index, value));
    let values = inputs.chain(parameters).chain(captures).collect::<Vec<_>>();
    CoreValueMetadata::local_mutation(values.iter())
}
