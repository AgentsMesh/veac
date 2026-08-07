use super::super::definitions::Definitions;
use super::super::error;
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, CoreValueMetadata, Stage,
};
use crate::program::expression::{
    CollectionOperation, ExpressionError, ValueId, ValueType, ValueTypeKind,
};

#[cfg(test)]
mod tests;

pub(super) fn expected(
    operation: CollectionOperation,
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<CoreValueMetadata, ExpressionError> {
    let CoreInstructionKind::Collection {
        iterable,
        initial,
        callable,
        ..
    } = &instruction.kind
    else {
        unreachable!("aggregate metadata receives a Collection instruction")
    };
    let iterable = definitions.metadata(*iterable);
    let initial = initial.map(|id| definitions.metadata(id));
    let callback = definitions.metadata(*callable);
    let callback_result = callable_result(program, definitions, *callable, instruction)?;
    let result_type = program.value_type(instruction.type_id).ok_or_else(|| {
        error(
            "aggregate result must have a value type",
            instruction.span.clone(),
        )
    })?;
    let invoked = CoreValueMetadata::aggregate_invocation(
        operation,
        iterable,
        initial,
        callback,
        callback_result,
    )
    .ok_or_else(|| {
        error(
            "aggregate callback lacks callable metadata",
            instruction.span.clone(),
        )
    })?;
    verify_effect(operation, invoked.effect, instruction)?;
    let metadata = CoreValueMetadata::aggregate(
        operation,
        iterable,
        initial,
        callback,
        callback_result,
        result_type,
    )
    .expect("aggregate invocation was already validated");
    if metadata.shape_stage() > Stage::Build {
        return Err(error(
            "aggregate topology cannot depend on Temporal-stage data",
            instruction.span.clone(),
        ));
    }
    Ok(metadata)
}

fn verify_effect(
    operation: CollectionOperation,
    effect: crate::program::expression::core::metadata::EffectEvidence,
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    super::super::effect::verify_collection(operation, Some(effect))
        .map_err(|violation| error(violation.message(), instruction.span.clone()))
}

fn callable_result<'a>(
    program: &'a CoreProgram,
    definitions: &Definitions,
    callable: ValueId,
    instruction: &CoreInstruction,
) -> Result<&'a ValueType, ExpressionError> {
    let callable = program
        .value_type(definitions.type_id(callable))
        .ok_or_else(|| {
            error(
                "aggregate callback type is unavailable",
                instruction.span.clone(),
            )
        })?;
    let ValueTypeKind::Function { result, .. } = callable.kind() else {
        return Err(error(
            "aggregate callback must have a function type",
            instruction.span.clone(),
        ));
    };
    Ok(result)
}
